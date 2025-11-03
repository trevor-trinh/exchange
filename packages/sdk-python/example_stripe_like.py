"""
Example showing Stripe-like SDK usage with types baked in.

Just import from exchange_sdk and everything is available - no separate type imports needed!
"""

import asyncio

# Everything imported from one place - Stripe-like! 🎉
from exchange_sdk import (
    ExchangeClient,
    LogLevel,
    # Generated WebSocket types are baked in
    ClientMessage,
    ServerMessage,
    TradeData,
    OrderbookData,
    PriceLevel,
    # REST API types
    Market,
    Token,
    Order,
    Trade,
    Balance,
    Side,
    OrderType,
    # Enhanced types with display values
    EnhancedTrade,
    # Format utilities
    format_price,
    format_size,
    to_atoms,
    to_display_value,
)


async def main():
    print("🚀 Exchange SDK with Stripe-like type imports!\n")

    # ========================================================================
    # Create client - types are available immediately
    # ========================================================================

    client = ExchangeClient(
        rest_url="http://localhost:8888",
        log_level=LogLevel.INFO,
    )

    await client.initialize_cache()

    # ========================================================================
    # Use generated types (no separate import needed!)
    # ========================================================================

    # Create a subscribe message using generated types
    subscribe_msg: ClientMessage = {
        "type": "subscribe",
        "channel": "trades",
        "market_id": "BTC/USDC",
        "user_address": None,
    }
    print(f"📨 Subscribe message: {subscribe_msg}\n")

    # ========================================================================
    # REST API with type hints
    # ========================================================================

    # Get markets - returns typed Market objects
    markets: list[Market] = await client.get_markets()
    btc_market: Market = markets[0] if markets else None

    if btc_market:
        print(f"📊 Market: {btc_market.id}")
        print(f"   Base: {btc_market.base_ticker}")
        print(f"   Quote: {btc_market.quote_ticker}")
        print(f"   Tick Size: {btc_market.tick_size}")
        print()

    # Get tokens - returns typed Token objects
    tokens: list[Token] = await client.get_tokens()
    btc_token: Token = next((t for t in tokens if t.ticker == "BTC"), None)

    if btc_token:
        print(f"🪙 Token: {btc_token.ticker}")
        print(f"   Name: {btc_token.name}")
        print(f"   Decimals: {btc_token.decimals}")
        print()

    # ========================================================================
    # Use format utilities (also baked in!)
    # ========================================================================

    # Convert atoms to display value
    price_atoms = "110000500000"  # 110,000.50 USDC (6 decimals)
    price_display = format_price(price_atoms, 6)
    price_value = to_display_value(price_atoms, 6)

    print(f"💰 Price formatting:")
    print(f"   Atoms: {price_atoms}")
    print(f"   Display: {price_display}")
    print(f"   Value: {price_value}")
    print()

    # Convert decimal to atoms
    size_decimal = "0.5"  # 0.5 BTC
    size_atoms = to_atoms(size_decimal, 8)
    size_display = format_size(size_atoms, 8)

    print(f"📏 Size formatting:")
    print(f"   Decimal: {size_decimal} BTC")
    print(f"   Atoms: {size_atoms}")
    print(f"   Display: {size_display}")
    print()

    # ========================================================================
    # WebSocket with enhanced types
    # ========================================================================

    def handle_trade(trade: EnhancedTrade):
        """Handle enhanced trades - types are fully known!"""
        print(f"🔔 Trade:")
        print(f"   Side: {trade['side']}")
        print(f"   Price: {trade['price_display']} ({trade['price_value']})")
        print(f"   Size: {trade['size_display']} ({trade['size_value']})")
        print(f"   Timestamp: {trade['timestamp']}")

    def handle_orderbook(update: dict[str, list[PriceLevel]]):
        """Handle orderbook - PriceLevel type is known!"""
        best_bid: PriceLevel = update["bids"][0] if update["bids"] else None
        best_ask: PriceLevel = update["asks"][0] if update["asks"] else None

        if best_bid:
            print(f"📊 Best Bid: {best_bid['price_display']}")
        if best_ask:
            print(f"📊 Best Ask: {best_ask['price_display']}")

    print("✅ All types available from single import - Stripe-like!\n")
    print("   - ClientMessage, ServerMessage")
    print("   - TradeData, OrderbookData, PriceLevel")
    print("   - Market, Token, Order, Trade, Balance")
    print("   - EnhancedTrade, EnhancedOrder, EnhancedBalance")
    print("   - format_price, format_size, to_atoms, to_display_value")
    print("   - ExchangeClient, LogLevel, Side, OrderType")
    print("\n🎉 No separate type imports needed!\n")

    await client.close()


if __name__ == "__main__":
    asyncio.run(main())
