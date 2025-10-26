use std::collections::HashSet;

use crate::models::api::Subscription;
use crate::models::domain::EngineEvent;

/// Manages client subscriptions and determines which events to forward
#[derive(Debug, Default)]
pub struct SubscriptionSet {
    subs: HashSet<Subscription>,
}

impl SubscriptionSet {
    /// Create a new empty subscription set
    pub fn new() -> Self {
        Self {
            subs: HashSet::new(),
        }
    }

    /// Subscribe to a channel
    /// Returns true if subscription was added, false if already existed
    pub fn subscribe(&mut self, sub: Subscription) -> bool {
        self.subs.insert(sub)
    }

    /// Unsubscribe from a channel
    /// Returns true if subscription was removed, false if didn't exist
    pub fn unsubscribe(&mut self, sub: &Subscription) -> bool {
        self.subs.remove(sub)
    }

    /// Check if this subscription set wants a particular event
    pub fn wants_event(&self, event: &EngineEvent) -> bool {
        match event {
            EngineEvent::TradeExecuted { trade } => {
                // Send if subscribed to this market's trades
                self.subs.contains(&Subscription::Trades {
                    market_id: trade.market_id.clone(),
                })
                // OR if subscribed as buyer
                || self.subs.contains(&Subscription::User {
                    user_address: trade.buyer_address.clone(),
                })
                // OR if subscribed as seller
                || self.subs.contains(&Subscription::User {
                    user_address: trade.seller_address.clone(),
                })
            }
            EngineEvent::OrderPlaced { order } => self.subs.contains(&Subscription::User {
                user_address: order.user_address.clone(),
            }),
            EngineEvent::OrderCancelled { user_address, .. } => {
                self.subs.contains(&Subscription::User {
                    user_address: user_address.clone(),
                })
            }
            EngineEvent::OrderbookChanged { market_id, .. } => {
                self.subs.contains(&Subscription::Orderbook {
                    market_id: market_id.clone(),
                })
            }
            EngineEvent::BalanceUpdated { user_address, .. } => {
                self.subs.contains(&Subscription::User {
                    user_address: user_address.clone(),
                })
            }
        }
    }

    /// Check if the subscription set is empty
    pub fn is_empty(&self) -> bool {
        self.subs.is_empty()
    }
}
