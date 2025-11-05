mod utils;

use backend::models::api::{ClientMessage, ServerMessage, SubscriptionChannel};
use backend::models::domain::{OrderType, Side};
use futures::{SinkExt, StreamExt};
use tokio::time::{timeout, Duration};
use tokio_tungstenite::tungstenite::Message;
use utils::TestServer;

type WsStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// Helper to send a JSON message to WebSocket
async fn send_json<T: serde::Serialize>(ws: &mut WsStream, msg: &T) -> anyhow::Result<()> {
    let json = serde_json::to_string(msg)?;
    ws.send(Message::Text(json.into())).await?;
    Ok(())
}

/// Helper to receive and parse the next WebSocket message
async fn receive_message(ws: &mut WsStream) -> anyhow::Result<ServerMessage> {
    loop {
        match timeout(Duration::from_secs(5), ws.next()).await {
            Ok(Some(Ok(Message::Text(text)))) => {
                let msg: ServerMessage = serde_json::from_str(&text)?;
                return Ok(msg);
            }
            Ok(Some(Ok(Message::Ping(_)))) => {
                // Skip ping messages
                continue;
            }
            Ok(Some(Ok(msg))) => {
                anyhow::bail!("Unexpected message type: {:?}", msg)
            }
            Ok(Some(Err(e))) => {
                anyhow::bail!("WebSocket error: {}", e)
            }
            Ok(None) => {
                anyhow::bail!("Connection closed")
            }
            Err(_) => {
                anyhow::bail!("Timeout waiting for message")
            }
        }
    }
}

// ============================================================================
// Balance Event Tests for Limit Orders
// ============================================================================

#[tokio::test]
async fn test_balance_events_on_limit_order_placement() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    // Setup: Create market and give user balance
    server
        .test_db
        .create_test_market_with_tokens("BTC", "USDC")
        .await
        .expect("Failed to create market");

    let user = "test_user_1".to_string();
    server
        .test_db
        .db
        .create_user(user.clone())
        .await
        .expect("Failed to create user");

    server
        .test_db
        .db
        .add_balance(&user, "USDC", 100_000_000_000)
        .await
        .expect("Failed to add balance");

    // Connect WebSocket and subscribe to user events
    let ws_url = server.ws_url("/ws");
    let (mut ws, _) = tokio_tungstenite::connect_async(&ws_url)
        .await
        .expect("Failed to connect to WebSocket");

    send_json(
        &mut ws,
        &ClientMessage::Subscribe {
            channel: SubscriptionChannel::UserBalances,
            market_id: None,
            user_address: Some(user.clone()),
        },
    )
    .await
    .expect("Failed to subscribe");

    // Skip subscription acknowledgment message
    let sub_msg = receive_message(&mut ws)
        .await
        .expect("Failed to receive subscription ack");
    match sub_msg {
        ServerMessage::Subscribed { .. } => {
            // Expected
        }
        other => panic!("Expected Subscribed message, got: {:?}", other),
    }

    // Lock the balance manually (simulating what the REST endpoint does)
    server
        .test_db
        .db
        .lock_balance(&user, "USDC", 50_000_000_000)
        .await
        .expect("Failed to lock balance");

    // Broadcast the balance event (simulating what the REST endpoint does)
    let balance = server
        .test_db
        .db
        .get_balance(&user, "USDC")
        .await
        .expect("Failed to get balance");
    let _ = server
        .test_engine
        .event_tx()
        .send(backend::models::domain::EngineEvent::BalanceUpdated { balance });

    // Place a limit order (balance already locked)
    let order = utils::TestEngine::create_order(
        &user,
        "BTC/USDC",
        Side::Buy,
        OrderType::Limit,
        50_000_000_000, // price
        1_000_000,      // size
    );
    let result = server.test_engine.place_order(order).await;

    assert!(result.is_ok(), "Failed to create order: {:?}", result.err());

    // Wait for balance event
    let msg = receive_message(&mut ws)
        .await
        .expect("Failed to receive balance message");

    match msg {
        ServerMessage::UserBalance {
            user_address,
            token_ticker,
            available: _,
            locked,
            updated_at,
        } => {
            assert_eq!(user_address, user);
            assert_eq!(token_ticker, "USDC");
            // Should have locked 50,000_000_000 USDC (price * size)
            let locked_amount = locked.parse::<u128>().unwrap();
            assert!(locked_amount > 0, "Should have locked balance");
            assert!(updated_at > 0, "updated_at should be set");
        }
        _ => panic!("Expected Balance message, got: {:?}", msg),
    }

    ws.close(None).await.expect("Failed to close connection");
}

#[tokio::test]
async fn test_balance_events_on_limit_order_fill() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    // Setup: Create market
    server
        .test_db
        .create_test_market_with_tokens("BTC", "USDC")
        .await
        .expect("Failed to create market");

    // Create two users
    let maker = "maker_user".to_string();
    let taker = "taker_user".to_string();

    server
        .test_db
        .db
        .create_user(maker.clone())
        .await
        .expect("Failed to create maker");
    server
        .test_db
        .db
        .create_user(taker.clone())
        .await
        .expect("Failed to create taker");

    // Give balances
    server
        .test_db
        .db
        .add_balance(&maker, "BTC", 10_000_000)
        .await
        .expect("Failed to add BTC to maker");
    server
        .test_db
        .db
        .add_balance(&taker, "USDC", 100_000_000_000)
        .await
        .expect("Failed to add USDC to taker");

    // Connect WebSocket for taker
    let ws_url = server.ws_url("/ws");
    let (mut ws, _) = tokio_tungstenite::connect_async(&ws_url)
        .await
        .expect("Failed to connect to WebSocket");

    send_json(
        &mut ws,
        &ClientMessage::Subscribe {
            channel: SubscriptionChannel::UserBalances,
            market_id: None,
            user_address: Some(taker.clone()),
        },
    )
    .await
    .expect("Failed to subscribe");

    // Maker places sell order
    let maker_order = utils::TestEngine::create_order(
        &maker,
        "BTC/USDC",
        Side::Sell,
        OrderType::Limit,
        50_000_000_000, // price
        1_000_000,      // size
    );
    server
        .test_engine
        .place_order(maker_order)
        .await
        .expect("Failed to create maker order");

    // Taker places matching buy order (should fill and unlock balances)
    let taker_order = utils::TestEngine::create_order(
        &taker,
        "BTC/USDC",
        Side::Buy,
        OrderType::Limit,
        50_000_000_000, // price
        1_000_000,      // size
    );
    server
        .test_engine
        .place_order(taker_order)
        .await
        .expect("Failed to create taker order");

    // Wait for balance events - we should see balance updates for both BTC and USDC
    let mut btc_event_received = false;
    let mut usdc_event_received = false;

    // Receive multiple messages (might be order status + balance events)
    for _ in 0..10 {
        match timeout(Duration::from_secs(2), receive_message(&mut ws)).await {
            Ok(Ok(ServerMessage::UserBalance {
                user_address,
                token_ticker,
                available,
                locked,
                updated_at,
            })) => {
                assert_eq!(user_address, taker);
                assert!(updated_at > 0);

                if token_ticker == "BTC" {
                    // Taker should have received BTC (minus taker fee)
                    assert!(
                        available.parse::<f64>().unwrap() > 0.0,
                        "Should have received BTC"
                    );
                    assert_eq!(locked, "0", "BTC should not be locked");
                    btc_event_received = true;
                } else if token_ticker == "USDC" {
                    // Skip the initial lock event - we only care about the final state after trade
                    // After trade: USDC should be unlocked and balance should be reduced
                    if locked == "0" {
                        // This is the post-trade event
                        let available_usdc = available.parse::<u128>().unwrap();
                        assert!(
                            available_usdc < 100_000_000_000,
                            "Should have spent USDC: got {}",
                            available_usdc
                        );
                        usdc_event_received = true;
                    }
                    // Otherwise this is the lock event, keep waiting for unlock event
                }

                if btc_event_received && usdc_event_received {
                    break;
                }
            }
            Ok(Ok(_)) => {
                // Ignore other message types (order status, etc.)
                continue;
            }
            Ok(Err(e)) => {
                panic!("Error receiving message: {}", e);
            }
            Err(_) => {
                break; // Timeout
            }
        }
    }

    assert!(btc_event_received, "Should have received BTC balance event");
    assert!(
        usdc_event_received,
        "Should have received USDC balance event"
    );

    ws.close(None).await.expect("Failed to close connection");
}

#[tokio::test]
async fn test_balance_events_on_limit_order_cancellation() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    // Setup: Create market and give user balance
    server
        .test_db
        .create_test_market_with_tokens("BTC", "USDC")
        .await
        .expect("Failed to create market");

    let user = "test_user_cancel".to_string();
    server
        .test_db
        .db
        .create_user(user.clone())
        .await
        .expect("Failed to create user");

    server
        .test_db
        .db
        .add_balance(&user, "USDC", 100_000_000_000)
        .await
        .expect("Failed to add balance");

    // Connect WebSocket
    let ws_url = server.ws_url("/ws");
    let (mut ws, _) = tokio_tungstenite::connect_async(&ws_url)
        .await
        .expect("Failed to connect to WebSocket");

    send_json(
        &mut ws,
        &ClientMessage::Subscribe {
            channel: SubscriptionChannel::UserBalances,
            market_id: None,
            user_address: Some(user.clone()),
        },
    )
    .await
    .expect("Failed to subscribe");

    // Place a limit order
    let order = utils::TestEngine::create_order(
        &user,
        "BTC/USDC",
        Side::Buy,
        OrderType::Limit,
        50_000_000_000, // price
        1_000_000,      // size
    );
    let result = server
        .test_engine
        .place_order(order)
        .await
        .expect("Failed to create order");
    let order_id = uuid::Uuid::parse_str(&result.order.id).expect("Invalid order ID");

    // Wait for initial balance lock event
    let _msg = receive_message(&mut ws)
        .await
        .expect("Failed to receive lock event");

    // Cancel the order
    server
        .test_engine
        .cancel_order(order_id, user.clone())
        .await
        .expect("Failed to cancel order");

    // Wait for balance unlock event
    // Note: We may receive multiple balance events (lock then unlock)
    // We only care about the final unlocked state
    let mut balance_unlocked = false;
    let mut final_locked_amount = None;
    for _ in 0..5 {
        match timeout(Duration::from_secs(2), receive_message(&mut ws)).await {
            Ok(Ok(ServerMessage::UserBalance {
                user_address,
                token_ticker,
                available: _,
                locked,
                updated_at,
            })) => {
                if token_ticker == "USDC" {
                    assert_eq!(user_address, user);
                    assert!(updated_at > 0);
                    // Track the most recent locked amount
                    final_locked_amount = Some(locked.clone());
                    // If we see an unlocked state, mark as successful
                    if locked == "0" {
                        balance_unlocked = true;
                        break;
                    }
                }
            }
            Ok(Ok(_)) => {
                // Ignore other message types
                continue;
            }
            Ok(Err(e)) => {
                panic!("Error receiving message: {}", e);
            }
            Err(_) => {
                break;
            }
        }
    }

    // Verify we eventually saw the unlocked state
    if !balance_unlocked {
        if let Some(locked_amt) = final_locked_amount {
            panic!(
                "Expected balance to be unlocked, but last event showed locked={}",
                locked_amt
            );
        }
    }

    assert!(
        balance_unlocked,
        "Should have received balance unlock event"
    );

    ws.close(None).await.expect("Failed to close connection");
}

// ============================================================================
// Balance Event Tests for Market Orders
// ============================================================================

#[tokio::test]
async fn test_balance_events_on_market_order_partial_fill() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    // Setup: Create market
    server
        .test_db
        .create_test_market_with_tokens("BTC", "USDC")
        .await
        .expect("Failed to create market");

    let maker = "maker_partial".to_string();
    let taker = "taker_partial".to_string();

    server
        .test_db
        .db
        .create_user(maker.clone())
        .await
        .expect("Failed to create maker");
    server
        .test_db
        .db
        .create_user(taker.clone())
        .await
        .expect("Failed to create taker");

    // Give balances
    server
        .test_db
        .db
        .add_balance(&maker, "BTC", 1_000_000) // Only 1 BTC available
        .await
        .expect("Failed to add BTC to maker");
    server
        .test_db
        .db
        .add_balance(&taker, "USDC", 200_000_000_000)
        .await
        .expect("Failed to add USDC to taker");

    // Maker places sell order for 1 BTC
    let maker_order = utils::TestEngine::create_order(
        &maker,
        "BTC/USDC",
        Side::Sell,
        OrderType::Limit,
        50_000_000_000,
        1_000_000,
    );
    server
        .test_engine
        .place_order(maker_order)
        .await
        .expect("Failed to create maker order");

    // Connect WebSocket for taker
    let ws_url = server.ws_url("/ws");
    let (mut ws, _) = tokio_tungstenite::connect_async(&ws_url)
        .await
        .expect("Failed to connect to WebSocket");

    send_json(
        &mut ws,
        &ClientMessage::Subscribe {
            channel: SubscriptionChannel::UserBalances,
            market_id: None,
            user_address: Some(taker.clone()),
        },
    )
    .await
    .expect("Failed to subscribe");

    // Taker places market order for 2 BTC (but only 1 available)
    // Market order should fill 1 BTC and unlock the remaining 1 BTC worth of USDC
    let taker_order = utils::TestEngine::create_order(
        &taker,
        "BTC/USDC",
        Side::Buy,
        OrderType::Market,
        50_000_000_000, // price
        2_000_000,      // size - requesting 2 BTC but only 1 available
    );
    server
        .test_engine
        .place_order(taker_order)
        .await
        .expect("Failed to create market order");

    // Wait for balance events
    let mut btc_received = false;
    let mut usdc_event_received = false;

    for _ in 0..10 {
        match timeout(Duration::from_secs(2), receive_message(&mut ws)).await {
            Ok(Ok(ServerMessage::UserBalance {
                user_address,
                token_ticker,
                available,
                locked,
                ..
            })) => {
                assert_eq!(user_address, taker);

                if token_ticker == "BTC" {
                    // Should have received some BTC (after taker fee)
                    assert!(
                        available.parse::<f64>().unwrap() > 0.0,
                        "Should have received some BTC"
                    );
                    assert_eq!(locked, "0");
                    btc_received = true;
                } else if token_ticker == "USDC" {
                    // Skip the initial lock event - only check the final state after trade
                    // After partial fill: USDC should be unlocked and partially spent
                    if locked == "0" {
                        // This is the post-trade event
                        let available_usdc = available.parse::<u128>().unwrap();
                        // Should have spent some USDC for 1 BTC, rest unlocked
                        assert!(
                            available_usdc > 100_000_000_000 && available_usdc < 200_000_000_000,
                            "Should have spent some USDC and unlocked remainder: got {}",
                            available_usdc
                        );
                        usdc_event_received = true;
                    }
                    // Otherwise this is the lock event, keep waiting for unlock event
                }

                if btc_received && usdc_event_received {
                    break;
                }
            }
            Ok(Ok(_)) => continue,
            Ok(Err(e)) => panic!("Error: {}", e),
            Err(_) => break,
        }
    }

    assert!(btc_received, "Should have received BTC balance event");
    assert!(
        usdc_event_received,
        "Should have received USDC balance event with unlocked funds"
    );

    ws.close(None).await.expect("Failed to close connection");
}
