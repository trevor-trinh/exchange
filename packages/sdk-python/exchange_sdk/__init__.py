"""Exchange SDK for Python."""

# Main unified client
from .sdk import ExchangeClient

# Individual clients
from .client import ExchangeClient as RestClient
from .websocket import WebSocketClient

# Services
from .cache import CacheService
from .enhancement import (
    EnhancementService,
    EnhancedTrade,
    EnhancedOrder,
    EnhancedBalance,
    EnhancedOrderbookLevel,
    WsTradeData,
)
from .logger import Logger, ConsoleLogger, NoopLogger, LogLevel
from .format import (
    to_display_value,
    to_atoms,
    format_price,
    format_size,
    format_number,
)

# Types
from .types import (
    Side,
    OrderType,
    OrderStatus,
    Order,
    Trade,
    Market,
    Token,
    Balance,
    SubscriptionChannel,
    Candle,
)

# Exceptions
from .exceptions import (
    ExchangeError,
    APIError,
    ConnectionError,
    TimeoutError,
    WebSocketError,
)

__all__ = [
    # Main client
    "ExchangeClient",
    # Individual clients
    "RestClient",
    "WebSocketClient",
    # Services
    "CacheService",
    "EnhancementService",
    "ConsoleLogger",
    "NoopLogger",
    "Logger",
    "LogLevel",
    # Enhanced types
    "EnhancedTrade",
    "EnhancedOrder",
    "EnhancedBalance",
    "EnhancedOrderbookLevel",
    "WsTradeData",
    # Format utilities
    "to_display_value",
    "to_atoms",
    "format_price",
    "format_size",
    "format_number",
    # Domain types
    "Side",
    "OrderType",
    "OrderStatus",
    "Order",
    "Trade",
    "Market",
    "Token",
    "Balance",
    "SubscriptionChannel",
    "Candle",
    # Exceptions
    "ExchangeError",
    "APIError",
    "ConnectionError",
    "TimeoutError",
    "WebSocketError",
]

__version__ = "0.1.0"
