mod events;
mod subscriptions;

use axum::{
    body::Bytes,
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
    routing::get,
    Router,
};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::time::{interval, Duration, Instant};

use crate::api::ws::subscriptions::SubscriptionSet;
use crate::models::api::{ClientMessage, ServerMessage, Subscription};
use crate::models::domain::EngineEvent;

// Configuration constants
const PING_INTERVAL: Duration = Duration::from_secs(30);
const PONG_TIMEOUT: Duration = Duration::from_secs(60);
const UNSUBSCRIBED_TIMEOUT: Duration = Duration::from_secs(300); // 5 minutes

/// Create the WebSocket router
pub fn create_ws() -> Router<crate::AppState> {
    Router::new().route("/ws", get(ws_handler))
}

/// WebSocket upgrade handler
async fn ws_handler(ws: WebSocketUpgrade, State(state): State<crate::AppState>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle a WebSocket connection with ping/pong keepalive
async fn handle_socket(socket: WebSocket, state: crate::AppState) {
    let (sender, receiver) = socket.split();
    let subscriptions = Arc::new(RwLock::new(SubscriptionSet::new()));
    let event_rx = state.event_tx.subscribe();

    let last_pong = Arc::new(RwLock::new(Instant::now()));
    let last_subscription_change = Arc::new(RwLock::new(Instant::now()));

    // Task 1: Handle incoming messages from client
    let recv_task = {
        let subscriptions = subscriptions.clone();
        let last_pong = last_pong.clone();
        let last_subscription_change = last_subscription_change.clone();

        tokio::spawn(async move {
            handle_client_messages(receiver, subscriptions, last_pong, last_subscription_change)
                .await
        })
    };

    // Task 2: Send outgoing messages to client + ping/pong management
    let send_task = {
        let subscriptions = subscriptions.clone();
        let last_pong = last_pong.clone();
        let last_subscription_change = last_subscription_change.clone();

        tokio::spawn(async move {
            handle_server_messages(
                sender,
                event_rx,
                subscriptions,
                last_pong,
                last_subscription_change,
            )
            .await
        })
    };

    // Wait for either task to complete (disconnection)
    tokio::select! {
        _ = recv_task => {
            log::info!("WebSocket receive task ended");
        },
        _ = send_task => {
            log::info!("WebSocket send task ended");
        },
    }

    log::info!("WebSocket connection closed");
}

/// Handle incoming messages from the client
async fn handle_client_messages(
    mut receiver: futures::stream::SplitStream<WebSocket>,
    subscriptions: Arc<RwLock<SubscriptionSet>>,
    last_pong: Arc<RwLock<Instant>>,
    last_subscription_change: Arc<RwLock<Instant>>,
) {
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Parse and handle client message
                if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                    match client_msg {
                        ClientMessage::Subscribe {
                            channel,
                            market_id,
                            user_address,
                        } => {
                            if let Some(sub) = Subscription::from_channel(
                                &channel,
                                market_id.clone(),
                                user_address,
                            ) {
                                let was_added = subscriptions.write().await.subscribe(sub);
                                *last_subscription_change.write().await = Instant::now();

                                if was_added {
                                    log::debug!("Client subscribed to: {}", channel);
                                } else {
                                    log::debug!("Client already subscribed to: {}", channel);
                                }

                                // Note: Subscription acknowledgment would be sent from handle_server_messages
                                // if we had a channel to send messages back. For now, we just log.
                                // In a production system, you'd want to send:
                                // ServerMessage::Subscribed { channel, market_id }
                            } else {
                                log::warn!("Invalid subscription channel: {}", channel);
                            }
                        }
                        ClientMessage::Unsubscribe { channel, market_id } => {
                            if let Some(sub) =
                                Subscription::from_channel(&channel, market_id.clone(), None)
                            {
                                let was_removed = subscriptions.write().await.unsubscribe(&sub);
                                *last_subscription_change.write().await = Instant::now();

                                if was_removed {
                                    log::debug!("Client unsubscribed from: {}", channel);
                                } else {
                                    log::debug!("Client was not subscribed to: {}", channel);
                                }

                                // Note: Unsubscription acknowledgment would be sent similarly
                            } else {
                                log::warn!("Invalid unsubscription channel: {}", channel);
                            }
                        }
                        ClientMessage::Ping => {
                            // Application-level ping (we primarily use WebSocket ping/pong)
                            log::debug!("Received application ping");
                        }
                    }
                }
            }
            Ok(Message::Pong(_)) => {
                // Client responded to our ping
                *last_pong.write().await = Instant::now();
                log::debug!("Received pong from client");
            }
            Ok(Message::Close(frame)) => {
                log::info!("Client sent close frame: {:?}", frame);
                break;
            }
            Err(e) => {
                log::error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }
}

/// Handle outgoing messages to the client and ping/pong management
async fn handle_server_messages(
    mut sender: futures::stream::SplitSink<WebSocket, Message>,
    mut event_rx: broadcast::Receiver<EngineEvent>,
    subscriptions: Arc<RwLock<SubscriptionSet>>,
    last_pong: Arc<RwLock<Instant>>,
    last_subscription_change: Arc<RwLock<Instant>>,
) {
    let mut ping_interval = interval(PING_INTERVAL);

    loop {
        tokio::select! {
            // Send ping and check for timeouts
            _ = ping_interval.tick() => {
                // 1. Check if last pong was too long ago (dead connection)
                let pong_elapsed = last_pong.read().await.elapsed();
                if pong_elapsed > PONG_TIMEOUT {
                    log::warn!("No pong received for {:?}, disconnecting client", pong_elapsed);
                    break;
                }

                // 2. Check if client has no subscriptions for too long
                let subs = subscriptions.read().await;
                if subs.is_empty() {
                    let sub_elapsed = last_subscription_change.read().await.elapsed();
                    if sub_elapsed > UNSUBSCRIBED_TIMEOUT {
                        log::info!("Client has no subscriptions for {:?}, disconnecting", sub_elapsed);
                        break;
                    }
                }
                drop(subs); // Release lock

                // 3. Send ping
                if sender.send(Message::Ping(Bytes::new())).await.is_err() {
                    log::error!("Failed to send ping, client disconnected");
                    break;
                }
                log::debug!("Sent ping to client");
            }

            // Forward engine events to client
            Ok(event) = event_rx.recv() => {
                let subs = subscriptions.read().await;
                if subs.wants_event(&event) {
                    let server_msg = ServerMessage::from(event);
                    if let Ok(json) = serde_json::to_string(&server_msg) {
                        if sender.send(Message::Text(json.into())).await.is_err() {
                            log::error!("Failed to send message to client");
                            break;
                        }
                    }
                }
            }
        }
    }
}
