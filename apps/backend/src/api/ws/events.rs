use crate::models::api::{PriceLevel, ServerMessage};
use crate::models::domain::EngineEvent;

/// Convert an EngineEvent to a ServerMessage for WebSocket transmission
impl From<EngineEvent> for ServerMessage {
    fn from(event: EngineEvent) -> Self {
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
}
