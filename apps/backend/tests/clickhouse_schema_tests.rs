/// Tests to verify ClickHouse schema matches Rust structs
/// These tests catch schema mismatches that cause runtime panics
use backend::models::db::{CandleInsertRow, ClickHouseTradeRow};
use exchange_test_utils::TestContainers;

#[tokio::test]
async fn test_clickhouse_trades_schema_matches_struct() {
    let containers = TestContainers::setup()
        .await
        .expect("Failed to setup containers");
    let db = containers.db_clone();

    // Try to insert a dummy trade row
    let trade = ClickHouseTradeRow {
        id: "test-id-123".to_string(),
        market_id: "BTC/USDC".to_string(),
        buyer_address: "buyer".to_string(),
        seller_address: "seller".to_string(),
        buyer_order_id: "buyer-order-id".to_string(),
        seller_order_id: "seller-order-id".to_string(),
        price: 95000000000,
        size: 1000000,
        timestamp: 1234567890,
    };

    // This will panic if schema doesn't match struct
    let result = db
        .clickhouse
        .insert::<ClickHouseTradeRow>("trades")
        .await
        .unwrap()
        .write(&trade)
        .await;

    assert!(
        result.is_ok(),
        "Failed to insert trade - schema mismatch! Error: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_clickhouse_candles_schema_matches_struct() {
    let containers = TestContainers::setup()
        .await
        .expect("Failed to setup containers");
    let db = containers.db_clone();

    // Try to insert a dummy candle row using CandleInsertRow
    let candle = CandleInsertRow {
        market_id: "BTC/USDC".to_string(),
        timestamp: 1234567890,
        trade_time: 1234567890,
        interval: "1m".to_string(),
        open: 95000000000,
        high: 95100000000,
        low: 94900000000,
        close: 95050000000,
        volume: 10000000,
    };

    // This will panic if schema doesn't match struct
    let result = db
        .clickhouse
        .insert::<CandleInsertRow>("candles")
        .await
        .unwrap()
        .write(&candle)
        .await;

    assert!(
        result.is_ok(),
        "Failed to insert candle - schema mismatch! Error: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_trades_table_has_all_required_columns() {
    let containers = TestContainers::setup()
        .await
        .expect("Failed to setup containers");
    let db = containers.db_clone();

    // Query the table schema - just get column names
    let query = "SELECT name FROM system.columns WHERE database = 'exchange' AND table = 'trades'";
    let columns: Vec<String> = db
        .clickhouse
        .query(query)
        .fetch_all::<String>()
        .await
        .expect("Failed to query table schema");

    // Check all required columns exist
    let required = vec![
        "id",
        "market_id",
        "buyer_address",
        "seller_address",
        "buyer_order_id",
        "seller_order_id",
        "price",
        "size",
        "timestamp",
    ];

    for col in required {
        assert!(
            columns.contains(&col.to_string()),
            "Missing required column: {}. Available columns: {:?}",
            col,
            columns
        );
    }
}

#[tokio::test]
async fn test_candles_table_has_all_required_columns() {
    let containers = TestContainers::setup()
        .await
        .expect("Failed to setup containers");
    let db = containers.db_clone();

    // Query the table schema - just get column names
    let query = "SELECT name FROM system.columns WHERE database = 'exchange' AND table = 'candles'";
    let columns: Vec<String> = db
        .clickhouse
        .query(query)
        .fetch_all::<String>()
        .await
        .expect("Failed to query table schema");

    // Check all required columns exist
    let required = vec![
        "market_id",
        "timestamp",
        "interval",
        "open",
        "high",
        "low",
        "close",
        "volume",
    ];

    for col in required {
        assert!(
            columns.contains(&col.to_string()),
            "Missing required column: {}. Available columns: {:?}",
            col,
            columns
        );
    }
}

/// Test that we can insert and retrieve a trade
/// Note: Disabled due to ClickHouse eventual consistency - data isn't immediately queryable
#[tokio::test]
async fn test_trades_roundtrip() {
    let containers = TestContainers::setup()
        .await
        .expect("Failed to setup containers");
    let db = containers.db_clone();

    let trade = ClickHouseTradeRow {
        id: "test-trade-roundtrip".to_string(),
        market_id: "BTC/USDC".to_string(),
        buyer_address: "buyer1".to_string(),
        seller_address: "seller1".to_string(),
        buyer_order_id: "buyer-order-1".to_string(),
        seller_order_id: "seller-order-1".to_string(),
        price: 95000000000,
        size: 1000000,
        timestamp: 1234567890,
    };

    // Insert trade
    let mut insert = db
        .clickhouse
        .insert::<ClickHouseTradeRow>("trades")
        .await
        .expect("Failed to create insert");
    insert.write(&trade).await.expect("Failed to write trade");
    insert.end().await.expect("Failed to end insert");

    // Wait for ClickHouse to process the insert (eventual consistency)
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Query back
    let query = "SELECT * FROM exchange.trades WHERE id = ?";
    let rows = db
        .clickhouse
        .query(query)
        .bind("test-trade-roundtrip")
        .fetch_all::<ClickHouseTradeRow>()
        .await
        .expect("Failed to fetch trade");

    assert_eq!(rows.len(), 1, "Should retrieve exactly one trade");
    let retrieved = &rows[0];

    assert_eq!(retrieved.id, trade.id);
    assert_eq!(retrieved.market_id, trade.market_id);
    assert_eq!(retrieved.price, trade.price);
    assert_eq!(retrieved.size, trade.size);
}
