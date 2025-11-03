"""Type definitions for the Exchange SDK."""

from datetime import datetime
from enum import Enum
from typing import Optional
from pydantic import BaseModel, Field
from uuid import UUID


# ============================================================================
# Enums
# ============================================================================


class Side(str, Enum):
    """Order side."""

    BUY = "buy"
    SELL = "sell"


class OrderType(str, Enum):
    """Order type."""

    LIMIT = "limit"
    MARKET = "market"


class OrderStatus(str, Enum):
    """Order status."""

    PENDING = "pending"
    OPEN = "open"
    PARTIALLY_FILLED = "partially_filled"
    FILLED = "filled"
    CANCELLED = "cancelled"


class SubscriptionChannel(str, Enum):
    """WebSocket subscription channels."""

    TRADES = "trades"
    ORDERBOOK = "orderbook"
    USER = "user"


# ============================================================================
# Domain Models
# ============================================================================


class Token(BaseModel):
    """Token information."""

    ticker: str
    decimals: int
    name: str


class Market(BaseModel):
    """Market information."""

    id: str
    base_ticker: str
    quote_ticker: str
    tick_size: str
    lot_size: str
    min_size: str
    maker_fee_bps: int
    taker_fee_bps: int


class Order(BaseModel):
    """Order information."""

    id: UUID
    user_address: str
    market_id: str
    price: str
    size: str
    side: Side
    order_type: OrderType
    status: OrderStatus
    filled_size: str
    created_at: datetime
    updated_at: datetime


class Trade(BaseModel):
    """Trade information."""

    id: UUID
    market_id: str
    buyer_address: str
    seller_address: str
    buyer_order_id: UUID
    seller_order_id: UUID
    price: str
    size: str
    timestamp: datetime


class Balance(BaseModel):
    """Balance information."""

    user_address: str
    token_ticker: str
    amount: str
    open_interest: str
    updated_at: datetime


# ============================================================================
# API Response Models
# ============================================================================


class OrderPlaced(BaseModel):
    """Response after placing an order."""

    order: Order
    trades: list[Trade]


class OrderCancelled(BaseModel):
    """Response after cancelling an order."""

    order_id: str


class OrdersCancelled(BaseModel):
    """Response after cancelling multiple orders."""

    cancelled_order_ids: list[str]
    count: int


class Candle(BaseModel):
    """OHLCV candle data."""

    timestamp: int
    open: str
    high: str
    low: str
    close: str
    volume: str


# ============================================================================
# WebSocket Message Models
# ============================================================================


class PriceLevel(BaseModel):
    """Orderbook price level."""

    price: str
    size: str


class OrderbookData(BaseModel):
    """Orderbook snapshot data."""

    market_id: str
    bids: list[PriceLevel]
    asks: list[PriceLevel]
