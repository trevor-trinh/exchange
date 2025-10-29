/// Comprehensive SDK WebSocket tests
///
/// These tests verify real-time event streams using only the WebSocket API.

mod helpers;

use backend::models::domain::{OrderType, Side};
use exchange_sdk::{SubscriptionChannel, WebSocketClient};
use helpers::TestFixture;

// ============================================================================
// WebSocket Subscription Tests
// ============================================================================

#[tokio::test]
async fn test_websocket_trade_events() {
    let fixture = TestFixture::new()
        .await
        .expect("Failed to create test fixture");

    // Setup users
    fixture
        .create_user_with_balance("alice", 10_000_000, 0)
        .await
        .expect("Failed to create alice");
    fixture
        .create_user_with_balance("bob", 0, 1_000_000_000_000)
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
        if let Some(msg) = tokio::time::timeout(
            tokio::time::Duration::from_secs(1),
            ws_handle.recv(),
        )
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
        if let Some(msg) = tokio::time::timeout(
            tokio::time::Duration::from_millis(500),
            ws_handle.recv(),
        )
        .await
        .ok()
        .flatten()
        {
            if msg["type"] == "trade" {
                // Verify trade details
                assert_eq!(msg["channel"], "trades");
                assert_eq!(msg["data"]["market_id"], fixture.market_id);
                assert_eq!(msg["data"]["buyer_address"], "bob");
                assert_eq!(msg["data"]["seller_address"], "alice");
                assert_eq!(msg["data"]["price"], "50000000000");
                assert_eq!(msg["data"]["size"], "1000000");
                trade_received = true;
                break;
            }
        }
    }
    assert!(trade_received, "Failed to receive trade event");
}

#[tokio::test]
async fn test_websocket_orderbook_events() {
    let fixture = TestFixture::new()
        .await
        .expect("Failed to create test fixture");

    fixture
        .create_user_with_balance("trader", 10_000_000, 1_000_000_000_000)
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
        if let Some(msg) = tokio::time::timeout(
            tokio::time::Duration::from_millis(500),
            ws_handle.recv(),
        )
        .await
        .ok()
        .flatten()
        {
            if msg["type"] == "orderbook_snapshot" || msg["type"] == "orderbook_update" {
                assert_eq!(msg["channel"], "orderbook");
                assert_eq!(msg["data"]["market_id"], fixture.market_id);

                // Should have asks (sell orders)
                let asks = msg["data"]["asks"].as_array();
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
    let fixture = TestFixture::new()
        .await
        .expect("Failed to create test fixture");

    fixture
        .create_user_with_balance("alice", 10_000_000, 0)
        .await
        .expect("Failed to create alice");
    fixture
        .create_user_with_balance("bob", 0, 1_000_000_000_000)
        .await
        .expect("Failed to create bob");

    // Alice subscribes to her user updates
    let ws_client = WebSocketClient::new(&fixture.server.ws_url);
    let mut alice_ws = ws_client
        .connect()
        .await
        .expect("Failed to connect to WebSocket");

    alice_ws
        .subscribe(
            SubscriptionChannel::UserUpdates,
            None,
            Some("alice".to_string()),
        )
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

    // Alice should receive order placed event
    let mut order_placed_received = false;
    for _ in 0..20 {
        if let Some(msg) = tokio::time::timeout(
            tokio::time::Duration::from_millis(500),
            alice_ws.recv(),
        )
        .await
        .ok()
        .flatten()
        {
            if msg["type"] == "order_placed" {
                assert_eq!(msg["data"]["user_address"], "alice");
                assert_eq!(msg["data"]["market_id"], fixture.market_id);
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

    // Alice should receive order filled event
    let mut order_filled_received = false;
    for _ in 0..20 {
        if let Some(msg) = tokio::time::timeout(
            tokio::time::Duration::from_millis(500),
            alice_ws.recv(),
        )
        .await
        .ok()
        .flatten()
        {
            if msg["type"] == "order_filled" {
                assert_eq!(msg["data"]["user_address"], "alice");
                order_filled_received = true;
                break;
            }
        }
    }
    assert!(
        order_filled_received,
        "Alice did not receive order filled event"
    );
}

#[tokio::test]
async fn test_websocket_multiple_subscriptions() {
    let fixture = TestFixture::new()
        .await
        .expect("Failed to create test fixture");

    fixture
        .create_user_with_balance("trader", 10_000_000, 1_000_000_000_000)
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
        .subscribe(
            SubscriptionChannel::UserUpdates,
            None,
            Some("trader".to_string()),
        )
        .expect("Failed to subscribe to user updates");

    // Wait for subscription confirmations
    let mut subscriptions = 0;
    for _ in 0..30 {
        if let Some(msg) = tokio::time::timeout(
            tokio::time::Duration::from_millis(500),
            ws_handle.recv(),
        )
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
    assert_eq!(subscriptions, 3, "Did not receive all subscription confirmations");
}

#[tokio::test]
async fn test_websocket_unsubscribe() {
    let fixture = TestFixture::new()
        .await
        .expect("Failed to create test fixture");

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
        if let Some(msg) = tokio::time::timeout(
            tokio::time::Duration::from_millis(500),
            ws_handle.recv(),
        )
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
    let fixture = TestFixture::new()
        .await
        .expect("Failed to create test fixture");

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
        if let Some(msg) = tokio::time::timeout(
            tokio::time::Duration::from_secs(2),
            ws_handle.recv(),
        )
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
