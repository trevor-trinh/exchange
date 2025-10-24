mod helpers;
mod subscribe;

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
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::time::{interval, Duration, Instant};

use crate::models::api::Subscription;
use crate::models::domain::EngineEvent;
use helpers::{event_to_message, should_send_event};
use subscribe::handle_subscription_message;

// Configuration constants
const PING_INTERVAL: Duration = Duration::from_secs(30);
const PONG_TIMEOUT: Duration = Duration::from_secs(60);
const UNSUBSCRIBED_TIMEOUT: Duration = Duration::from_secs(300); // 5 minutes

/// Create WebSocket routes
/// Note: This function is generic to allow any state type
/// When wiring up in main.rs, you'll need to provide AppState
pub fn create_ws_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static + WsState,
{
    Router::new().route("/ws", get(ws_handler::<S>))
}

/// WebSocket upgrade handler
async fn ws_handler<S>(ws: WebSocketUpgrade, State(state): State<S>) -> Response
where
    S: Clone + Send + Sync + 'static + WsState,
{
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Trait that AppState must implement to provide WebSocket functionality
pub trait WsState {
    fn event_tx(&self) -> &broadcast::Sender<EngineEvent>;
}

/// Handle a WebSocket connection with ping/pong keepalive
async fn handle_socket<S>(socket: WebSocket, state: S)
where
    S: WsState,
{
    let (sender, receiver) = socket.split();
    let subscriptions = Arc::new(RwLock::new(HashSet::<Subscription>::new()));
    let event_rx = state.event_tx().subscribe();

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
    subscriptions: Arc<RwLock<HashSet<Subscription>>>,
    last_pong: Arc<RwLock<Instant>>,
    last_subscription_change: Arc<RwLock<Instant>>,
) {
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Parse and handle client message
                if let Ok(client_msg) = serde_json::from_str(&text) {
                    handle_subscription_message(
                        client_msg,
                        &subscriptions,
                        &last_subscription_change,
                    )
                    .await;
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
    subscriptions: Arc<RwLock<HashSet<Subscription>>>,
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
                if should_send_event(&event, &*subs) {
                    let server_msg = event_to_message(event);
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
