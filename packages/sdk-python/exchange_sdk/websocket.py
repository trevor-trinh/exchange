"""WebSocket client for real-time exchange data with convenience methods."""

import asyncio
import json
from typing import Any, Callable, Optional
import websockets
from websockets.client import WebSocketClientProtocol

from .exceptions import WebSocketError
from .types import SubscriptionChannel
from .cache import CacheService
from .enhancement import EnhancementService, EnhancedTrade, EnhancedOrderbookLevel, WsTradeData
from .logger import Logger


class WebSocketClient:
    """
    WebSocket client for real-time exchange data.

    Features:
    - Auto-reconnect with exponential backoff
    - Reference counting for subscriptions (prevents duplicates)
    - Automatic resubscription after reconnect
    - Ping/pong with timeout detection
    - Convenience methods with data enhancement
    - Type-safe message handlers
    """

    def __init__(
        self,
        ws_url: str,
        cache: CacheService,
        enhancer: EnhancementService,
        logger: Logger,
        reconnect_delays: Optional[list[float]] = None,
        ping_interval: float = 30.0,
    ):
        """
        Initialize WebSocket client.

        Args:
            ws_url: WebSocket URL
            cache: Cache service instance
            enhancer: Enhancement service instance
            logger: Logger instance
            reconnect_delays: List of delays (in seconds) between reconnect attempts
            ping_interval: Interval (in seconds) between ping messages
        """
        self.ws_url = ws_url
        self.cache = cache
        self.enhancer = enhancer
        self.logger = logger
        self.reconnect_delays = reconnect_delays or [1, 2, 4, 8, 16]
        self.ping_interval = ping_interval

        self._ws: Optional[WebSocketClientProtocol] = None
        self._is_connected = False
        self._reconnect_attempt = 0
        self._reconnect_task: Optional[asyncio.Task] = None
        self._ping_task: Optional[asyncio.Task] = None
        self._receive_task: Optional[asyncio.Task] = None
        self._last_pong_time = 0.0
        self._pong_timeout = 60.0  # seconds
        self._should_reconnect = True

        # Message queue for when disconnected
        self._message_queue: list[dict[str, Any]] = []

        # Message handlers: message_type -> set of handlers
        self._handlers: dict[str, set[Callable]] = {}

        # Active subscriptions with reference counting
        # Key format: "channel:identifier" (e.g., "trades:BTC/USDC", "user:0x123")
        self._active_subscriptions: dict[str, int] = {}

        # Auto-connect
        asyncio.create_task(self._connect())

    async def _connect(self) -> None:
        """Connect to WebSocket server."""
        if self._ws and not self._ws.closed:
            return

        try:
            self.logger.debug(f"Connecting to {self.ws_url}...")
            self._ws = await websockets.connect(self.ws_url)
            self._is_connected = True
            self._reconnect_attempt = 0
            self._last_pong_time = asyncio.get_event_loop().time()

            self.logger.info("WebSocket connected")

            # Re-subscribe to all active subscriptions
            await self._resubscribe_all()

            # Send queued messages
            while self._message_queue:
                msg = self._message_queue.pop(0)
                await self._send(msg)

            # Start ping and receive tasks
            self._ping_task = asyncio.create_task(self._ping_loop())
            self._receive_task = asyncio.create_task(self._receive_loop())

        except Exception as e:
            self.logger.error(f"Failed to connect: {e}")
            self._is_connected = False
            await self._schedule_reconnect()

    async def _receive_loop(self) -> None:
        """Receive and handle messages."""
        try:
            while self._ws and not self._ws.closed:
                try:
                    message_str = await self._ws.recv()
                    message = json.loads(message_str)
                    await self._handle_message(message)
                except websockets.exceptions.ConnectionClosed:
                    self.logger.warn("WebSocket connection closed")
                    break
                except Exception as e:
                    self.logger.error(f"Error receiving message: {e}")

        except asyncio.CancelledError:
            pass
        finally:
            self._is_connected = False
            if self._should_reconnect:
                await self._schedule_reconnect()

    async def _handle_message(self, message: dict[str, Any]) -> None:
        """Handle incoming message."""
        msg_type = message.get("type")

        # Handle pong messages automatically
        if msg_type == "pong":
            self._last_pong_time = asyncio.get_event_loop().time()
            return

        # Call registered handlers
        if msg_type and msg_type in self._handlers:
            for handler in list(self._handlers[msg_type]):
                try:
                    # Handle both sync and async handlers
                    result = handler(message)
                    if asyncio.iscoroutine(result):
                        await result
                except Exception as e:
                    self.logger.error(f"Error in {msg_type} handler: {e}")

    async def _ping_loop(self) -> None:
        """Send periodic ping messages."""
        try:
            while self._should_reconnect and self._is_connected:
                await asyncio.sleep(self.ping_interval)

                # Check if we've received a pong recently
                time_since_pong = asyncio.get_event_loop().time() - self._last_pong_time
                if time_since_pong > self._pong_timeout:
                    self.logger.warn("No pong received, reconnecting...")
                    if self._ws:
                        await self._ws.close()
                    break

                # Send ping
                await self._send({"type": "ping"})

        except asyncio.CancelledError:
            pass

    async def _schedule_reconnect(self) -> None:
        """Schedule reconnection attempt."""
        if not self._should_reconnect:
            return

        delay_index = min(self._reconnect_attempt, len(self.reconnect_delays) - 1)
        delay = self.reconnect_delays[delay_index]

        self.logger.info(f"Reconnecting in {delay}s (attempt {self._reconnect_attempt + 1})...")
        await asyncio.sleep(delay)

        self._reconnect_attempt += 1
        await self._connect()

    async def _resubscribe_all(self) -> None:
        """Re-subscribe to all active subscriptions after reconnect."""
        if not self._active_subscriptions:
            return

        self.logger.info(f"Re-subscribing to {len(self._active_subscriptions)} subscriptions")

        for key in self._active_subscriptions.keys():
            channel, identifier = key.split(":", 1)

            message = {
                "type": "subscribe",
                "channel": channel,
                "market_id": identifier if channel in ["trades", "orderbook"] else None,
                "user_address": identifier if channel == "user" else None,
            }

            await self._send(message)
            self.logger.debug(f"Re-subscribed to {key}")

    async def _send(self, message: dict[str, Any]) -> None:
        """Send message to server."""
        if not self._is_connected or not self._ws or self._ws.closed:
            self._message_queue.append(message)
            return

        try:
            await self._ws.send(json.dumps(message))
        except Exception as e:
            self.logger.error(f"Failed to send message: {e}")
            self._message_queue.append(message)

    def _get_subscription_key(self, channel: str, market_id: Optional[str] = None, user_address: Optional[str] = None) -> str:
        """Get subscription key for tracking."""
        identifier = market_id or user_address or "global"
        return f"{channel}:{identifier}"

    async def subscribe(
        self, channel: SubscriptionChannel, market_id: Optional[str] = None, user_address: Optional[str] = None
    ) -> None:
        """
        Subscribe to a channel (with reference counting).

        Args:
            channel: Subscription channel
            market_id: Optional market ID (for trades/orderbook)
            user_address: Optional user address (for user updates)
        """
        key = self._get_subscription_key(channel.value, market_id, user_address)
        current_count = self._active_subscriptions.get(key, 0)

        # Only send subscribe message if this is the first subscription
        if current_count == 0:
            self.logger.debug(f"Subscribing to {key}")
            message = {
                "type": "subscribe",
                "channel": channel.value,
                "market_id": market_id,
                "user_address": user_address,
            }
            await self._send(message)
        else:
            self.logger.debug(f"Already subscribed to {key} (count: {current_count}), incrementing ref count")

        # Increment reference count
        self._active_subscriptions[key] = current_count + 1

    async def unsubscribe(
        self, channel: SubscriptionChannel, market_id: Optional[str] = None, user_address: Optional[str] = None
    ) -> None:
        """
        Unsubscribe from a channel (with reference counting).

        Args:
            channel: Subscription channel
            market_id: Optional market ID
            user_address: Optional user address
        """
        key = self._get_subscription_key(channel.value, market_id, user_address)
        current_count = self._active_subscriptions.get(key, 0)

        if current_count == 0:
            self.logger.warn(f"Attempted to unsubscribe from {key} but no active subscription found")
            return

        new_count = current_count - 1

        # Only send unsubscribe message when ref count reaches 0
        if new_count == 0:
            self.logger.debug(f"Unsubscribing from {key}")
            message = {
                "type": "unsubscribe",
                "channel": channel.value,
                "market_id": market_id,
                "user_address": user_address,
            }
            await self._send(message)
            del self._active_subscriptions[key]
        else:
            self.logger.debug(f"Decrementing ref count for {key} (count: {new_count})")
            self._active_subscriptions[key] = new_count

    def on(self, message_type: str, handler: Callable) -> Callable[[], None]:
        """
        Register a message handler.

        Args:
            message_type: Type of message to handle
            handler: Handler function

        Returns:
            Unsubscribe function
        """
        if message_type not in self._handlers:
            self._handlers[message_type] = set()
        self._handlers[message_type].add(handler)

        # Return unsubscribe function
        def unsubscribe():
            if message_type in self._handlers:
                self._handlers[message_type].discard(handler)

        return unsubscribe

    def off(self, message_type: str, handler: Callable) -> None:
        """Remove a message handler."""
        if message_type in self._handlers:
            self._handlers[message_type].discard(handler)

    # ========================================================================
    # Type-Safe Convenience Methods
    # ========================================================================

    def on_trades(self, market_id: str, handler: Callable[[EnhancedTrade], None]) -> Callable[[], None]:
        """
        Stream trades for a market (enhanced with display values).

        Args:
            market_id: Market ID
            handler: Handler function that receives enhanced trades

        Returns:
            Unsubscribe function
        """
        self.logger.debug(f"Setting up trades subscription for {market_id}")

        def trade_handler(msg: dict[str, Any]):
            if msg.get("type") != "trade":
                return

            trade_data = msg.get("trade", {})
            if trade_data.get("market_id") != market_id:
                return

            if not self.cache.is_ready():
                self.logger.warn("Trade received before cache initialized, skipping")
                return

            try:
                ws_trade: WsTradeData = {
                    "id": trade_data["id"],
                    "market_id": trade_data["market_id"],
                    "buyer_address": trade_data["buyer_address"],
                    "seller_address": trade_data["seller_address"],
                    "buyer_order_id": trade_data["buyer_order_id"],
                    "seller_order_id": trade_data["seller_order_id"],
                    "price": trade_data["price"],
                    "size": trade_data["size"],
                    "side": trade_data["side"],
                    "timestamp": trade_data["timestamp"],
                }
                enhanced = self.enhancer.enhance_ws_trade(ws_trade)
                handler(enhanced)
            except Exception as e:
                self.logger.error(f"Failed to enhance trade: {e}")

        remove_handler = self.on("trade", trade_handler)
        asyncio.create_task(self.subscribe(SubscriptionChannel.TRADES, market_id=market_id))

        def unsubscribe():
            self.logger.debug(f"Cleaning up trades subscription for {market_id}")
            remove_handler()
            asyncio.create_task(self.unsubscribe(SubscriptionChannel.TRADES, market_id=market_id))

        return unsubscribe

    def on_orderbook(
        self,
        market_id: str,
        handler: Callable[[dict[str, list[EnhancedOrderbookLevel]]], None],
    ) -> Callable[[], None]:
        """
        Stream orderbook updates for a market (enhanced with display values).

        Args:
            market_id: Market ID
            handler: Handler function that receives {"bids": [...], "asks": [...]}

        Returns:
            Unsubscribe function
        """
        self.logger.debug(f"Setting up orderbook subscription for {market_id}")

        def orderbook_handler(msg: dict[str, Any]):
            if msg.get("type") != "orderbook":
                return

            orderbook_data = msg.get("orderbook", {})
            if orderbook_data.get("market_id") != market_id:
                return

            if not self.cache.is_ready():
                self.logger.warn("Orderbook received before cache initialized, skipping")
                return

            try:
                enhanced_bids = [
                    self.enhancer.enhance_orderbook_level(bid, market_id)
                    for bid in orderbook_data.get("bids", [])
                ]
                enhanced_asks = [
                    self.enhancer.enhance_orderbook_level(ask, market_id)
                    for ask in orderbook_data.get("asks", [])
                ]
                handler({"bids": enhanced_bids, "asks": enhanced_asks})
            except Exception as e:
                self.logger.error(f"Failed to enhance orderbook: {e}")

        remove_handler = self.on("orderbook", orderbook_handler)
        asyncio.create_task(self.subscribe(SubscriptionChannel.ORDERBOOK, market_id=market_id))

        def unsubscribe():
            self.logger.debug(f"Cleaning up orderbook subscription for {market_id}")
            remove_handler()
            asyncio.create_task(self.unsubscribe(SubscriptionChannel.ORDERBOOK, market_id=market_id))

        return unsubscribe

    def on_user_orders(
        self, user_address: str, handler: Callable[[dict[str, Any]], None]
    ) -> Callable[[], None]:
        """
        Stream order updates for a user.

        Args:
            user_address: User address
            handler: Handler function that receives order updates

        Returns:
            Unsubscribe function
        """
        self.logger.debug(f"Setting up user orders subscription for {user_address}")

        def order_handler(msg: dict[str, Any]):
            if msg.get("type") == "order":
                handler({
                    "order_id": msg.get("order_id"),
                    "status": msg.get("status"),
                    "filled_size": msg.get("filled_size"),
                })

        remove_handler = self.on("order", order_handler)
        asyncio.create_task(self.subscribe(SubscriptionChannel.USER, user_address=user_address))

        def unsubscribe():
            self.logger.debug(f"Cleaning up user orders subscription for {user_address}")
            remove_handler()
            asyncio.create_task(self.unsubscribe(SubscriptionChannel.USER, user_address=user_address))

        return unsubscribe

    def on_user_trades(
        self, user_address: str, handler: Callable[[EnhancedTrade], None]
    ) -> Callable[[], None]:
        """
        Stream trade updates for a user (enhanced with display values).

        Args:
            user_address: User address
            handler: Handler function that receives enhanced trades

        Returns:
            Unsubscribe function
        """
        self.logger.debug(f"Setting up user trades subscription for {user_address}")

        def trade_handler(msg: dict[str, Any]):
            if msg.get("type") != "trade":
                return

            if not self.cache.is_ready():
                self.logger.warn("Trade received before cache initialized, skipping")
                return

            try:
                trade_data = msg.get("trade", {})
                ws_trade: WsTradeData = {
                    "id": trade_data["id"],
                    "market_id": trade_data["market_id"],
                    "buyer_address": trade_data["buyer_address"],
                    "seller_address": trade_data["seller_address"],
                    "buyer_order_id": trade_data["buyer_order_id"],
                    "seller_order_id": trade_data["seller_order_id"],
                    "price": trade_data["price"],
                    "size": trade_data["size"],
                    "side": trade_data["side"],
                    "timestamp": trade_data["timestamp"],
                }
                enhanced = self.enhancer.enhance_ws_trade(ws_trade)
                handler(enhanced)
            except Exception as e:
                self.logger.error(f"Failed to enhance user trade: {e}")

        remove_handler = self.on("trade", trade_handler)
        asyncio.create_task(self.subscribe(SubscriptionChannel.USER, user_address=user_address))

        def unsubscribe():
            self.logger.debug(f"Cleaning up user trades subscription for {user_address}")
            remove_handler()
            asyncio.create_task(self.unsubscribe(SubscriptionChannel.USER, user_address=user_address))

        return unsubscribe

    def on_user_balances(
        self, user_address: str, handler: Callable[[dict[str, Any]], None]
    ) -> Callable[[], None]:
        """
        Stream balance updates for a user.

        Args:
            user_address: User address
            handler: Handler function that receives balance updates

        Returns:
            Unsubscribe function
        """
        self.logger.debug(f"Setting up user balances subscription for {user_address}")

        def balance_handler(msg: dict[str, Any]):
            if msg.get("type") == "balance":
                handler({
                    "token_ticker": msg.get("token_ticker"),
                    "available": msg.get("available"),
                    "locked": msg.get("locked"),
                })

        remove_handler = self.on("balance", balance_handler)
        asyncio.create_task(self.subscribe(SubscriptionChannel.USER, user_address=user_address))

        def unsubscribe():
            self.logger.debug(f"Cleaning up user balances subscription for {user_address}")
            remove_handler()
            asyncio.create_task(self.unsubscribe(SubscriptionChannel.USER, user_address=user_address))

        return unsubscribe

    def is_ready(self) -> bool:
        """Check if WebSocket is connected and ready."""
        return self._is_connected and self._ws is not None and not self._ws.closed

    async def close(self) -> None:
        """Close WebSocket connection."""
        self._should_reconnect = False

        # Cancel tasks
        if self._ping_task:
            self._ping_task.cancel()
            try:
                await self._ping_task
            except asyncio.CancelledError:
                pass

        if self._receive_task:
            self._receive_task.cancel()
            try:
                await self._receive_task
            except asyncio.CancelledError:
                pass

        if self._reconnect_task:
            self._reconnect_task.cancel()
            try:
                await self._reconnect_task
            except asyncio.CancelledError:
                pass

        # Close WebSocket
        if self._ws:
            await self._ws.close()
            self._ws = None

        self._is_connected = False
        self.logger.info("WebSocket closed")
