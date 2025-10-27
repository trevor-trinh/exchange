use backend::db::Db;
use backend::engine::MatchingEngine;
use backend::models::domain::{EngineEvent, EngineRequest, Order, OrderStatus, OrderType, Side};
use chrono::Utc;
use tokio::sync::{broadcast, mpsc, oneshot};
use uuid::Uuid;

mod utils;
use utils::TestDb;

/// Helper to create a matching engine with channels for testing
struct TestEngine {
    #[allow(dead_code)]
    db: Db,
    engine_tx: mpsc::Sender<EngineRequest>,
    #[allow(dead_code)]
    event_rx: broadcast::Receiver<EngineEvent>,
}

impl TestEngine {
    async fn new(test_db: &TestDb) -> Self {
        let (engine_tx, engine_rx) = mpsc::channel::<EngineRequest>(100);
        let (event_tx, event_rx) = broadcast::channel::<EngineEvent>(1000);

        let engine = MatchingEngine::new(test_db.db.clone(), engine_rx, event_tx);

        // Spawn engine in background
        tokio::spawn(async move {
            engine.run().await;
        });

        Self {
            db: test_db.db.clone(),
            engine_tx,
            event_rx,
        }
    }

    /// Helper to place an order and get the response
    async fn place_order(&self, order: Order) -> Result<backend::models::api::OrderPlaced, String> {
        let (response_tx, response_rx) = oneshot::channel();

        self.engine_tx
            .send(EngineRequest::PlaceOrder { order, response_tx })
            .await
            .map_err(|e| format!("Failed to send order: {}", e))?;

        response_rx
            .await
            .map_err(|e| format!("Failed to receive response: {}", e))?
            .map_err(|e| format!("Order placement failed: {}", e))
    }

    /// Helper to cancel an order
    async fn cancel_order(
        &self,
        order_id: Uuid,
        user_address: String,
    ) -> Result<backend::models::api::OrderCancelled, String> {
        let (response_tx, response_rx) = oneshot::channel();

        self.engine_tx
            .send(EngineRequest::CancelOrder {
                order_id,
                user_address,
                response_tx,
            })
            .await
            .map_err(|e| format!("Failed to send cancel request: {}", e))?;

        response_rx
            .await
            .map_err(|e| format!("Failed to receive response: {}", e))?
            .map_err(|e| format!("Order cancellation failed: {}", e))
    }

    /// Helper to create a test order
    fn create_order(
        user_address: &str,
        market_id: &str,
        side: Side,
        order_type: OrderType,
        price: u128,
        size: u128,
    ) -> Order {
        Order {
            id: Uuid::new_v4(),
            user_address: user_address.to_string(),
            market_id: market_id.to_string(),
            price,
            size,
            side,
            order_type,
            status: OrderStatus::Pending,
            filled_size: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[tokio::test]
async fn test_basic_limit_order_matching() {
    let test_db = TestDb::setup().await.expect("Failed to setup test DB");
    let market = test_db
        .create_test_market_with_tokens("BTC", "USDC")
        .await
        .expect("Failed to create market");

    let engine = TestEngine::new(&test_db).await;

    // Create sell order first (maker)
    let sell_order = TestEngine::create_order(
        "seller",
        &market.id,
        Side::Sell,
        OrderType::Limit,
        50_000_000_000, // $50,000
        1_000_000,      // 1 BTC
    );

    let result = engine.place_order(sell_order.clone()).await;
    assert!(result.is_ok(), "Failed to place sell order: {:?}", result);
    let placed = result.unwrap();
    assert_eq!(placed.order.status, OrderStatus::Pending);
    assert_eq!(placed.trades.len(), 0); // No match yet

    // Create buy order that matches (taker)
    let buy_order = TestEngine::create_order(
        "buyer",
        &market.id,
        Side::Buy,
        OrderType::Limit,
        50_000_000_000, // Willing to pay $50,000
        1_000_000,      // 1 BTC
    );

    let result = engine.place_order(buy_order.clone()).await;
    assert!(result.is_ok(), "Failed to place buy order: {:?}", result);
    let placed = result.unwrap();

    // Should be fully filled
    assert_eq!(placed.order.status, OrderStatus::Filled);
    assert_eq!(placed.order.filled_size, 1_000_000);
    assert_eq!(placed.trades.len(), 1);

    // Verify trade details
    let trade = &placed.trades[0];
    assert_eq!(trade.buyer_address, "buyer");
    assert_eq!(trade.seller_address, "seller");
    assert_eq!(trade.price, 50_000_000_000); // Match at maker's price
    assert_eq!(trade.size, 1_000_000);
}

#[tokio::test]
async fn test_partial_fill() {
    let test_db = TestDb::setup().await.expect("Failed to setup test DB");
    let market = test_db
        .create_test_market_with_tokens("ETH", "USDC")
        .await
        .expect("Failed to create market");

    let engine = TestEngine::new(&test_db).await;

    // Create sell order for 10 ETH (maker)
    let sell_order = TestEngine::create_order(
        "seller",
        &market.id,
        Side::Sell,
        OrderType::Limit,
        3_000_000_000, // $3,000
        10_000_000,    // 10 ETH
    );

    let result = engine.place_order(sell_order).await;
    assert!(result.is_ok());

    // Create buy order for only 3 ETH (taker)
    let buy_order = TestEngine::create_order(
        "buyer",
        &market.id,
        Side::Buy,
        OrderType::Limit,
        3_000_000_000,
        3_000_000, // 3 ETH
    );

    let result = engine.place_order(buy_order).await;
    assert!(result.is_ok());
    let placed = result.unwrap();

    // Buyer's order should be fully filled
    assert_eq!(placed.order.status, OrderStatus::Filled);
    assert_eq!(placed.order.filled_size, 3_000_000);
    assert_eq!(placed.trades.len(), 1);

    // Trade should be for 3 ETH
    assert_eq!(placed.trades[0].size, 3_000_000);
}

#[tokio::test]
async fn test_price_time_priority() {
    let test_db = TestDb::setup().await.expect("Failed to setup test DB");
    let market = test_db
        .create_test_market_with_tokens("SOL", "USDC")
        .await
        .expect("Failed to create market");

    let engine = TestEngine::new(&test_db).await;

    // Place two sell orders at different prices
    let sell1 = TestEngine::create_order(
        "seller1",
        &market.id,
        Side::Sell,
        OrderType::Limit,
        100_000_000, // $100
        5_000_000,   // 5 SOL
    );

    let sell2 = TestEngine::create_order(
        "seller2",
        &market.id,
        Side::Sell,
        OrderType::Limit,
        95_000_000, // $95 (better price)
        3_000_000,  // 3 SOL
    );

    engine
        .place_order(sell1)
        .await
        .expect("Failed to place sell1");
    engine
        .place_order(sell2)
        .await
        .expect("Failed to place sell2");

    // Place buy order that can match both
    let buy = TestEngine::create_order(
        "buyer",
        &market.id,
        Side::Buy,
        OrderType::Limit,
        105_000_000, // Willing to pay $105
        8_000_000,   // 8 SOL
    );

    let result = engine.place_order(buy).await;
    assert!(result.is_ok());
    let placed = result.unwrap();

    // Should match with better price first (sell2 at $95)
    assert_eq!(placed.trades.len(), 2);
    assert_eq!(placed.trades[0].price, 95_000_000); // Best price first
    assert_eq!(placed.trades[0].size, 3_000_000);
    assert_eq!(placed.trades[1].price, 100_000_000); // Then second best
    assert_eq!(placed.trades[1].size, 5_000_000);

    // Buyer should be fully filled
    assert_eq!(placed.order.status, OrderStatus::Filled);
    assert_eq!(placed.order.filled_size, 8_000_000);
}

#[tokio::test]
async fn test_fifo_time_priority_at_same_price() {
    let test_db = TestDb::setup().await.expect("Failed to setup test DB");
    let market = test_db
        .create_test_market_with_tokens("AVAX", "USDC")
        .await
        .expect("Failed to create market");

    let engine = TestEngine::new(&test_db).await;

    // Place two sell orders at the SAME price (first one should match first)
    let sell1 = TestEngine::create_order(
        "seller1",
        &market.id,
        Side::Sell,
        OrderType::Limit,
        40_000_000, // $40
        2_000_000,  // 2 AVAX
    );

    let sell2 = TestEngine::create_order(
        "seller2",
        &market.id,
        Side::Sell,
        OrderType::Limit,
        40_000_000, // $40 (same price)
        2_000_000,  // 2 AVAX
    );

    engine
        .place_order(sell1)
        .await
        .expect("Failed to place sell1");
    engine
        .place_order(sell2)
        .await
        .expect("Failed to place sell2");

    // Place buy order for only 2 AVAX
    let buy = TestEngine::create_order(
        "buyer",
        &market.id,
        Side::Buy,
        OrderType::Limit,
        40_000_000,
        2_000_000,
    );

    let result = engine.place_order(buy).await;
    assert!(result.is_ok());
    let placed = result.unwrap();

    // Should only match with first order (FIFO)
    assert_eq!(placed.trades.len(), 1);
    assert_eq!(placed.trades[0].seller_address, "seller1"); // First in, first matched
    assert_eq!(placed.trades[0].size, 2_000_000);
}

#[tokio::test]
async fn test_market_order_execution() {
    let test_db = TestDb::setup().await.expect("Failed to setup test DB");
    let market = test_db
        .create_test_market_with_tokens("MATIC", "USDC")
        .await
        .expect("Failed to create market");

    let engine = TestEngine::new(&test_db).await;

    // Place limit sell orders at different prices
    let sell1 = TestEngine::create_order(
        "seller1",
        &market.id,
        Side::Sell,
        OrderType::Limit,
        1_000_000, // $1.00
        10_000_000,
    );

    let sell2 = TestEngine::create_order(
        "seller2",
        &market.id,
        Side::Sell,
        OrderType::Limit,
        1_100_000, // $1.10
        10_000_000,
    );

    engine
        .place_order(sell1)
        .await
        .expect("Failed to place sell1");
    engine
        .place_order(sell2)
        .await
        .expect("Failed to place sell2");

    // Place market buy order (should match at any price)
    let market_buy = TestEngine::create_order(
        "buyer",
        &market.id,
        Side::Buy,
        OrderType::Market,
        0, // Price doesn't matter for market orders
        15_000_000,
    );

    let result = engine.place_order(market_buy).await;
    assert!(result.is_ok());
    let placed = result.unwrap();

    // Should match with both orders
    assert_eq!(placed.trades.len(), 2);
    assert_eq!(placed.trades[0].price, 1_000_000); // Best price first
    assert_eq!(placed.trades[0].size, 10_000_000);
    assert_eq!(placed.trades[1].price, 1_100_000); // Then worse price
    assert_eq!(placed.trades[1].size, 5_000_000);

    // Buyer should be fully filled
    assert_eq!(placed.order.status, OrderStatus::Filled);
    assert_eq!(placed.order.filled_size, 15_000_000);
}

#[tokio::test]
async fn test_order_cancellation() {
    let test_db = TestDb::setup().await.expect("Failed to setup test DB");
    let market = test_db
        .create_test_market_with_tokens("DOT", "USDC")
        .await
        .expect("Failed to create market");

    let engine = TestEngine::new(&test_db).await;

    // Place a limit order
    let order = TestEngine::create_order(
        "user1",
        &market.id,
        Side::Sell,
        OrderType::Limit,
        7_000_000,
        5_000_000,
    );
    let order_id = order.id;

    let result = engine.place_order(order).await;
    assert!(result.is_ok());

    // Cancel the order
    let cancel_result = engine.cancel_order(order_id, "user1".to_string()).await;
    assert!(
        cancel_result.is_ok(),
        "Failed to cancel order: {:?}",
        cancel_result
    );

    let cancelled = cancel_result.unwrap();
    assert_eq!(cancelled.order_id, order_id.to_string());
}

#[tokio::test]
async fn test_cannot_cancel_others_order() {
    let test_db = TestDb::setup().await.expect("Failed to setup test DB");
    let market = test_db
        .create_test_market_with_tokens("ATOM", "USDC")
        .await
        .expect("Failed to create market");

    let engine = TestEngine::new(&test_db).await;

    // Place order as user1
    let order = TestEngine::create_order(
        "user1",
        &market.id,
        Side::Sell,
        OrderType::Limit,
        10_000_000,
        5_000_000,
    );
    let order_id = order.id;

    engine
        .place_order(order)
        .await
        .expect("Failed to place order");

    // Try to cancel as user2 (should fail)
    let cancel_result = engine.cancel_order(order_id, "user2".to_string()).await;
    assert!(
        cancel_result.is_err(),
        "Should not be able to cancel other user's order"
    );
}

#[tokio::test]
async fn test_no_match_different_market() {
    let test_db = TestDb::setup().await.expect("Failed to setup test DB");
    let market1 = test_db
        .create_test_market_with_tokens("BTC", "USDC")
        .await
        .expect("Failed to create market1");
    let market2 = test_db
        .create_test_market_with_tokens("ETH", "USDC")
        .await
        .expect("Failed to create market2");

    let engine = TestEngine::new(&test_db).await;

    // Place sell order in BTC/USDC market
    let sell_btc = TestEngine::create_order(
        "seller",
        &market1.id,
        Side::Sell,
        OrderType::Limit,
        50_000_000_000,
        1_000_000,
    );

    engine
        .place_order(sell_btc)
        .await
        .expect("Failed to place BTC sell");

    // Place buy order in ETH/USDC market (different market)
    let buy_eth = TestEngine::create_order(
        "buyer",
        &market2.id,
        Side::Buy,
        OrderType::Limit,
        3_000_000_000,
        10_000_000,
    );

    let result = engine.place_order(buy_eth).await;
    assert!(result.is_ok());
    let placed = result.unwrap();

    // Should not match (different markets)
    assert_eq!(placed.trades.len(), 0);
    assert_eq!(placed.order.status, OrderStatus::Pending);
}

#[tokio::test]
async fn test_buy_limit_order_wont_match_above_limit() {
    let test_db = TestDb::setup().await.expect("Failed to setup test DB");
    let market = test_db
        .create_test_market_with_tokens("LINK", "USDC")
        .await
        .expect("Failed to create market");

    let engine = TestEngine::new(&test_db).await;

    // Place sell order at $20
    let sell = TestEngine::create_order(
        "seller",
        &market.id,
        Side::Sell,
        OrderType::Limit,
        20_000_000, // $20
        5_000_000,
    );

    engine
        .place_order(sell)
        .await
        .expect("Failed to place sell");

    // Place buy limit order at $15 (below sell price)
    let buy = TestEngine::create_order(
        "buyer",
        &market.id,
        Side::Buy,
        OrderType::Limit,
        15_000_000, // $15 (won't match $20 ask)
        5_000_000,
    );

    let result = engine.place_order(buy).await;
    assert!(result.is_ok());
    let placed = result.unwrap();

    // Should not match
    assert_eq!(placed.trades.len(), 0);
    assert_eq!(placed.order.status, OrderStatus::Pending);
}

#[tokio::test]
async fn test_sell_limit_order_wont_match_below_limit() {
    let test_db = TestDb::setup().await.expect("Failed to setup test DB");
    let market = test_db
        .create_test_market_with_tokens("UNI", "USDC")
        .await
        .expect("Failed to create market");

    let engine = TestEngine::new(&test_db).await;

    // Place buy order at $5
    let buy = TestEngine::create_order(
        "buyer",
        &market.id,
        Side::Buy,
        OrderType::Limit,
        5_000_000, // $5
        10_000_000,
    );

    engine.place_order(buy).await.expect("Failed to place buy");

    // Place sell limit order at $8 (above buy price)
    let sell = TestEngine::create_order(
        "seller",
        &market.id,
        Side::Sell,
        OrderType::Limit,
        8_000_000, // $8 (won't match $5 bid)
        10_000_000,
    );

    let result = engine.place_order(sell).await;
    assert!(result.is_ok());
    let placed = result.unwrap();

    // Should not match
    assert_eq!(placed.trades.len(), 0);
    assert_eq!(placed.order.status, OrderStatus::Pending);
}

#[tokio::test]
async fn test_multiple_orders_complex_matching() {
    let test_db = TestDb::setup().await.expect("Failed to setup test DB");
    let market = test_db
        .create_test_market_with_tokens("ADA", "USDC")
        .await
        .expect("Failed to create market");

    let engine = TestEngine::new(&test_db).await;

    // Build an orderbook with multiple levels
    // Sell side:
    // $0.55 - 100 ADA (seller3)
    // $0.52 - 200 ADA (seller2)
    // $0.50 - 150 ADA (seller1)

    let sell1 = TestEngine::create_order(
        "seller1",
        &market.id,
        Side::Sell,
        OrderType::Limit,
        500_000, // $0.50
        150_000_000,
    );

    let sell2 = TestEngine::create_order(
        "seller2",
        &market.id,
        Side::Sell,
        OrderType::Limit,
        520_000, // $0.52
        200_000_000,
    );

    let sell3 = TestEngine::create_order(
        "seller3",
        &market.id,
        Side::Sell,
        OrderType::Limit,
        550_000, // $0.55
        100_000_000,
    );

    engine
        .place_order(sell1)
        .await
        .expect("Failed to place sell1");
    engine
        .place_order(sell2)
        .await
        .expect("Failed to place sell2");
    engine
        .place_order(sell3)
        .await
        .expect("Failed to place sell3");

    // Place large buy order that matches multiple levels
    let big_buy = TestEngine::create_order(
        "big_buyer",
        &market.id,
        Side::Buy,
        OrderType::Limit,
        600_000,     // Willing to pay $0.60
        400_000_000, // 400 ADA
    );

    let result = engine.place_order(big_buy).await;
    assert!(result.is_ok());
    let placed = result.unwrap();

    // Should match all three sell orders
    // First two fully, third one partially (total 400 ADA filled)
    assert_eq!(placed.trades.len(), 3);

    // Check prices are in correct order (best price first)
    assert_eq!(placed.trades[0].price, 500_000);
    assert_eq!(placed.trades[0].size, 150_000_000);
    assert_eq!(placed.trades[1].price, 520_000);
    assert_eq!(placed.trades[1].size, 200_000_000);
    assert_eq!(placed.trades[2].price, 550_000);
    assert_eq!(placed.trades[2].size, 50_000_000); // Only 50 ADA from third order

    // Buyer should be fully filled (400 ADA total)
    assert_eq!(placed.order.status, OrderStatus::Filled);
    assert_eq!(placed.order.filled_size, 400_000_000);
}
