"""Main Exchange SDK client with unified interface."""

import asyncio
from typing import Optional, Callable, Any
from .client import ExchangeClient as RestClient
from .websocket import WebSocketClient
from .cache import CacheService
from .enhancement import (
    EnhancementService,
    EnhancedTrade,
    EnhancedOrder,
    EnhancedBalance,
    EnhancedOrderbookLevel,
)
from .logger import Logger, ConsoleLogger, LogLevel
from .types import Market, Token


class ExchangeClient:
    """
    Main Exchange SDK client with unified REST + WebSocket + Cache interface.

    Example:
        ```python
        client = ExchangeClient(
            rest_url="http://localhost:8888",
            ws_url="ws://localhost:8888/ws"
        )

        # Wait for cache to initialize
        await client.initialize_cache()

        # REST API
        markets = await client.get_markets()

        # WebSocket
        unsubscribe = client.on_trades("BTC/USDC", lambda trade: print(trade))
        ```
    """

    def __init__(
        self,
        rest_url: Optional[str] = None,
        ws_url: Optional[str] = None,
        rest_timeout: float = 30.0,
        ws_reconnect_delays: Optional[list[float]] = None,
        ws_ping_interval: float = 30.0,
        log_level: LogLevel = LogLevel.INFO,
        logger: Optional[Logger] = None,
    ):
        """
        Initialize the Exchange SDK.

        Args:
            rest_url: Base URL for REST API (e.g., "http://localhost:8888")
            ws_url: WebSocket URL (e.g., "ws://localhost:8888/ws")
            rest_timeout: Request timeout in seconds
            ws_reconnect_delays: List of delays between reconnect attempts
            ws_ping_interval: Interval between ping messages
            log_level: Minimum log level
            logger: Custom logger instance
        """
        # If only rest_url provided, derive ws_url
        if rest_url and not ws_url:
            ws_url = rest_url.replace("http://", "ws://").replace("https://", "wss://") + "/ws"

        if not rest_url:
            raise ValueError("rest_url is required")

        # Create shared services
        self.logger = logger or ConsoleLogger(level=log_level)
        self.cache = CacheService(self.logger)
        self.enhancer = EnhancementService(self.cache)

        # Create REST client
        self.rest = RestClient(base_url=rest_url, timeout=rest_timeout)

        # Create WebSocket client if ws_url provided
        self.ws: Optional[WebSocketClient] = None
        if ws_url:
            self.ws = WebSocketClient(
                ws_url=ws_url,
                cache=self.cache,
                enhancer=self.enhancer,
                logger=self.logger,
                reconnect_delays=ws_reconnect_delays,
                ping_interval=ws_ping_interval,
            )

        self._cache_init_task: Optional[asyncio.Task] = None

    async def __aenter__(self):
        """Async context manager entry."""
        await self.initialize_cache()
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Async context manager exit."""
        await self.close()

    async def initialize_cache(self) -> None:
        """
        Initialize the SDK cache with markets and tokens.

        Called automatically, but can also be called manually to force refresh.
        """
        # If already initialized, return immediately
        if self.cache.is_ready():
            return

        # If initialization is in progress, wait for it
        if self._cache_init_task and not self._cache_init_task.done():
            return await self._cache_init_task

        self.logger.info("Starting cache initialization...")

        async def _init():
            try:
                # Fetch markets and tokens in parallel
                markets, tokens = await asyncio.gather(
                    self.rest.get_markets(), self.rest.get_tokens()
                )
                self.cache.set_markets(markets)
                self.cache.set_tokens(tokens)
                self.cache.mark_initialized()
            except Exception as error:
                self.logger.error(f"Failed to initialize cache: {error}")
                self._cache_init_task = None  # Allow retry
                raise

        self._cache_init_task = asyncio.create_task(_init())
        return await self._cache_init_task

    def is_cache_ready(self) -> bool:
        """Check if cache is ready (initialized with markets and tokens)."""
        return self.cache.is_ready()

    async def close(self) -> None:
        """Close all connections."""
        await self.rest.close()
        if self.ws:
            await self.ws.close()

    # ========================================================================
    # Convenience Methods - REST API
    # ========================================================================

    async def get_markets(self) -> list[Market]:
        """
        Get all markets.

        Returns cached markets if available, otherwise fetches from API.
        """
        cached = self.cache.get_all_markets()
        if cached:
            return cached
        markets = await self.rest.get_markets()
        self.cache.set_markets(markets)
        return markets

    async def get_market(self, market_id: str) -> Market:
        """
        Get a specific market.

        Checks cache first, then fetches from API if not found.
        """
        cached = self.cache.get_market(market_id)
        if cached:
            return cached
        market = await self.rest.get_market(market_id)
        return market

    async def get_tokens(self) -> list[Token]:
        """
        Get all tokens.

        Returns cached tokens if available, otherwise fetches from API.
        """
        cached = self.cache.get_all_tokens()
        if cached:
            return cached
        tokens = await self.rest.get_tokens()
        self.cache.set_tokens(tokens)
        return tokens

    async def get_token(self, ticker: str) -> Token:
        """
        Get a specific token.

        Checks cache first, then fetches from API if not found.
        """
        cached = self.cache.get_token(ticker)
        if cached:
            return cached
        token = await self.rest.get_token(ticker)
        return token

    # Delegate other REST methods to rest client
    async def get_balances(self, user_address: str):
        """Get user balances."""
        return await self.rest.get_balances(user_address)

    async def get_orders(self, user_address: str, market_id: Optional[str] = None):
        """Get user orders."""
        return await self.rest.get_orders(user_address, market_id)

    async def get_trades(self, user_address: str, market_id: Optional[str] = None):
        """Get user trades."""
        return await self.rest.get_trades(user_address, market_id)

    async def get_candles(
        self,
        market_id: str,
        interval: str,
        from_timestamp: int,
        to_timestamp: int,
        count_back: Optional[int] = None,
    ):
        """Get OHLCV candles for a market."""
        return await self.rest.get_candles(
            market_id, interval, from_timestamp, to_timestamp, count_back
        )

    async def place_order(
        self,
        user_address: str,
        market_id: str,
        side: str,
        order_type: str,
        price: str,
        size: str,
        signature: str,
    ):
        """Place an order."""
        from .types import Side, OrderType

        return await self.rest.place_order(
            user_address, market_id, Side(side), OrderType(order_type), price, size, signature
        )

    async def cancel_order(self, user_address: str, order_id: str, signature: str):
        """Cancel an order."""
        from uuid import UUID

        return await self.rest.cancel_order(user_address, UUID(order_id), signature)

    async def cancel_all_orders(
        self, user_address: str, signature: str, market_id: Optional[str] = None
    ):
        """Cancel all orders for a user, optionally filtered by market."""
        return await self.rest.cancel_all_orders(user_address, signature, market_id)

    # ========================================================================
    # Convenience Methods - WebSocket (delegate to ws)
    # ========================================================================

    def on_trades(self, market_id: str, handler: Callable[[EnhancedTrade], None]) -> Callable[[], None]:
        """
        Stream trades for a market.

        Returns:
            Unsubscribe function
        """
        if not self.ws:
            raise ValueError("WebSocket client not initialized")
        return self.ws.on_trades(market_id, handler)

    def on_orderbook(
        self,
        market_id: str,
        handler: Callable[
            [dict[str, list[EnhancedOrderbookLevel]]], None
        ],  # {"bids": [...], "asks": [...]}
    ) -> Callable[[], None]:
        """
        Stream orderbook updates for a market.

        Returns:
            Unsubscribe function
        """
        if not self.ws:
            raise ValueError("WebSocket client not initialized")
        return self.ws.on_orderbook(market_id, handler)

    def on_user_orders(
        self, user_address: str, handler: Callable[[dict[str, Any]], None]
    ) -> Callable[[], None]:
        """
        Stream order updates for a user.

        Returns:
            Unsubscribe function
        """
        if not self.ws:
            raise ValueError("WebSocket client not initialized")
        return self.ws.on_user_orders(user_address, handler)

    def on_user_trades(
        self, user_address: str, handler: Callable[[EnhancedTrade], None]
    ) -> Callable[[], None]:
        """
        Stream trade updates for a user.

        Returns:
            Unsubscribe function
        """
        if not self.ws:
            raise ValueError("WebSocket client not initialized")
        return self.ws.on_user_trades(user_address, handler)

    def on_user_balances(
        self, user_address: str, handler: Callable[[dict[str, Any]], None]
    ) -> Callable[[], None]:
        """
        Stream balance updates for a user.

        Returns:
            Unsubscribe function
        """
        if not self.ws:
            raise ValueError("WebSocket client not initialized")
        return self.ws.on_user_balances(user_address, handler)

    def disconnect(self) -> None:
        """Disconnect WebSocket connection."""
        if self.ws:
            asyncio.create_task(self.ws.close())
