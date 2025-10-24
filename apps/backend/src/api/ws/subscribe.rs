use std::collections::HashSet;
use tokio::time::Instant;
use tokio::sync::RwLock;
use std::sync::Arc;

use crate::models::api::{ClientMessage, Subscription};

/// Handle subscription/unsubscription logic from client messages
pub async fn handle_subscription_message(
    msg: ClientMessage,
    subscriptions: &Arc<RwLock<HashSet<Subscription>>>,
    last_subscription_change: &Arc<RwLock<Instant>>,
) {
    match msg {
        ClientMessage::Subscribe {
            channel,
            market_id,
            user_address,
        } => {
            if let Some(sub) = Subscription::from_channel(&channel, market_id, user_address) {
                subscriptions.write().await.insert(sub);
                *last_subscription_change.write().await = Instant::now();
                log::debug!("Client subscribed to: {}", channel);
            }
        }
        ClientMessage::Unsubscribe { channel, market_id } => {
            if let Some(sub) = Subscription::from_channel(&channel, market_id, None) {
                subscriptions.write().await.remove(&sub);
                *last_subscription_change.write().await = Instant::now();
                log::debug!("Client unsubscribed from: {}", channel);
            }
        }
        ClientMessage::Ping => {
            // Application-level ping (we primarily use WebSocket ping/pong)
            log::debug!("Received application ping");
        }
    }
}
