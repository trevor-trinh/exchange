"""Service for enhancing raw API data with display values."""

from datetime import datetime
from typing import TypedDict
from .types import Order, Trade, Balance
from .cache import CacheService
from .format import to_display_value, format_price, format_size


class EnhancedTrade(TypedDict):
    """Trade with enhanced display values."""

    # Original fields
    id: str
    market_id: str
    buyer_address: str
    seller_address: str
    buyer_order_id: str
    seller_order_id: str
    price: str
    size: str
    side: str
    timestamp: datetime
    # Enhanced fields
    price_display: str
    size_display: str
    price_value: float
    size_value: float


class EnhancedOrder(TypedDict):
    """Order with enhanced display values."""

    # Original fields
    id: str
    user_address: str
    market_id: str
    side: str
    order_type: str
    price: str
    size: str
    filled_size: str
    status: str
    signature: str
    created_at: datetime
    updated_at: datetime
    # Enhanced fields
    price_display: str
    size_display: str
    filled_display: str
    price_value: float
    size_value: float
    filled_value: float


class EnhancedBalance(TypedDict):
    """Balance with enhanced display values."""

    # Original fields
    user_address: str
    token_ticker: str
    amount: str
    open_interest: str
    updated_at: datetime
    # Enhanced fields
    amount_display: str
    locked_display: str
    amount_value: float
    locked_value: float


class EnhancedOrderbookLevel(TypedDict):
    """Orderbook level with enhanced display values."""

    price: str
    size: str
    price_display: str
    size_display: str
    price_value: float
    size_value: float


class WsTradeData(TypedDict):
    """WebSocket trade data (uses Unix timestamp instead of ISO string)."""

    id: str
    market_id: str
    buyer_address: str
    seller_address: str
    buyer_order_id: str
    seller_order_id: str
    price: str
    size: str
    side: str
    timestamp: int  # Unix timestamp in seconds


class EnhancementService:
    """Service for enhancing raw data with display values and conversions."""

    def __init__(self, cache: CacheService):
        """
        Initialize enhancement service.

        Args:
            cache: Cache service instance
        """
        self._cache = cache

    def enhance_trade(self, trade: Trade) -> EnhancedTrade:
        """
        Enhance a REST trade with display values.

        Args:
            trade: Raw trade data

        Returns:
            Enhanced trade with display values
        """
        market = self._cache.get_market(trade.market_id)
        if not market:
            available = ", ".join([m.id for m in self._cache.get_all_markets()]) or "none"
            raise ValueError(
                f"Market {trade.market_id} not found in cache. "
                f"Available markets: {available}. "
                f"Call get_markets() first to populate cache."
            )

        base_token = self._cache.get_token(market.base_ticker)
        quote_token = self._cache.get_token(market.quote_ticker)

        if not base_token or not quote_token:
            available = ", ".join([t.ticker for t in self._cache.get_all_tokens()]) or "none"
            raise ValueError(
                f"Tokens for market {trade.market_id} not found in cache. "
                f"Need: {market.base_ticker}, {market.quote_ticker}. "
                f"Available: {available}. "
                f"Call get_tokens() first to populate cache."
            )

        return EnhancedTrade(
            id=trade.id,
            market_id=trade.market_id,
            buyer_address=trade.buyer_address,
            seller_address=trade.seller_address,
            buyer_order_id=trade.buyer_order_id,
            seller_order_id=trade.seller_order_id,
            price=trade.price,
            size=trade.size,
            side=trade.side.value,
            timestamp=datetime.fromisoformat(trade.timestamp.replace("Z", "+00:00")),
            price_display=format_price(trade.price, quote_token.decimals),
            size_display=format_size(trade.size, base_token.decimals),
            price_value=to_display_value(trade.price, quote_token.decimals),
            size_value=to_display_value(trade.size, base_token.decimals),
        )

    def enhance_ws_trade(self, trade: WsTradeData) -> EnhancedTrade:
        """
        Enhance a WebSocket trade with display values.

        WebSocket trades use Unix timestamps (seconds) instead of ISO strings.

        Args:
            trade: Raw WebSocket trade data

        Returns:
            Enhanced trade with display values
        """
        # Convert to REST trade format
        from .types import Side

        rest_trade = Trade(
            id=trade["id"],
            market_id=trade["market_id"],
            buyer_address=trade["buyer_address"],
            seller_address=trade["seller_address"],
            buyer_order_id=trade["buyer_order_id"],
            seller_order_id=trade["seller_order_id"],
            price=trade["price"],
            size=trade["size"],
            side=Side(trade["side"]),
            timestamp=datetime.fromtimestamp(trade["timestamp"]).isoformat(),
        )

        return self.enhance_trade(rest_trade)

    def enhance_order(self, order: Order, market_id: str) -> EnhancedOrder:
        """
        Enhance an order with display values.

        Args:
            order: Raw order data
            market_id: Market ID

        Returns:
            Enhanced order with display values
        """
        market = self._cache.get_market(market_id)
        if not market:
            raise ValueError(f"Market {market_id} not found in cache. Call get_markets() first.")

        base_token = self._cache.get_token(market.base_ticker)
        quote_token = self._cache.get_token(market.quote_ticker)

        if not base_token or not quote_token:
            raise ValueError(
                f"Tokens for market {market_id} not found in cache. Call get_tokens() first."
            )

        return EnhancedOrder(
            id=order.id,
            user_address=order.user_address,
            market_id=order.market_id,
            side=order.side.value,
            order_type=order.order_type.value,
            price=order.price,
            size=order.size,
            filled_size=order.filled_size,
            status=order.status.value,
            signature=order.signature,
            created_at=datetime.fromisoformat(order.created_at.replace("Z", "+00:00")),
            updated_at=datetime.fromisoformat(order.updated_at.replace("Z", "+00:00")),
            price_display=format_price(order.price, quote_token.decimals),
            size_display=format_size(order.size, base_token.decimals),
            filled_display=format_size(order.filled_size, base_token.decimals),
            price_value=to_display_value(order.price, quote_token.decimals),
            size_value=to_display_value(order.size, base_token.decimals),
            filled_value=to_display_value(order.filled_size, base_token.decimals),
        )

    def enhance_balance(self, balance: Balance) -> EnhancedBalance:
        """
        Enhance a balance with display values.

        Args:
            balance: Raw balance data

        Returns:
            Enhanced balance with display values
        """
        token = self._cache.get_token(balance.token_ticker)
        if not token:
            raise ValueError(
                f"Token {balance.token_ticker} not found in cache. Call get_tokens() first."
            )

        return EnhancedBalance(
            user_address=balance.user_address,
            token_ticker=balance.token_ticker,
            amount=balance.amount,
            open_interest=balance.open_interest,
            updated_at=datetime.fromisoformat(balance.updated_at.replace("Z", "+00:00")),
            amount_display=format_size(balance.amount, token.decimals),
            locked_display=format_size(balance.open_interest, token.decimals),
            amount_value=to_display_value(balance.amount, token.decimals),
            locked_value=to_display_value(balance.open_interest, token.decimals),
        )

    def enhance_orderbook_level(
        self, level: dict[str, str], market_id: str
    ) -> EnhancedOrderbookLevel:
        """
        Enhance an orderbook level with display values.

        Args:
            level: Raw orderbook level with 'price' and 'size'
            market_id: Market ID

        Returns:
            Enhanced orderbook level with display values
        """
        market = self._cache.get_market(market_id)
        if not market:
            raise ValueError(f"Market {market_id} not found in cache. Call get_markets() first.")

        base_token = self._cache.get_token(market.base_ticker)
        quote_token = self._cache.get_token(market.quote_ticker)

        if not base_token or not quote_token:
            raise ValueError(
                f"Tokens for market {market_id} not found in cache. Call get_tokens() first."
            )

        return EnhancedOrderbookLevel(
            price=level["price"],
            size=level["size"],
            price_display=format_price(level["price"], quote_token.decimals),
            size_display=format_size(level["size"], base_token.decimals),
            price_value=to_display_value(level["price"], quote_token.decimals),
            size_value=to_display_value(level["size"], base_token.decimals),
        )
