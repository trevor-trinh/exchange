# Exchange Bots

Trading bots that mirror Binance activity to create a realistic trading simulator.

## Architecture

This package contains two bots that work together:

### 1. Orderbook Mirror Bot (Maker)
- Connects to Binance WebSocket to stream orderbook updates
- Mirrors the top N price levels on both bid and ask sides
- Continuously updates orders as Binance's orderbook changes
- Provides liquidity to the exchange

### 2. Trade Mirror Bot (Taker)
- Connects to Binance WebSocket to stream trade executions
- Executes matching trades on our exchange
- Creates realistic price movement and trading volume
- Ensures price follows Binance

## Features

- **Real-time Sync**: Sub-second latency from Binance to our exchange
- **Configurable Scaling**: Adjust order sizes via `size_multiplier`
- **Robust Error Handling**: Automatic reconnection and error recovery
- **Structured Logging**: Full visibility into bot operations

## Usage

### Environment Variables

```bash
EXCHANGE_URL=http://localhost:8001  # Your exchange URL
BINANCE_SYMBOL=BTCUSDC              # Binance symbol to mirror
MARKET_ID=BTC/USDC                  # Market ID on your exchange
MAKER_ADDRESS=maker_bot             # Maker bot wallet address
TAKER_ADDRESS=taker_bot             # Taker bot wallet address
```

### Running the Bots

```bash
# Using cargo
cd apps/bots
cargo run

# Or from project root
just run-bots  # If you add this to justfile
```

### Configuration

Edit `config.toml` to customize:
- Orderbook depth levels
- Size multipliers
- Update intervals
- Minimum trade sizes

## How It Works

1. **Initialization**
   - Both bots connect to Binance WebSocket streams
   - Orderbook bot fetches initial snapshot via REST API
   - Bots authenticate with exchange using configured addresses

2. **Orderbook Sync** (Maker Bot)
   - Receives depth updates from Binance
   - Maintains local orderbook state
   - Cancels stale orders and places new ones
   - Mirrors top 5 levels (configurable) on each side

3. **Trade Execution** (Taker Bot)
   - Receives trade events from Binance
   - Determines side (buy/sell) based on aggressor
   - Places market orders to execute matching trades
   - Applies size multiplier and minimum thresholds

4. **Price Scaling**
   - Converts Binance decimal prices to u128 format
   - Applies 6 decimal precision (configurable)
   - Ensures compatibility with exchange API

## Dependencies

- **exchange-sdk**: Rust SDK for exchange API
- **tokio-tungstenite**: WebSocket client
- **rust_decimal**: Precise decimal arithmetic
- **tracing**: Structured logging

## Testing

```bash
cargo test
```

## Future Enhancements

- [ ] Support multiple markets simultaneously
- [ ] Add performance metrics and monitoring
- [ ] Implement inventory management for maker bot
- [ ] Add configurable spread adjustment
- [ ] Support other exchanges beyond Binance
- [ ] Add AI trading bot strategies
- [ ] Implement random noise trader bot
