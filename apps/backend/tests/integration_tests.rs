use chrono::{DateTime, Utc};
use std::str::FromStr;

mod utils;
use utils::TestDb;

#[tokio::test]
async fn test_user_crud_operations() {
    let test_db = TestDb::setup()
        .await
        .expect("Failed to setup test database");

    // Test creating a user
    let user_address = "0x1234567890abcdef";
    let user = test_db
        .create_test_user(user_address)
        .await
        .expect("Failed to create user");

    assert_eq!(user.address, user_address);
    assert!(user.created_at <= Utc::now());

    // Test getting a user
    let retrieved_user = test_db
        .db
        .get_user(user_address)
        .await
        .expect("Failed to get user");

    assert_eq!(retrieved_user.address, user.address);
    assert_eq!(retrieved_user.created_at, user.created_at);

    // Test listing users
    let users = test_db.db.list_users().await.expect("Failed to list users");
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].address, user_address);

    // Test creating another user
    let user2_address = "0xabcdef1234567890";
    let _user2 = test_db
        .create_test_user(user2_address)
        .await
        .expect("Failed to create second user");

    let users = test_db.db.list_users().await.expect("Failed to list users");
    assert_eq!(users.len(), 2);
}

#[tokio::test]
async fn test_market_operations() {
    let test_db = TestDb::setup()
        .await
        .expect("Failed to setup test database");

    // Test creating a market
    let market = test_db
        .create_test_market("BTC", "USD")
        .await
        .expect("Failed to create market");

    assert_eq!(market.base_ticker, "BTC");
    assert_eq!(market.quote_ticker, "USD");
    assert_eq!(market.tick_size, 1000);
    assert_eq!(market.lot_size, 1000000);
    assert_eq!(market.min_size, 1000000);
    assert_eq!(market.maker_fee_bps, 10);
    assert_eq!(market.taker_fee_bps, 20);

    // Test getting a market
    let retrieved_market = test_db
        .db
        .get_market(&market.id)
        .await
        .expect("Failed to get market");

    assert_eq!(retrieved_market.id, market.id);
    assert_eq!(retrieved_market.base_ticker, market.base_ticker);
    assert_eq!(retrieved_market.quote_ticker, market.quote_ticker);
}

#[tokio::test]
async fn test_candle_operations() {
    let test_db = TestDb::setup()
        .await
        .expect("Failed to setup test database");

    // First create a market to associate candles with
    let market = test_db
        .create_test_market("ETH", "USD")
        .await
        .expect("Failed to create market");

    // Test inserting candles
    let timestamp1 = DateTime::from_str("2023-01-01T00:00:00Z").unwrap();
    let timestamp2 = DateTime::from_str("2023-01-01T01:00:00Z").unwrap();
    let timestamp3 = DateTime::from_str("2023-01-01T02:00:00Z").unwrap();

    // Insert test candles (open, high, low, close, volume)
    test_db
        .create_test_candle(&market.id, timestamp1, (100, 110, 95, 105, 1000))
        .await
        .expect("Failed to create candle 1");

    test_db
        .create_test_candle(&market.id, timestamp2, (105, 120, 100, 115, 1500))
        .await
        .expect("Failed to create candle 2");

    test_db
        .create_test_candle(&market.id, timestamp3, (115, 125, 110, 120, 2000))
        .await
        .expect("Failed to create candle 3");

    // Test getting candles
    let start_time = DateTime::from_str("2023-01-01T00:00:00Z").unwrap();
    let end_time = DateTime::from_str("2023-01-01T03:00:00Z").unwrap();

    let candles = test_db
        .db
        .get_candles(&market.id, start_time, end_time)
        .await
        .expect("Failed to get candles");

    assert_eq!(candles.len(), 3);

    println!("candles: {:?}", candles);

    // Verify first candle
    assert_eq!(candles[0].market_id, market.id);
    assert_eq!(candles[0].timestamp, timestamp1);
    assert_eq!(candles[0].open, 100);
    assert_eq!(candles[0].high, 110);
    assert_eq!(candles[0].low, 95);
    assert_eq!(candles[0].close, 105);
    assert_eq!(candles[0].volume, 1000);

    // Test getting candles with narrower time range
    let narrow_start = DateTime::from_str("2023-01-01T01:00:00Z").unwrap();
    let narrow_end = DateTime::from_str("2023-01-01T02:00:00Z").unwrap();

    let narrow_candles = test_db
        .db
        .get_candles(&market.id, narrow_start, narrow_end)
        .await
        .expect("Failed to get narrow candles");

    assert_eq!(narrow_candles.len(), 1);
    assert_eq!(narrow_candles[0].timestamp, timestamp2);
}

#[tokio::test]
async fn test_multiple_markets_and_candles() {
    let test_db = TestDb::setup()
        .await
        .expect("Failed to setup test database");

    // Create multiple markets
    let btc_market = test_db
        .create_test_market("BTC", "USD")
        .await
        .expect("Failed to create BTC market");

    let eth_market = test_db
        .create_test_market("ETH", "USD")
        .await
        .expect("Failed to create ETH market");

    let timestamp = DateTime::from_str("2023-01-01T00:00:00Z").unwrap();

    // Insert candles for both markets
    test_db
        .create_test_candle(&btc_market.id, timestamp, (50000, 51000, 49000, 50500, 100))
        .await
        .expect("Failed to create BTC candle");

    test_db
        .create_test_candle(&eth_market.id, timestamp, (3000, 3100, 2900, 3050, 500))
        .await
        .expect("Failed to create ETH candle");

    // Get candles for each market separately
    let start_time = DateTime::from_str("2023-01-01T00:00:00Z").unwrap();
    let end_time = DateTime::from_str("2023-01-01T01:00:00Z").unwrap();

    let btc_candles = test_db
        .db
        .get_candles(&btc_market.id, start_time, end_time)
        .await
        .expect("Failed to get BTC candles");

    let eth_candles = test_db
        .db
        .get_candles(&eth_market.id, start_time, end_time)
        .await
        .expect("Failed to get ETH candles");

    assert_eq!(btc_candles.len(), 1);
    assert_eq!(eth_candles.len(), 1);

    assert_eq!(btc_candles[0].market_id, btc_market.id);
    assert_eq!(eth_candles[0].market_id, eth_market.id);

    assert_eq!(btc_candles[0].open, 50000);
    assert_eq!(eth_candles[0].open, 3000);
}

#[tokio::test]
async fn test_database_isolation() {
    let test_db = TestDb::setup()
        .await
        .expect("Failed to setup test database");

    // Each test gets a fresh database - verify it starts empty
    let users = test_db.db.list_users().await.expect("Failed to list users");
    assert_eq!(users.len(), 0, "Database should start empty");

    let tokens = test_db
        .db
        .list_tokens()
        .await
        .expect("Failed to list tokens");
    assert_eq!(tokens.len(), 0, "Database should start empty");

    // Create some data
    let _user = test_db
        .create_test_user("test_user")
        .await
        .expect("Failed to create user");

    let market = test_db
        .create_test_market("SOL", "USD")
        .await
        .expect("Failed to create market");

    // Verify data exists
    let users = test_db.db.list_users().await.expect("Failed to list users");
    assert_eq!(users.len(), 1);

    let retrieved_market = test_db
        .db
        .get_market(&market.id)
        .await
        .expect("Failed to get market");
    assert_eq!(retrieved_market.id, market.id);

    // Verify tokens were auto-created by create_test_market
    let tokens = test_db
        .db
        .list_tokens()
        .await
        .expect("Failed to list tokens");
    assert_eq!(tokens.len(), 2, "Should have created SOL and USD tokens");
}
