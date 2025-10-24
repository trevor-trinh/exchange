use std::collections::HashSet;

use crate::models::api::{PriceLevel, ServerMessage, Subscription};
use crate::models::domain::EngineEvent;

/// Check if an engine event should be sent to a client based on their subscriptions
pub fn should_send_event(event: &EngineEvent, subscriptions: &HashSet<Subscription>) -> bool {
    match event {
        EngineEvent::TradeExecuted { trade } => {
            // Send if subscribed to this market's trades
            subscriptions.contains(&Subscription::Trades {
                market_id: trade.market_id.clone(),
            })
            // OR if subscribed as buyer
            || subscriptions.contains(&Subscription::User {
                user_address: trade.buyer_address.clone(),
            })
            // OR if subscribed as seller
            || subscriptions.contains(&Subscription::User {
                user_address: trade.seller_address.clone(),
            })
        }
        EngineEvent::OrderPlaced { order } => subscriptions.contains(&Subscription::User {
            user_address: order.user_address.clone(),
        }),
        EngineEvent::OrderCancelled { user_address, .. } => {
            subscriptions.contains(&Subscription::User {
                user_address: user_address.clone(),
            })
        }
        EngineEvent::OrderbookChanged { market_id, .. } => {
            subscriptions.contains(&Subscription::Orderbook {
                market_id: market_id.clone(),
            })
        }
        EngineEvent::BalanceUpdated { user_address, .. } => {
            subscriptions.contains(&Subscription::User {
                user_address: user_address.clone(),
            })
        }
    }
}

/// Convert an EngineEvent to a ServerMessage for WebSocket transmission
pub fn event_to_message(event: EngineEvent) -> ServerMessage {
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
        EngineEvent::OrderbookChanged {
            market_id,
            bids,
            asks,
        } => ServerMessage::OrderbookUpdate {
            market_id,
            bids: bids
                .into_iter()
                .map(|(price, size)| PriceLevel {
                    price: price.to_string(),
                    size: size.to_string(),
                })
                .collect(),
            asks: asks
                .into_iter()
                .map(|(price, size)| PriceLevel {
                    price: price.to_string(),
                    size: size.to_string(),
                })
                .collect(),
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
    }
}
