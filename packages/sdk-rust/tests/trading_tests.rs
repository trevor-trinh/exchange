/// Comprehensive SDK trading workflow tests
///
/// These tests verify the SDK through realistic trading scenarios,
/// using ONLY the public REST and WebSocket APIs (no direct DB access for verification).

mod helpers;

use backend::models::domain::{OrderType, Side};
use exchange_sdk::{ExchangeClient, SubscriptionChannel, WebSocketClient};
use helpers::TestFixture;

// ============================================================================
// Basic Trading Workflows
// ============================================================================

#[tokio::test]
async fn test_complete_trading_workflow() {
    let fixture = TestFixture::new()
        .await
        .expect("Failed to create test fixture");

    // Setup: Create two users with balances
    fixture
        .create_user_with_balance("alice", 10_000_000, 0) // 10 BTC
        .await
        .expect("Failed to create alice");
    fixture
        .create_user_with_balance("bob", 0, 1_000_000_000_000) // 1M USDC
        .await
        .expect("Failed to create bob");

    // 1. Alice checks her balance
    let balances = fixture
        .client
        .get_balances("alice")
        .await
        .expect("Failed to get alice's balances");
    assert_eq!(balances.len(), 1);
    assert_eq!(balances[0].token_ticker, "BTC");
    assert_eq!(balances[0].amount, 10_000_000);

    // 2. Alice places a sell order (maker)
    let alice_order = fixture
        .client
        .place_order(
            "alice".to_string(),
            fixture.market_id.clone(),
            Side::Sell,
            OrderType::Limit,
            "50000000000".to_string(), // $50,000
            "1000000".to_string(),      // 1 BTC
            "test_sig".to_string(),
        )
        .await
        .expect("Failed to place alice's order");

    assert_eq!(alice_order.order.status, backend::models::domain::OrderStatus::Pending);
    assert_eq!(alice_order.trades.len(), 0); // No match yet

    // 3. Alice's order is now pending
    // (Note: Balance details like locked/available aren't exposed in the API)

    // 4. Alice can see her open order
    let orders = fixture
        .client
        .get_orders("alice", Some(fixture.market_id.clone()))
        .await
        .expect("Failed to get alice's orders");
    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0].id, alice_order.order.id);

    // 5. Bob places a buy order that matches (taker)
    let bob_order = fixture
        .client
        .place_order(
            "bob".to_string(),
            fixture.market_id.clone(),
            Side::Buy,
            OrderType::Limit,
            "50000000000".to_string(), // Willing to pay $50,000
            "1000000".to_string(),      // 1 BTC
            "test_sig".to_string(),
        )
        .await
        .expect("Failed to place bob's order");

    assert_eq!(bob_order.order.status, backend::models::domain::OrderStatus::Filled);
    assert_eq!(bob_order.trades.len(), 1); // Matched!

    // 6. Verify trade details
    let trade = &bob_order.trades[0];
    assert_eq!(trade.buyer_address, "bob");
    assert_eq!(trade.seller_address, "alice");
    assert_eq!(trade.price, 50_000_000_000);
    assert_eq!(trade.size, 1_000_000);

    // 7. Check Alice received USDC (minus fees)
    let balances = fixture
        .client
        .get_balances("alice")
        .await
        .expect("Failed to get alice's balances");
    let usdc_balance = balances.iter().find(|b| b.token_ticker == "USDC");
    assert!(usdc_balance.is_some());
    let usdc = usdc_balance.unwrap();
    // Alice gets: 50000 USDC - 0.1% maker fee
    assert!(usdc.amount > 49_990_000_000 && usdc.amount < 50_000_000_000);

    // 8. Check Bob received BTC (minus fees)
    let balances = fixture
        .client
        .get_balances("bob")
        .await
        .expect("Failed to get bob's balances");
    let btc_balance = balances.iter().find(|b| b.token_ticker == "BTC");
    assert!(btc_balance.is_some());
    let btc = btc_balance.unwrap();
    // Bob gets: 1 BTC - 0.2% taker fee = 0.998 BTC
    assert!(btc.amount > 998_000 && btc.amount < 1_000_000);

    // 9. Both users can see their trade history
    let alice_trades = fixture
        .client
        .get_trades("alice", Some(fixture.market_id.clone()))
        .await
        .expect("Failed to get alice's trades");
    assert_eq!(alice_trades.len(), 1);

    let bob_trades = fixture
        .client
        .get_trades("bob", Some(fixture.market_id.clone()))
        .await
        .expect("Failed to get bob's trades");
    assert_eq!(bob_trades.len(), 1);
}

#[tokio::test]
async fn test_partial_fill() {
    let fixture = TestFixture::new()
        .await
        .expect("Failed to create test fixture");

    fixture
        .create_user_with_balance("seller", 10_000_000, 0)
        .await
        .expect("Failed to create seller");
    fixture
        .create_user_with_balance("buyer", 0, 1_000_000_000_000)
        .await
        .expect("Failed to create buyer");

    // Seller places order for 10 BTC
    let sell_order = fixture
        .client
        .place_order(
            "seller".to_string(),
            fixture.market_id.clone(),
            Side::Sell,
            OrderType::Limit,
            "50000000000".to_string(),
            "10000000".to_string(), // 10 BTC
            "test_sig".to_string(),
        )
        .await
        .expect("Failed to place sell order");

    assert_eq!(sell_order.order.status, backend::models::domain::OrderStatus::Pending);

    // Buyer only wants 3 BTC
    let buy_order = fixture
        .client
        .place_order(
            "buyer".to_string(),
            fixture.market_id.clone(),
            Side::Buy,
            OrderType::Limit,
            "50000000000".to_string(),
            "3000000".to_string(), // 3 BTC only
            "test_sig".to_string(),
        )
        .await
        .expect("Failed to place buy order");

    // Buyer order should be fully filled
    assert_eq!(buy_order.order.status, backend::models::domain::OrderStatus::Filled);
    assert_eq!(buy_order.order.filled_size, 3_000_000);

    // Seller should have a partially filled order remaining
    let seller_orders = fixture
        .client
        .get_orders("seller", Some(fixture.market_id.clone()))
        .await
        .expect("Failed to get seller orders");

    assert_eq!(seller_orders.len(), 1);
    let remaining_order = &seller_orders[0];
    assert_eq!(remaining_order.filled_size, 3_000_000); // 3 filled
    assert_eq!(remaining_order.size, 10_000_000); // Original 10
    assert_eq!(remaining_order.status, backend::models::domain::OrderStatus::PartiallyFilled);
}

#[tokio::test]
async fn test_order_cancellation() {
    let fixture = TestFixture::new()
        .await
        .expect("Failed to create test fixture");

    fixture
        .create_user_with_balance("trader", 10_000_000, 0)
        .await
        .expect("Failed to create trader");

    // Place an order
    let order = fixture
        .client
        .place_order(
            "trader".to_string(),
            fixture.market_id.clone(),
            Side::Sell,
            OrderType::Limit,
            "50000000000".to_string(),
            "1000000".to_string(),
            "test_sig".to_string(),
        )
        .await
        .expect("Failed to place order");

    let order_id = order.order.id.to_string();

    // Cancel the order
    let cancelled = fixture
        .client
        .cancel_order(
            "trader".to_string(),
            order_id.clone(),
            "test_sig".to_string(),
        )
        .await
        .expect("Failed to cancel order");

    assert_eq!(cancelled.order_id, order_id);

    // Order should no longer appear in active orders
    let orders = fixture
        .client
        .get_orders("trader", Some(fixture.market_id.clone()))
        .await
        .expect("Failed to get orders");

    // Should be empty or only contain non-pending orders
    let pending_orders: Vec<_> = orders
        .iter()
        .filter(|o| o.status == backend::models::domain::OrderStatus::Pending)
        .collect();
    assert_eq!(pending_orders.len(), 0);
}

#[tokio::test]
async fn test_market_info_endpoints() {
    let fixture = TestFixture::new()
        .await
        .expect("Failed to create test fixture");

    // Get all markets
    let markets = fixture
        .client
        .get_markets()
        .await
        .expect("Failed to get markets");
    assert_eq!(markets.len(), 1);
    assert_eq!(markets[0].id, fixture.market_id);

    // Get specific market
    let market = fixture
        .client
        .get_market(&fixture.market_id)
        .await
        .expect("Failed to get market");
    assert_eq!(market.id, fixture.market_id);
    assert_eq!(market.base_ticker, "BTC");
    assert_eq!(market.quote_ticker, "USDC");

    // Get all tokens
    let tokens = fixture
        .client
        .get_tokens()
        .await
        .expect("Failed to get tokens");
    assert_eq!(tokens.len(), 2);

    let tickers: Vec<_> = tokens.iter().map(|t| t.ticker.as_str()).collect();
    assert!(tickers.contains(&"BTC"));
    assert!(tickers.contains(&"USDC"));

    // Get specific token
    let btc = fixture
        .client
        .get_token("BTC")
        .await
        .expect("Failed to get BTC token");
    assert_eq!(btc.ticker, "BTC");
    assert_eq!(btc.decimals, 18);
}

#[tokio::test]
async fn test_multiple_concurrent_orders() {
    let fixture = TestFixture::new()
        .await
        .expect("Failed to create test fixture");

    fixture
        .create_user_with_balance("trader", 10_000_000, 0)
        .await
        .expect("Failed to create trader");

    // Place multiple orders concurrently
    let mut handles = vec![];

    for i in 0..5 {
        let client = fixture.client.clone();
        let market_id = fixture.market_id.clone();

        let handle = tokio::spawn(async move {
            client
                .place_order(
                    "trader".to_string(),
                    market_id,
                    Side::Sell,
                    OrderType::Limit,
                    format!("{}", 50_000_000_000 + (i * 1_000_000_000)),
                    "500000".to_string(), // 0.5 BTC each
                    "test_sig".to_string(),
                )
                .await
        });

        handles.push(handle);
    }

    // Wait for all orders
    let mut successful_orders = 0;
    for handle in handles {
        if handle.await.is_ok() {
            successful_orders += 1;
        }
    }

    // All orders should succeed
    assert_eq!(successful_orders, 5);

    // Verify all orders are tracked
    let orders = fixture
        .client
        .get_orders("trader", Some(fixture.market_id.clone()))
        .await
        .expect("Failed to get orders");

    assert!(orders.len() >= 5);
}
