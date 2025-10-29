# Exchange SDK (Rust)

Type-safe Rust SDK for interacting with the exchange API.

## Features

- **REST Client**: Full API coverage for trading, market data, and user information
- **WebSocket Client**: Real-time data streams for trades, orderbook, and user updates
- **Type Safety**: Uses backend types directly for guaranteed compatibility
- **Async/Await**: Built on tokio for efficient async operations

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
exchange-sdk = { path = "../../packages/sdk-rust" }
```

## Usage

### REST Client

```rust
use exchange_sdk::{ExchangeClient, Side, OrderType};

#[tokio::main]
async fn main() {
    let client = ExchangeClient::new("http://localhost:8001");

    // Get all markets
    let markets = client.get_markets().await.unwrap();
    println!("Markets: {:?}", markets);

    // Get user balances
    let balances = client.get_balances("user_address").await.unwrap();
    println!("Balances: {:?}", balances);

    // Place an order
    let order = client
        .place_order(
            "user_address".to_string(),
            1, // market_id
            Side::Buy,
            OrderType::Limit,
            67000_000000000000000000, // price
            1_000000000000000000,     // size
            "signature".to_string(),
        )
        .await
        .unwrap();
    println!("Order placed: {:?}", order);
}
```

### WebSocket Client

```rust
use exchange_sdk::{WebSocketClient, SubscriptionChannel};

#[tokio::main]
async fn main() {
    let ws_client = WebSocketClient::new("ws://localhost:8001/ws");
    let mut handle = ws_client.connect().await.unwrap();

    // Subscribe to trades
    handle
        .subscribe(
            SubscriptionChannel::Trades,
            Some("BTC-USD".to_string()),
            None,
        )
        .unwrap();

    // Receive messages
    while let Some(msg) = handle.recv().await {
        println!("Received: {:?}", msg);
    }
}
```

## Running Tests

```bash
cargo test
```

Integration tests will automatically spin up PostgreSQL and ClickHouse containers using testcontainers.
