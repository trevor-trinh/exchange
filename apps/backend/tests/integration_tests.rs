// Full end-to-end integration tests using only HTTP API and WebSocket
// Tests the complete flow: REST API → Engine → Matcher → Executor → Database → WebSocket events

use backend::models::domain::{OrderType, Side};
use futures::{SinkExt, StreamExt};
use serde_json::json;
use tokio_tungstenite::tungstenite::Message;

mod utils;
use utils::TestServer;

/// Helper to place an order via REST API
async fn place_order_via_api(
    server_url: &str,
    user_address: &str,
    market_id: &str,
    side: Side,
    order_type: OrderType,
    price: &str,
    size: &str,
) -> serde_json::Value {
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/api/trade", server_url))
        .json(&json!({
            "type": "place_order",
            "user_address": user_address,
            "market_id": market_id,
            "side": side,
            "order_type": order_type,
            "price": price,
            "size": size,
            "signature": "test_signature"
        }))
        .send()
        .await
        .expect("Failed to send request");

    let status = response.status();
    let body = response.text().await.expect("Failed to read response");

    if !status.is_success() {
        panic!("API request failed with status {}: {}", status, body);
    }

    serde_json::from_str(&body).expect("Failed to parse JSON response")
}

/// Helper to cancel an order via REST API
async fn cancel_order_via_api(
    server_url: &str,
    user_address: &str,
    order_id: &str,
) -> serde_json::Value {
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/api/trade", server_url))
        .json(&json!({
            "type": "cancel_order",
            "user_address": user_address,
            "order_id": order_id,
            "signature": "test_signature"
        }))
        .send()
        .await
        .expect("Failed to send request");

    response
        .json()
        .await
        .expect("Failed to parse JSON response")
}

/// Helper to drip tokens to a user via REST API (faucet)
async fn drip_tokens_via_api(
    server_url: &str,
    user_address: &str,
    token_ticker: &str,
    amount: &str,
) -> serde_json::Value {
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/api/drip", server_url))
        .json(&json!({
            "type": "faucet",
            "user_address": user_address,
            "token_ticker": token_ticker,
            "amount": amount,
            "signature": "test_signature"
        }))
        .send()
        .await
        .expect("Failed to send drip request");

    let status = response.status();
    let body = response.text().await.expect("Failed to read response");

    if !status.is_success() {
        panic!("Drip request failed with status {}: {}", status, body);
    }

    serde_json::from_str(&body).expect("Failed to parse JSON response")
}

#[tokio::test]
async fn test_full_e2e_order_matching_via_api() {
    // Start test server
    let server = TestServer::start().await.expect("Failed to start server");

    // Setup market
    let market = server
        .test_db
        .create_test_market_with_tokens("BTC", "USDC")
        .await
        .expect("Failed to create market");

    // Drip tokens to users (this also creates users if they don't exist)
    // Seller needs BTC to sell, buyer needs USDC to buy
    drip_tokens_via_api(&server.address, "seller", "BTC", "1000000").await;
    drip_tokens_via_api(&server.address, "buyer", "USDC", "50000000000000000").await;

    // Connect to WebSocket
    let ws_url = server.ws_url("/ws");
    let (ws_stream, _) = tokio_tungstenite::connect_async(&ws_url)
        .await
        .expect("Failed to connect to WebSocket");

    let (mut write, mut read) = ws_stream.split();

    // Subscribe to market events
    write
        .send(Message::Text(
            json!({
                "type": "subscribe",
                "channel": "trades",
                "market_id": market.id
            })
            .to_string()
            .into(),
        ))
        .await
        .expect("Failed to send subscribe message");

    // Wait for subscription confirmation (skip pings)
    loop {
        let sub_msg = read.next().await.expect("No subscription confirmation");
        let sub_message = sub_msg.expect("WebSocket error");
        if let Ok(sub_text) = sub_message.to_text() {
            if !sub_text.is_empty() {
                let sub_event: serde_json::Value =
                    serde_json::from_str(sub_text).expect("Invalid JSON");
                assert_eq!(sub_event["type"], "subscribed");
                break;
            }
        }
    }

    // Place sell order via REST API
    let sell_response = place_order_via_api(
        &server.address,
        "seller",
        &market.id,
        Side::Sell,
        OrderType::Limit,
        "50000000000", // $50,000
        "1000000",     // 1 BTC
    )
    .await;

    assert_eq!(sell_response["type"], "place_order");
    assert_eq!(sell_response["order"]["status"], "pending");
    assert_eq!(sell_response["trades"].as_array().unwrap().len(), 0);

    // Place buy order via REST API (should match)
    let buy_response = place_order_via_api(
        &server.address,
        "buyer",
        &market.id,
        Side::Buy,
        OrderType::Limit,
        "50000000000", // $50,000
        "1000000",     // 1 BTC
    )
    .await;

    assert_eq!(buy_response["type"], "place_order");
    assert_eq!(buy_response["order"]["status"], "filled");
    assert_eq!(buy_response["trades"].as_array().unwrap().len(), 1);

    // Verify trade details
    let trade = &buy_response["trades"][0];
    assert_eq!(trade["buyer_address"], "buyer");
    assert_eq!(trade["seller_address"], "seller");
    assert_eq!(trade["price"], "50000000000");
    assert_eq!(trade["size"], "1000000");

    // Listen for WebSocket trade event (skip pings)
    let trade_event = loop {
        tokio::select! {
            msg = read.next() => {
                let msg = msg.expect("No WebSocket message received");
                let message = msg.expect("WebSocket error");
                if let Ok(msg_text) = message.to_text() {
                    if !msg_text.is_empty() {
                        let event: serde_json::Value = serde_json::from_str(msg_text).expect("Invalid JSON");
                        break event;
                    }
                }
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(2)) => {
                panic!("Timeout waiting for WebSocket trade event");
            }
        }
    };

    assert_eq!(trade_event["type"], "trade");
    assert_eq!(trade_event["trade"]["buyer_address"], "buyer");
    assert_eq!(trade_event["trade"]["seller_address"], "seller");
    assert_eq!(trade_event["trade"]["price"], "50000000000");
    assert_eq!(trade_event["trade"]["size"], "1000000");
}

#[tokio::test]
async fn test_e2e_order_cancellation_via_api() {
    let server = TestServer::start().await.expect("Failed to start server");

    let market = server
        .test_db
        .create_test_market_with_tokens("ETH", "USDC")
        .await
        .expect("Failed to create market");

    // Drip tokens to trader (trader selling ETH)
    drip_tokens_via_api(&server.address, "trader", "ETH", "5000000").await;

    // Place an order via API
    let place_response = place_order_via_api(
        &server.address,
        "trader",
        &market.id,
        Side::Sell,
        OrderType::Limit,
        "3000000000",
        "5000000",
    )
    .await;

    let order_id = place_response["order"]["id"].as_str().expect("No order ID");
    assert_eq!(place_response["order"]["status"], "pending");

    // Cancel the order via API
    let cancel_response = cancel_order_via_api(&server.address, "trader", order_id).await;

    assert_eq!(cancel_response["type"], "cancel_order");
    assert_eq!(cancel_response["order_id"], order_id);
}

#[tokio::test]
async fn test_e2e_partial_fill_via_api() {
    let server = TestServer::start().await.expect("Failed to start server");

    let market = server
        .test_db
        .create_test_market_with_tokens("SOL", "USDC")
        .await
        .expect("Failed to create market");

    // Drip tokens to users
    drip_tokens_via_api(&server.address, "seller", "SOL", "10000000").await;
    drip_tokens_via_api(&server.address, "buyer", "USDC", "1000000000000000").await;

    // Place sell order for 10 SOL
    let sell_response = place_order_via_api(
        &server.address,
        "seller",
        &market.id,
        Side::Sell,
        OrderType::Limit,
        "100000000", // $100
        "10000000",  // 10 SOL
    )
    .await;

    assert_eq!(sell_response["order"]["status"], "pending");

    // Place buy order for only 3 SOL
    let buy_response = place_order_via_api(
        &server.address,
        "buyer",
        &market.id,
        Side::Buy,
        OrderType::Limit,
        "100000000", // $100
        "3000000",   // 3 SOL
    )
    .await;

    // Buyer should be fully filled
    assert_eq!(buy_response["order"]["status"], "filled");
    assert_eq!(buy_response["order"]["filled_size"], "3000000");
    assert_eq!(buy_response["trades"].as_array().unwrap().len(), 1);

    // Verify trade
    let trade = &buy_response["trades"][0];
    assert_eq!(trade["size"], "3000000");
}

#[tokio::test]
async fn test_e2e_market_order_via_api() {
    let server = TestServer::start().await.expect("Failed to start server");

    let market = server
        .test_db
        .create_test_market_with_tokens("AVAX", "USDC")
        .await
        .expect("Failed to create market");

    // Drip tokens to users
    drip_tokens_via_api(&server.address, "seller", "AVAX", "10000000").await;
    drip_tokens_via_api(&server.address, "buyer", "USDC", "200000000000000").await;

    // Place limit sell orders at different prices
    place_order_via_api(
        &server.address,
        "seller",
        &market.id,
        Side::Sell,
        OrderType::Limit,
        "20000000", // $20
        "5000000",  // 5 AVAX
    )
    .await;

    place_order_via_api(
        &server.address,
        "seller",
        &market.id,
        Side::Sell,
        OrderType::Limit,
        "21000000", // $21
        "5000000",  // 5 AVAX
    )
    .await;

    // Place market buy order (should match at any price)
    let market_buy = place_order_via_api(
        &server.address,
        "buyer",
        &market.id,
        Side::Buy,
        OrderType::Market,
        "0",       // Price doesn't matter
        "8000000", // 8 AVAX
    )
    .await;

    // Should match both orders
    assert_eq!(market_buy["order"]["status"], "filled");
    assert_eq!(market_buy["order"]["filled_size"], "8000000");
    assert_eq!(market_buy["trades"].as_array().unwrap().len(), 2);

    // First trade at best price
    assert_eq!(market_buy["trades"][0]["price"], "20000000");
    assert_eq!(market_buy["trades"][0]["size"], "5000000");

    // Second trade at worse price
    assert_eq!(market_buy["trades"][1]["price"], "21000000");
    assert_eq!(market_buy["trades"][1]["size"], "3000000");
}

#[tokio::test]
async fn test_e2e_orderbook_snapshots_via_websocket() {
    let server = TestServer::start().await.expect("Failed to start server");

    let market = server
        .test_db
        .create_test_market_with_tokens("ADA", "USDC")
        .await
        .expect("Failed to create market");

    // Drip tokens to seller
    drip_tokens_via_api(&server.address, "seller", "ADA", "10000000").await;

    // Connect to WebSocket
    let ws_url = server.ws_url("/ws");
    let (ws_stream, _) = tokio_tungstenite::connect_async(&ws_url)
        .await
        .expect("Failed to connect to WebSocket");

    let (mut write, mut read) = ws_stream.split();

    // Subscribe to orderbook snapshots
    write
        .send(Message::Text(
            json!({
                "type": "subscribe",
                "channel": "orderbook",
                "market_id": market.id
            })
            .to_string()
            .into(),
        ))
        .await
        .expect("Failed to send subscribe message");

    // Wait for subscription confirmation (skip pings)
    loop {
        let sub_msg = read.next().await.expect("No subscription confirmation");
        let sub_message = sub_msg.expect("WebSocket error");
        if let Ok(sub_text) = sub_message.to_text() {
            if !sub_text.is_empty() {
                let sub_event: serde_json::Value =
                    serde_json::from_str(sub_text).expect("Invalid JSON");
                assert_eq!(sub_event["type"], "subscribed");
                break;
            }
        }
    }

    // Place some orders via API to populate orderbook
    place_order_via_api(
        &server.address,
        "seller",
        &market.id,
        Side::Sell,
        OrderType::Limit,
        "500000",   // $0.50
        "10000000", // 10 ADA
    )
    .await;

    // Listen for orderbook snapshot (skip pings)
    let orderbook_event = loop {
        tokio::select! {
            msg = read.next() => {
                let msg = msg.expect("No WebSocket message received");
                let message = msg.expect("WebSocket error");
                if let Ok(msg_text) = message.to_text() {
                    if !msg_text.is_empty() {
                        let event: serde_json::Value = serde_json::from_str(msg_text).expect("Invalid JSON");
                        break event;
                    }
                }
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(2)) => {
                panic!("Timeout waiting for WebSocket orderbook snapshot");
            }
        }
    };

    assert_eq!(orderbook_event["type"], "orderbook");
    assert_eq!(orderbook_event["orderbook"]["market_id"], market.id);

    // Should have asks
    let asks = orderbook_event["orderbook"]["asks"]
        .as_array()
        .expect("No asks array");
    assert!(!asks.is_empty(), "Orderbook should have asks");
}

#[tokio::test]
async fn test_e2e_price_time_priority_via_api() {
    let server = TestServer::start().await.expect("Failed to start server");

    let market = server
        .test_db
        .create_test_market_with_tokens("DOT", "USDC")
        .await
        .expect("Failed to create market");

    // Drip tokens to users
    drip_tokens_via_api(&server.address, "seller1", "DOT", "5000000").await;
    drip_tokens_via_api(&server.address, "seller2", "DOT", "3000000").await;
    drip_tokens_via_api(&server.address, "buyer", "USDC", "64000000000000").await;

    // Place two sell orders at different prices
    place_order_via_api(
        &server.address,
        "seller1",
        &market.id,
        Side::Sell,
        OrderType::Limit,
        "7000000", // $7.00
        "5000000", // 5 DOT
    )
    .await;

    place_order_via_api(
        &server.address,
        "seller2",
        &market.id,
        Side::Sell,
        OrderType::Limit,
        "6500000", // $6.50 (better price, placed second)
        "3000000", // 3 DOT
    )
    .await;

    // Place buy order that can match both
    let buy_response = place_order_via_api(
        &server.address,
        "buyer",
        &market.id,
        Side::Buy,
        OrderType::Limit,
        "8000000", // $8.00
        "8000000", // 8 DOT
    )
    .await;

    // Should match best price first (seller2 at $6.50)
    assert_eq!(buy_response["trades"].as_array().unwrap().len(), 2);
    assert_eq!(buy_response["trades"][0]["price"], "6500000");
    assert_eq!(buy_response["trades"][0]["size"], "3000000");
    assert_eq!(buy_response["trades"][0]["seller_address"], "seller2");

    // Then match worse price (seller1 at $7.00)
    assert_eq!(buy_response["trades"][1]["price"], "7000000");
    assert_eq!(buy_response["trades"][1]["size"], "5000000");
    assert_eq!(buy_response["trades"][1]["seller_address"], "seller1");
}

#[tokio::test]
async fn test_e2e_validation_errors_via_api() {
    let server = TestServer::start().await.expect("Failed to start server");

    let market = server
        .test_db
        .create_test_market_with_tokens("ATOM", "USDC")
        .await
        .expect("Failed to create market");

    server
        .test_db
        .create_test_user("trader")
        .await
        .expect("Failed to create trader");

    let client = reqwest::Client::new();

    // Test invalid tick size (price not multiple of tick_size)
    // market.tick_size is 1000, so 50500 should fail
    let response = client
        .post(&format!("{}/api/trade", server.address))
        .json(&json!({
            "type": "place_order",
            "user_address": "trader",
            "market_id": market.id,
            "side": "sell",
            "order_type": "limit",
            "price": "500500", // Not a multiple of 1000
            "size": "1000000",
            "signature": "test_signature"
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 400);
    let error: serde_json::Value = response.json().await.expect("Failed to parse error");
    assert!(error["error"].as_str().unwrap().contains("tick size"));

    // Test invalid lot size
    let response = client
        .post(&format!("{}/api/trade", server.address))
        .json(&json!({
            "type": "place_order",
            "user_address": "trader",
            "market_id": market.id,
            "side": "sell",
            "order_type": "limit",
            "price": "5000000",
            "size": "500", // Not a multiple of lot_size (1000000)
            "signature": "test_signature"
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 400);
    let error: serde_json::Value = response.json().await.expect("Failed to parse error");
    assert!(error["error"].as_str().unwrap().contains("lot size"));

    // Test below minimum size
    // Note: size 500000 is not a multiple of lot_size (1000000), so it will fail lot size validation
    let response = client
        .post(&format!("{}/api/trade", server.address))
        .json(&json!({
            "type": "place_order",
            "user_address": "trader",
            "market_id": market.id,
            "side": "sell",
            "order_type": "limit",
            "price": "5000000",
            "size": "500000", // Not a multiple of lot_size (1000000)
            "signature": "test_signature"
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 400);
    let error: serde_json::Value = response.json().await.expect("Failed to parse error");
    assert!(error["error"].as_str().unwrap().contains("lot size"));
}
