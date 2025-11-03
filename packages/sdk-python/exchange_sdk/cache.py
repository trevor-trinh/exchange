"""Cache service for markets and tokens."""

from typing import Optional
from .types import Market, Token
from .logger import Logger


class CacheService:
    """Cache service for storing and retrieving markets and tokens."""

    def __init__(self, logger: Logger):
        """
        Initialize cache service.

        Args:
            logger: Logger instance
        """
        self._tokens_cache: dict[str, Token] = {}
        self._markets_cache: dict[str, Market] = {}
        self._initialized = False
        self._logger = logger

    # ===== Tokens =====

    def set_tokens(self, tokens: list[Token]) -> None:
        """
        Cache tokens.

        Args:
            tokens: List of tokens to cache
        """
        self._tokens_cache.clear()
        for token in tokens:
            self._tokens_cache[token.ticker] = token
        self._logger.debug(f"Cached {len(tokens)} tokens")

    def get_token(self, ticker: str) -> Optional[Token]:
        """
        Get token by ticker.

        Args:
            ticker: Token ticker symbol

        Returns:
            Token or None if not found
        """
        return self._tokens_cache.get(ticker)

    def get_all_tokens(self) -> list[Token]:
        """Get all cached tokens."""
        return list(self._tokens_cache.values())

    def has_token(self, ticker: str) -> bool:
        """
        Check if token is cached.

        Args:
            ticker: Token ticker symbol

        Returns:
            True if token is cached
        """
        return ticker in self._tokens_cache

    # ===== Markets =====

    def set_markets(self, markets: list[Market]) -> None:
        """
        Cache markets.

        Args:
            markets: List of markets to cache
        """
        self._markets_cache.clear()
        for market in markets:
            self._markets_cache[market.id] = market
        self._logger.debug(f"Cached {len(markets)} markets")

    def get_market(self, market_id: str) -> Optional[Market]:
        """
        Get market by ID.

        Args:
            market_id: Market ID

        Returns:
            Market or None if not found
        """
        return self._markets_cache.get(market_id)

    def get_all_markets(self) -> list[Market]:
        """Get all cached markets."""
        return list(self._markets_cache.values())

    def has_market(self, market_id: str) -> bool:
        """
        Check if market is cached.

        Args:
            market_id: Market ID

        Returns:
            True if market is cached
        """
        return market_id in self._markets_cache

    # ===== Cache State =====

    def is_ready(self) -> bool:
        """
        Check if cache is ready.

        Returns:
            True if cache is initialized and has data
        """
        return self._initialized and len(self._tokens_cache) > 0 and len(self._markets_cache) > 0

    def mark_initialized(self) -> None:
        """Mark cache as initialized."""
        self._initialized = True
        self._logger.info(
            f"Cache initialized: {len(self._markets_cache)} markets, {len(self._tokens_cache)} tokens"
        )

    def clear(self) -> None:
        """Clear all cached data."""
        self._tokens_cache.clear()
        self._markets_cache.clear()
        self._initialized = False
        self._logger.debug("Cache cleared")

    def get_stats(self) -> dict[str, int | bool]:
        """
        Get cache statistics.

        Returns:
            Dictionary with cache stats
        """
        return {
            "markets": len(self._markets_cache),
            "tokens": len(self._tokens_cache),
            "initialized": self._initialized,
        }
