//! WebSocket server message handling - sends messages to clients

use axum::{
    body::Bytes,
    extract::ws::{Message, WebSocket},
};
use futures::SinkExt;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::time::interval;

use crate::models::api::{PriceLevel, ServerMessage};
use crate::models::domain::EngineEvent;

use super::{SocketState, PING_INTERVAL, PONG_TIMEOUT, UNSUBSCRIBED_TIMEOUT};

/// Handle outgoing messages to the client and ping/pong management
pub(super) async fn handle_server_messages(
    mut sender: futures::stream::SplitSink<WebSocket, Message>,
    mut event_rx: broadcast::Receiver<EngineEvent>,
    socket_state: Arc<RwLock<SocketState>>,
    mut ack_rx: tokio::sync::mpsc::UnboundedReceiver<ServerMessage>,
) {
    let mut ping_interval = interval(PING_INTERVAL);

    loop {
        tokio::select! {
            // Send ping and check for timeouts
            _ = ping_interval.tick() => {
                let state = socket_state.read().await;

                // 1. Check if last pong was too long ago (dead connection)
                if state.last_pong.elapsed() > PONG_TIMEOUT {
                    log::warn!("No pong received for {:?}, disconnecting client", state.last_pong.elapsed());
                    break;
                }

                // 2. Check if client has no subscriptions for too long
                if state.subscriptions.is_empty() && state.last_subscription_change.elapsed() > UNSUBSCRIBED_TIMEOUT {
                    log::info!("Client has no subscriptions for {:?}, disconnecting", state.last_subscription_change.elapsed());
                    break;
                }

                drop(state); // Release lock

                // 3. Send ping
                if sender.send(Message::Ping(Bytes::new())).await.is_err() {
                    log::error!("Failed to send ping, client disconnected");
                    break;
                }
                log::debug!("Sent ping to client");
            }

            // Send acknowledgment messages
            Some(ack) = ack_rx.recv() => {
                if let Ok(json) = serde_json::to_string(&ack) {
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        log::error!("Failed to send acknowledgment to client");
                        break;
                    }
                    log::debug!("Sent acknowledgment: {:?}", ack);
                }
            }

            // Forward engine events to client
            Ok(event) = event_rx.recv() => {
                let state = socket_state.read().await;
                if state.subscriptions.wants_event(&event) {
                    drop(state); // Release lock before serialization
                    let server_msg = engine_event_to_message(event);
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

/// Convert an EngineEvent to a ServerMessage for WebSocket transmission
fn engine_event_to_message(event: EngineEvent) -> ServerMessage {
    match event {
        EngineEvent::TradeExecuted { trade } => ServerMessage::Trade {
            market_id: trade.market_id,
            price: trade.price.to_string(),
            size: trade.size.to_string(),
            side: "unknown".to_string(), // Will be determined by client context
            timestamp: trade.timestamp.timestamp(),
        },
        EngineEvent::OrderPlaced { order } => ServerMessage::OrderUpdate {
            order_id: order.id.to_string(),
            status: format!("{:?}", order.status).to_lowercase(),
            filled_size: order.filled_size.to_string(),
        },
        EngineEvent::OrderCancelled { order_id, .. } => ServerMessage::OrderUpdate {
            order_id: order_id.to_string(),
            status: "cancelled".to_string(),
            filled_size: "0".to_string(),
        },
        EngineEvent::BalanceUpdated {
            token_ticker,
            available,
            locked,
            ..
        } => ServerMessage::BalanceUpdate {
            token_ticker,
            available: available.to_string(),
            locked: locked.to_string(),
        },
        EngineEvent::OrderbookSnapshot { orderbook } => ServerMessage::OrderbookSnapshot {
            market_id: orderbook.market_id,
            bids: orderbook
                .bids
                .into_iter()
                .map(|level| PriceLevel {
                    price: level.price.to_string(),
                    size: level.size.to_string(),
                })
                .collect(),
            asks: orderbook
                .asks
                .into_iter()
                .map(|level| PriceLevel {
                    price: level.price.to_string(),
                    size: level.size.to_string(),
                })
                .collect(),
        },
    }
}
