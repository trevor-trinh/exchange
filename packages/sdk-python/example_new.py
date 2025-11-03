"""Example usage of the new Exchange SDK with all features."""

import asyncio
from exchange_sdk import ExchangeClient, LogLevel


async def main():
    # Create unified client
    client = ExchangeClient(
        rest_url="http://localhost:8888",
        ws_url="ws://localhost:8888/ws",
        log_level=LogLevel.INFO,
    )

    # Initialize cache (automatically populates markets and tokens)
    await client.initialize_cache()

    # ========================================================================
    # REST API Examples
    # ========================================================================

    # Get markets (cached after first call)
    markets = await client.get_markets()
    print(f"Found {len(markets)} markets:")
    for market in markets:
        print(f"  - {market.id}: {market.base_ticker}/{market.quote_ticker}")

    # Get tokens (cached after first call)
    tokens = await client.get_tokens()
    print(f"\nFound {len(tokens)} tokens:")
    for token in tokens:
        print(f"  - {token.ticker}: {token.name} ({token.decimals} decimals)")

    # Get specific market from cache
    btc_usdc = await client.get_market("BTC/USDC")
    print(f"\nBTC/USDC Market: {btc_usdc}")

    # Get user balances
    user_address = "user123"
    balances = await client.get_balances(user_address)
    print(f"\nBalances for {user_address}:")
    for balance in balances:
        print(f"  - {balance.token_ticker}: {balance.amount}")

    # ========================================================================
    # WebSocket Examples with Convenience Methods
    # ========================================================================

    # Subscribe to trades for BTC/USDC (automatically enhanced with display values)
    def handle_trade(trade):
        print(f"\n🔔 Trade: {trade['side']} {trade['size_display']} @ {trade['price_display']}")

    unsubscribe_trades = client.on_trades("BTC/USDC", handle_trade)

    # Subscribe to orderbook updates (automatically enhanced)
    def handle_orderbook(update):
        print(f"\n📊 Orderbook Update:")
        print(f"  Best Bid: {update['bids'][0]['price_display'] if update['bids'] else 'N/A'}")
        print(f"  Best Ask: {update['asks'][0]['price_display'] if update['asks'] else 'N/A'}")

    unsubscribe_orderbook = client.on_orderbook("BTC/USDC", handle_orderbook)

    # Subscribe to user orders
    def handle_user_order(order):
        print(f"\n📦 Order Update: {order['order_id']} - {order['status']}")

    unsubscribe_user_orders = client.on_user_orders(user_address, handle_user_order)

    # Subscribe to user trades
    def handle_user_trade(trade):
        print(f"\n💰 User Trade: {trade['side']} {trade['size_display']} @ {trade['price_display']}")

    unsubscribe_user_trades = client.on_user_trades(user_address, handle_user_trade)

    # Subscribe to balance updates
    def handle_balance(balance):
        print(f"\n💵 Balance Update: {balance['token_ticker']} = {balance['available']}")

    unsubscribe_balances = client.on_user_balances(user_address, handle_balance)

    # Keep running to receive WebSocket messages
    print("\n✅ Subscribed to all channels. Press Ctrl+C to exit...")
    try:
        await asyncio.sleep(60)  # Run for 60 seconds
    except KeyboardInterrupt:
        print("\n🛑 Shutting down...")

    # Cleanup: unsubscribe from all channels
    unsubscribe_trades()
    unsubscribe_orderbook()
    unsubscribe_user_orders()
    unsubscribe_user_trades()
    unsubscribe_balances()

    # Close client
    await client.close()
    print("✅ Client closed")


if __name__ == "__main__":
    asyncio.run(main())
