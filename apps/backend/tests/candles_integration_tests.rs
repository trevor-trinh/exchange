/// Integration tests for the full trade → ClickHouse → candles flow
/// These tests verify end-to-end functionality from trade execution to candle generation
use backend::models::domain::{OrderType, Side};

mod utils;
use utils::{TestDb, TestEngine};

/// Test that trades are persisted to ClickHouse when engine executes them
#[tokio::test]
async fn test_trades_persisted_to_clickhouse() {
    let test_db = TestDb::setup().await.expect("Failed to setup test DB");
    let market = test_db
        .create_test_market_with_tokens("BTC", "USDC")
        .await
        .expect("Failed to create market");

    let engine = TestEngine::new(&test_db).await;

    // Give seller BTC and buyer USDC
    test_db
        .db
        .add_balance("seller", "BTC", 100_000_000) // 1 BTC with 8 decimals
        .await
        .expect("Failed to add BTC to seller");
    test_db
        .db
        .add_balance("buyer", "USDC", 100_000_000_000) // 100,000 USDC with 6 decimals
        .await
        .expect("Failed to add USDC to buyer");

    // Execute a trade
    let sell_order = TestEngine::create_order(
        "seller",
        &market.id,
        Side::Sell,
        OrderType::Limit,
        95000000000, // $95,000
        1000000,     // 1 BTC
    );
    engine
        .place_order(sell_order)
        .await
        .expect("Failed to place sell order");

    let buy_order = TestEngine::create_order(
        "buyer",
        &market.id,
        Side::Buy,
        OrderType::Limit,
        95000000000,
        1000000,
    );
    let result = engine
        .place_order(buy_order)
        .await
        .expect("Failed to place buy order");

    // Verify trade was executed
    assert_eq!(result.trades.len(), 1, "Expected 1 trade to be executed");

    // Wait for ClickHouse to process
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Query ClickHouse to verify trade was persisted
    let query = "SELECT COUNT(*) FROM exchange.trades WHERE market_id = ?";
    let count: u64 = test_db
        .db
        .clickhouse
        .query(query)
        .bind(&market.id)
        .fetch_one::<u64>()
        .await
        .expect("Failed to query trades");

    assert!(
        count >= 1,
        "Expected at least 1 trade in ClickHouse, got {}",
        count
    );
}

/// Test that 1-minute candles are generated from trades via materialized view
#[tokio::test]
async fn test_candles_generated_from_trades() {
    let test_db = TestDb::setup().await.expect("Failed to setup test DB");
    let market = test_db
        .create_test_market_with_tokens("SOL", "USDC")
        .await
        .expect("Failed to create market");

    let engine = TestEngine::new(&test_db).await;

    // Give seller SOL and buyer USDC (enough for multiple trades)
    test_db
        .db
        .add_balance("seller", "SOL", 10_000_000) // 0.1 SOL with 8 decimals (enough for 4 trades)
        .await
        .expect("Failed to add SOL to seller");
    test_db
        .db
        .add_balance("buyer", "USDC", 500_000_000_000) // 500,000 USDC with 6 decimals
        .await
        .expect("Failed to add USDC to buyer");

    // Execute multiple trades to create OHLCV data
    let trades = vec![
        (95000000000, 1000000), // Open: $95,000, 1 BTC
        (95100000000, 1000000), // High: $95,100, 1 BTC
        (94900000000, 1000000), // Low:  $94,900, 1 BTC
        (95050000000, 1000000), // Close: $95,050, 1 BTC
    ];

    for (price, size) in trades {
        let sell_order = TestEngine::create_order(
            "seller",
            &market.id,
            Side::Sell,
            OrderType::Limit,
            price,
            size,
        );
        engine
            .place_order(sell_order)
            .await
            .expect("Failed to place sell order");

        let buy_order = TestEngine::create_order(
            "buyer",
            &market.id,
            Side::Buy,
            OrderType::Limit,
            price,
            size,
        );
        engine
            .place_order(buy_order)
            .await
            .expect("Failed to place buy order");
    }

    // Wait for ClickHouse materialized view to aggregate
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // Query candles table
    let query = "SELECT COUNT(*) FROM exchange.candles WHERE market_id = ? AND interval = '1m'";
    let count: u64 = test_db
        .db
        .clickhouse
        .query(query)
        .bind(&market.id)
        .fetch_one::<u64>()
        .await
        .expect("Failed to query candles");

    assert!(
        count >= 1,
        "Expected at least 1 candle generated, got {}",
        count
    );

    // Query the actual candle data to verify OHLCV
    let query = "SELECT open, high, low, close, volume FROM exchange.candles WHERE market_id = ? AND interval = '1m' ORDER BY timestamp DESC LIMIT 1";
    let candle: Option<(u128, u128, u128, u128, u128)> = test_db
        .db
        .clickhouse
        .query(query)
        .bind(&market.id)
        .fetch_one::<(u128, u128, u128, u128, u128)>()
        .await
        .ok();

    if let Some((open, high, low, close, volume)) = candle {
        // Verify candle has reasonable OHLCV values
        assert!(high >= low, "High must be >= low");
        assert!(high >= open, "High must be >= open");
        assert!(high >= close, "High must be >= close");
        assert!(low <= open, "Low must be <= open");
        assert!(low <= close, "Low must be <= close");
        assert!(volume > 0, "Volume must be > 0");

        // Verify values are in our expected price range
        assert!(
            (94000000000..=96000000000).contains(&open),
            "Open price in reasonable range"
        );
        assert!(
            (94000000000..=96000000000).contains(&high),
            "High price in reasonable range"
        );
        assert!(
            (94000000000..=96000000000).contains(&low),
            "Low price in reasonable range"
        );
        assert!(
            (94000000000..=96000000000).contains(&close),
            "Close price in reasonable range"
        );
    } else {
        panic!("No candle data found");
    }
}

/// Test that empty markets have no candles
#[tokio::test]
async fn test_no_trades_means_no_candles() {
    let test_db = TestDb::setup().await.expect("Failed to setup test DB");
    let market = test_db
        .create_test_market_with_tokens("ETH", "USDC")
        .await
        .expect("Failed to create market");

    // Don't execute any trades

    // Wait a bit
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Query candles - should be empty
    let query = "SELECT COUNT(*) FROM exchange.candles WHERE market_id = ?";
    let count: u64 = test_db
        .db
        .clickhouse
        .query(query)
        .bind(&market.id)
        .fetch_one::<u64>()
        .await
        .expect("Failed to query candles");

    assert_eq!(count, 0, "Expected no candles for market with no trades");
}
