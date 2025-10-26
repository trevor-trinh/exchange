//! WebSocket client message handling - processes messages from clients

use axum::extract::ws::{Message, WebSocket};
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Instant;

use crate::models::api::{ClientMessage, ServerMessage};
use crate::models::domain::Subscription;

use super::SocketState;

/// Handle incoming messages from the client
pub(super) async fn handle_client_messages(
    mut receiver: futures::stream::SplitStream<WebSocket>,
    socket_state: Arc<RwLock<SocketState>>,
    ack_tx: tokio::sync::mpsc::UnboundedSender<ServerMessage>,
) {
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Parse and handle client message
                if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                    match &client_msg {
                        ClientMessage::Subscribe {
                            channel,
                            market_id,
                            user_address,
                        } => {
                            if let Some(sub) = Subscription::from_message(&client_msg) {
                                let mut state = socket_state.write().await;
                                let was_added = state.subscriptions.subscribe(sub);
                                state.last_subscription_change = Instant::now();
                                drop(state);

                                // Send acknowledgment
                                let ack = ServerMessage::Subscribed {
                                    channel: channel.clone(),
                                    market_id: market_id.clone(),
                                    user_address: user_address.clone(),
                                };
                                let _ = ack_tx.send(ack);

                                if was_added {
                                    log::debug!("Client subscribed to {:?}", channel);
                                } else {
                                    log::debug!("Client already subscribed to {:?}", channel);
                                }
                            } else {
                                log::warn!("Invalid subscription: missing required fields");
                            }
                        }

                        ClientMessage::Unsubscribe {
                            channel,
                            market_id,
                            user_address,
                        } => {
                            if let Some(sub) = Subscription::from_message(&client_msg) {
                                let mut state = socket_state.write().await;
                                let was_removed = state.subscriptions.unsubscribe(&sub);
                                state.last_subscription_change = Instant::now();
                                drop(state);

                                // Send acknowledgment
                                let ack = ServerMessage::Unsubscribed {
                                    channel: channel.clone(),
                                    market_id: market_id.clone(),
                                    user_address: user_address.clone(),
                                };
                                let _ = ack_tx.send(ack);

                                if was_removed {
                                    log::debug!("Client unsubscribed from {:?}", channel);
                                } else {
                                    log::debug!("Client was not subscribed to {:?}", channel);
                                }
                            } else {
                                log::warn!("Invalid unsubscription: missing required fields");
                            }
                        }

                        ClientMessage::Ping => {
                            log::debug!("Received application ping");
                        }
                    }
                }
            }
            Ok(Message::Pong(_)) => {
                socket_state.write().await.last_pong = Instant::now();
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
