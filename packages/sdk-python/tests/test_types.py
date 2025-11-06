"""Tests for type imports and usage."""

import pytest
from pydantic import ValidationError


class TestGeneratedTypes:
    """Test that generated types work correctly."""

    def test_client_message_import(self):
        """Test importing ClientMessage."""
        from exchange_sdk import ClientMessage

        # Should be able to validate a subscribe message
        msg = {
            "type": "subscribe",
            "channel": "trades",
            "market_id": "BTC/USDC",
            "user_address": None,
        }

        # Pydantic v2 validation - ClientMessage is a RootModel
        validated = ClientMessage.model_validate(msg)
        assert validated is not None

    def test_server_message_import(self):
        """Test importing ServerMessage."""
        from exchange_sdk import ServerMessage

        # Should be able to validate a subscribed message
        msg = {
            "type": "subscribed",
            "channel": "trades",
            "market_id": "BTC/USDC",
            "user_address": None,
        }

        validated = ServerMessage.model_validate(msg)
        assert validated is not None

    def test_trade_data_import(self):
        """Test importing TradeData."""
        from exchange_sdk import TradeData

        trade = TradeData(
            id="trade123",
            market_id="BTC/USDC",
            buyer_address="buyer",
            seller_address="seller",
            buyer_order_id="order1",
            seller_order_id="order2",
            price="110000000000",
            size="50000000",
            side="buy",
            timestamp=1234567890,
        )

        assert trade.id == "trade123"
        assert trade.market_id == "BTC/USDC"
        assert trade.side.value == "buy"  # Side is an enum

    def test_orderbook_data_import(self):
        """Test importing OrderbookData."""
        from exchange_sdk import OrderbookData, PriceLevel

        level = PriceLevel(price="110000000000", size="50000000")

        orderbook = OrderbookData(
            market_id="BTC/USDC",
            bids=[level],
            asks=[level],
        )

        assert orderbook.market_id == "BTC/USDC"
        assert len(orderbook.bids) == 1
        assert len(orderbook.asks) == 1

    def test_price_level_import(self):
        """Test importing PriceLevel."""
        from exchange_sdk import PriceLevel

        level = PriceLevel(price="110000000000", size="50000000")

        assert level.price == "110000000000"
        assert level.size == "50000000"

    def test_subscription_channel_import(self):
        """Test importing SubscriptionChannel."""
        from exchange_sdk import SubscriptionChannel

        # Should be an enum with correct values (uppercase)
        assert hasattr(SubscriptionChannel, "TRADES")
        assert hasattr(SubscriptionChannel, "ORDERBOOK")
        assert hasattr(SubscriptionChannel, "USER_FILLS")
        assert hasattr(SubscriptionChannel, "USER_ORDERS")
        assert hasattr(SubscriptionChannel, "USER_BALANCES")

        assert SubscriptionChannel.TRADES.value == "trades"
        assert SubscriptionChannel.ORDERBOOK.value == "orderbook"
        assert SubscriptionChannel.USER_FILLS.value == "user_fills"
        assert SubscriptionChannel.USER_ORDERS.value == "user_orders"
        assert SubscriptionChannel.USER_BALANCES.value == "user_balances"


class TestRestApiTypes:
    """Test REST API types."""

    def test_market_import(self):
        """Test importing Market type."""
        from exchange_sdk import Market

        market = Market(
            id="BTC/USDC",
            base_ticker="BTC",
            quote_ticker="USDC",
            tick_size="1000000",
            lot_size="100",
            min_size="100",
            maker_fee_bps=10,
            taker_fee_bps=20,
        )

        assert market.id == "BTC/USDC"
        assert market.base_ticker == "BTC"
        assert market.quote_ticker == "USDC"

    def test_token_import(self):
        """Test importing Token type."""
        from exchange_sdk import Token

        token = Token(ticker="BTC", name="Bitcoin", decimals=8)

        assert token.ticker == "BTC"
        assert token.name == "Bitcoin"
        assert token.decimals == 8

    def test_side_enum(self):
        """Test Side enum."""
        from exchange_sdk import Side

        assert Side.BUY.value == "buy"
        assert Side.SELL.value == "sell"

    def test_order_type_enum(self):
        """Test OrderType enum."""
        from exchange_sdk import OrderType

        assert OrderType.LIMIT.value == "limit"
        assert OrderType.MARKET.value == "market"

    def test_order_status_enum(self):
        """Test OrderStatus enum."""
        from exchange_sdk import OrderStatus

        assert OrderStatus.OPEN.value == "open"
        assert OrderStatus.FILLED.value == "filled"
        assert OrderStatus.CANCELLED.value == "cancelled"


class TestStripelikeImports:
    """Test that all types can be imported from single package (Stripe-like)."""

    def test_single_import_statement(self):
        """Test importing everything from exchange_sdk."""
        from exchange_sdk import (
            # Main client
            ExchangeClient,
            # Generated types
            ClientMessage,
            ServerMessage,
            TradeData,
            OrderbookData,
            PriceLevel,
            SubscriptionChannel,
            # REST types
            Market,
            Token,
            Order,
            Trade,
            Balance,
            Side,
            OrderType,
            OrderStatus,
            # Enhanced types
            EnhancedTrade,
            EnhancedOrder,
            EnhancedBalance,
            # Utilities
            format_price,
            format_size,
            to_atoms,
            to_display_value,
            # Logger
            LogLevel,
            ConsoleLogger,
            # Services
            CacheService,
            EnhancementService,
        )

        # All imports should succeed
        assert ExchangeClient is not None
        assert ClientMessage is not None
        assert ServerMessage is not None
        assert Market is not None
        assert format_price is not None
        assert LogLevel is not None
