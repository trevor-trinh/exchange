"""Tests for cache service."""

import pytest
from exchange_sdk import CacheService, NoopLogger
from exchange_sdk.types import Market, Token


@pytest.fixture
def cache():
    """Create a cache service for testing."""
    return CacheService(NoopLogger())


@pytest.fixture
def sample_token():
    """Create a sample token."""
    return Token(ticker="BTC", name="Bitcoin", decimals=8)


@pytest.fixture
def sample_market():
    """Create a sample market."""
    return Market(
        id="BTC/USDC",
        base_ticker="BTC",
        quote_ticker="USDC",
        tick_size="1000000",
        lot_size="100",
        min_size="100",
        maker_fee_bps=10,
        taker_fee_bps=20,
    )


class TestCacheService:
    """Test cache service functionality."""

    def test_token_caching(self, cache, sample_token):
        """Test token caching."""
        # Initially empty
        assert not cache.has_token("BTC")
        assert cache.get_token("BTC") is None
        assert cache.get_all_tokens() == []

        # Add token
        cache.set_tokens([sample_token])
        assert cache.has_token("BTC")
        assert cache.get_token("BTC") == sample_token
        assert cache.get_all_tokens() == [sample_token]

    def test_market_caching(self, cache, sample_market):
        """Test market caching."""
        # Initially empty
        assert not cache.has_market("BTC/USDC")
        assert cache.get_market("BTC/USDC") is None
        assert cache.get_all_markets() == []

        # Add market
        cache.set_markets([sample_market])
        assert cache.has_market("BTC/USDC")
        assert cache.get_market("BTC/USDC") == sample_market
        assert cache.get_all_markets() == [sample_market]

    def test_cache_replacement(self, cache):
        """Test that set_tokens/set_markets replaces existing data."""
        token1 = Token(ticker="BTC", name="Bitcoin", decimals=8)
        token2 = Token(ticker="ETH", name="Ethereum", decimals=18)

        # Set initial tokens
        cache.set_tokens([token1])
        assert len(cache.get_all_tokens()) == 1

        # Replace with new tokens
        cache.set_tokens([token2])
        assert len(cache.get_all_tokens()) == 1
        assert cache.has_token("ETH")
        assert not cache.has_token("BTC")

    def test_cache_ready_state(self, cache, sample_token, sample_market):
        """Test cache ready state."""
        # Not ready initially
        assert not cache.is_ready()

        # Add tokens only - still not ready
        cache.set_tokens([sample_token])
        assert not cache.is_ready()

        # Add markets - still not ready (not marked initialized)
        cache.set_markets([sample_market])
        assert not cache.is_ready()

        # Mark initialized - now ready
        cache.mark_initialized()
        assert cache.is_ready()

    def test_cache_clear(self, cache, sample_token, sample_market):
        """Test cache clearing."""
        cache.set_tokens([sample_token])
        cache.set_markets([sample_market])
        cache.mark_initialized()

        assert cache.is_ready()

        cache.clear()

        assert not cache.is_ready()
        assert cache.get_all_tokens() == []
        assert cache.get_all_markets() == []

    def test_cache_stats(self, cache, sample_token, sample_market):
        """Test cache statistics."""
        stats = cache.get_stats()
        assert stats["tokens"] == 0
        assert stats["markets"] == 0
        assert stats["initialized"] is False

        cache.set_tokens([sample_token])
        cache.set_markets([sample_market])
        cache.mark_initialized()

        stats = cache.get_stats()
        assert stats["tokens"] == 1
        assert stats["markets"] == 1
        assert stats["initialized"] is True
