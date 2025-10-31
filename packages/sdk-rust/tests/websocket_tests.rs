/// Comprehensive SDK WebSocket tests
///
/// These tests verify real-time event streams using only the WebSocket API.
use backend::models::domain::{OrderType, Side};
use exchange_sdk::{SubscriptionChannel, WebSocketClient};
use exchange_test_utils::TestExchange;

// ============================================================================
// WebSocket Subscription Tests
// ============================================================================

#[tokio::test]
async fn test_websocket_trade_events() {
    let fixture = TestExchange::new()
        .await
        .expect("Failed to create test exchange");

    // Setup users
    fixture
        .create_user_with_balance("alice", 10_000_000, 0)
        .await
        .expect("Failed to create alice");
    fixture
        .create_user_with_balance("bob", 0, 100_000_000_000_000_000) // 100M USDC (enough for trade + fees)
        .await
        .expect("Failed to create bob");

    // Connect WebSocket and subscribe to trades
    let ws_client = WebSocketClient::new(&fixture.server.ws_url);
    let mut ws_handle = ws_client
        .connect()
        .await
        .expect("Failed to connect to WebSocket");

    ws_handle
        .subscribe(
            SubscriptionChannel::Trades,
            Some(fixture.market_id.clone()),
            None,
        )
        .expect("Failed to subscribe to trades");

    // Wait for subscription confirmation
    let mut subscribed = false;
    for _ in 0..10 {
        if let Some(msg) =
            tokio::time::timeout(tokio::time::Duration::from_secs(1), ws_handle.recv())
                .await
                .ok()
                .flatten()
        {
            if msg["type"] == "subscribed" {
                subscribed = true;
                break;
            }
        }
    }
    assert!(subscribed, "Failed to receive subscription confirmation");

    // Place orders that will match
    fixture
        .client
        .place_order(
            "alice".to_string(),
            fixture.market_id.clone(),
            Side::Sell,
            OrderType::Limit,
            "50000000000".to_string(),
            "1000000".to_string(),
            "test_sig".to_string(),
        )
        .await
        .expect("Failed to place alice's order");

    fixture
        .client
        .place_order(
            "bob".to_string(),
            fixture.market_id.clone(),
            Side::Buy,
            OrderType::Limit,
            "50000000000".to_string(),
            "1000000".to_string(),
            "test_sig".to_string(),
        )
        .await
        .expect("Failed to place bob's order");

    // Receive trade event
    let mut trade_received = false;
    for _ in 0..20 {
        if let Some(msg) =
            tokio::time::timeout(tokio::time::Duration::from_millis(500), ws_handle.recv())
                .await
                .ok()
                .flatten()
        {
            if msg["type"] == "trade" {
                // Verify trade details
                assert_eq!(msg["trade"]["market_id"], fixture.market_id);
                assert_eq!(msg["trade"]["buyer_address"], "bob");
                assert_eq!(msg["trade"]["seller_address"], "alice");
                assert_eq!(msg["trade"]["price"], "50000000000");
                assert_eq!(msg["trade"]["size"], "1000000");
                trade_received = true;
                break;
            }
        }
    }
    assert!(trade_received, "Failed to receive trade event");
}

#[tokio::test]
async fn test_websocket_orderbook_events() {
    let fixture = TestExchange::new()
        .await
        .expect("Failed to create test exchange");

    fixture
        .create_user_with_balance("trader", 10_000_000, 100_000_000_000_000_000) // 10 BTC + 100M USDC
        .await
        .expect("Failed to create trader");

    // Connect and subscribe to orderbook
    let ws_client = WebSocketClient::new(&fixture.server.ws_url);
    let mut ws_handle = ws_client
        .connect()
        .await
        .expect("Failed to connect to WebSocket");

    ws_handle
        .subscribe(
            SubscriptionChannel::Orderbook,
            Some(fixture.market_id.clone()),
            None,
        )
        .expect("Failed to subscribe to orderbook");

    // Wait for subscription confirmation
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Place an order
    fixture
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

    // Receive orderbook event
    let mut orderbook_received = false;
    for _ in 0..20 {
        if let Some(msg) =
            tokio::time::timeout(tokio::time::Duration::from_millis(500), ws_handle.recv())
                .await
                .ok()
                .flatten()
        {
            if msg["type"] == "orderbook" {
                assert_eq!(msg["orderbook"]["market_id"], fixture.market_id);

                // Should have asks (sell orders)
                let asks = msg["orderbook"]["asks"].as_array();
                assert!(asks.is_some());
                orderbook_received = true;
                break;
            }
        }
    }
    assert!(orderbook_received, "Failed to receive orderbook event");
}

#[tokio::test]
async fn test_websocket_user_events() {
    let fixture = TestExchange::new()
        .await
        .expect("Failed to create test exchange");

    fixture
        .create_user_with_balance("alice", 10_000_000, 0)
        .await
        .expect("Failed to create alice");
    fixture
        .create_user_with_balance("bob", 0, 100_000_000_000_000_000) // 100M USDC (enough for trade + fees)
        .await
        .expect("Failed to create bob");

    // Alice subscribes to her user updates
    let ws_client = WebSocketClient::new(&fixture.server.ws_url);
    let mut alice_ws = ws_client
        .connect()
        .await
        .expect("Failed to connect to WebSocket");

    alice_ws
        .subscribe(SubscriptionChannel::User, None, Some("alice".to_string()))
        .expect("Failed to subscribe to user updates");

    // Wait for subscription confirmation
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Alice places an order
    fixture
        .client
        .place_order(
            "alice".to_string(),
            fixture.market_id.clone(),
            Side::Sell,
            OrderType::Limit,
            "50000000000".to_string(),
            "1000000".to_string(),
            "test_sig".to_string(),
        )
        .await
        .expect("Failed to place alice's order");

    // Alice should receive order update event (order placed means status="pending")
    let mut order_placed_received = false;
    for _ in 0..20 {
        if let Some(msg) =
            tokio::time::timeout(tokio::time::Duration::from_millis(500), alice_ws.recv())
                .await
                .ok()
                .flatten()
        {
            if msg["type"] == "order" && msg["status"] == "pending" {
                order_placed_received = true;
                break;
            }
        }
    }
    assert!(
        order_placed_received,
        "Alice did not receive order placed event"
    );

    // Bob matches the order
    fixture
        .client
        .place_order(
            "bob".to_string(),
            fixture.market_id.clone(),
            Side::Buy,
            OrderType::Limit,
            "50000000000".to_string(),
            "1000000".to_string(),
            "test_sig".to_string(),
        )
        .await
        .expect("Failed to place bob's order");

    // Alice should receive trade executed event (which indicates order is filled)
    let mut trade_event_received = false;
    for _ in 0..20 {
        if let Some(msg) =
            tokio::time::timeout(tokio::time::Duration::from_millis(500), alice_ws.recv())
                .await
                .ok()
                .flatten()
        {
            // Trade events indicate an order was filled
            if msg["type"] == "trade" {
                // Verify it's Alice's trade
                if msg["trade"]["seller_address"] == "alice"
                    || msg["trade"]["buyer_address"] == "alice"
                {
                    trade_event_received = true;
                    break;
                }
            }
        }
    }
    assert!(
        trade_event_received,
        "Alice did not receive trade executed event"
    );
}

#[tokio::test]
async fn test_websocket_multiple_subscriptions() {
    let fixture = TestExchange::new()
        .await
        .expect("Failed to create test exchange");

    fixture
        .create_user_with_balance("trader", 10_000_000, 100_000_000_000_000_000) // 10 BTC + 100M USDC
        .await
        .expect("Failed to create trader");

    // Connect and subscribe to multiple channels
    let ws_client = WebSocketClient::new(&fixture.server.ws_url);
    let mut ws_handle = ws_client
        .connect()
        .await
        .expect("Failed to connect to WebSocket");

    ws_handle
        .subscribe(
            SubscriptionChannel::Trades,
            Some(fixture.market_id.clone()),
            None,
        )
        .expect("Failed to subscribe to trades");

    ws_handle
        .subscribe(
            SubscriptionChannel::Orderbook,
            Some(fixture.market_id.clone()),
            None,
        )
        .expect("Failed to subscribe to orderbook");

    ws_handle
        .subscribe(SubscriptionChannel::User, None, Some("trader".to_string()))
        .expect("Failed to subscribe to user updates");

    // Wait for subscription confirmations
    let mut subscriptions = 0;
    for _ in 0..30 {
        if let Some(msg) =
            tokio::time::timeout(tokio::time::Duration::from_millis(500), ws_handle.recv())
                .await
                .ok()
                .flatten()
        {
            if msg["type"] == "subscribed" {
                subscriptions += 1;
                if subscriptions == 3 {
                    break;
                }
            }
        }
    }
    assert_eq!(
        subscriptions, 3,
        "Did not receive all subscription confirmations"
    );
}

#[tokio::test]
async fn test_websocket_unsubscribe() {
    let fixture = TestExchange::new()
        .await
        .expect("Failed to create test exchange");

    let ws_client = WebSocketClient::new(&fixture.server.ws_url);
    let mut ws_handle = ws_client
        .connect()
        .await
        .expect("Failed to connect to WebSocket");

    // Subscribe
    ws_handle
        .subscribe(
            SubscriptionChannel::Trades,
            Some(fixture.market_id.clone()),
            None,
        )
        .expect("Failed to subscribe");

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Unsubscribe
    ws_handle
        .unsubscribe(
            SubscriptionChannel::Trades,
            Some(fixture.market_id.clone()),
            None,
        )
        .expect("Failed to unsubscribe");

    // Wait for unsubscribe confirmation
    let mut unsubscribed = false;
    for _ in 0..10 {
        if let Some(msg) =
            tokio::time::timeout(tokio::time::Duration::from_millis(500), ws_handle.recv())
                .await
                .ok()
                .flatten()
        {
            if msg["type"] == "unsubscribed" {
                unsubscribed = true;
                break;
            }
        }
    }
    assert!(unsubscribed, "Did not receive unsubscribe confirmation");
}

#[tokio::test]
async fn test_websocket_ping_pong() {
    let fixture = TestExchange::new()
        .await
        .expect("Failed to create test exchange");

    let ws_client = WebSocketClient::new(&fixture.server.ws_url);
    let mut ws_handle = ws_client
        .connect()
        .await
        .expect("Failed to connect to WebSocket");

    // Send ping
    ws_handle.ping().expect("Failed to send ping");

    // Wait for pong
    let mut pong_received = false;
    for _ in 0..10 {
        if let Some(msg) =
            tokio::time::timeout(tokio::time::Duration::from_secs(2), ws_handle.recv())
                .await
                .ok()
                .flatten()
        {
            if msg["type"] == "pong" {
                pong_received = true;
                break;
            }
        }
    }
    assert!(pong_received, "Did not receive pong response");
}
