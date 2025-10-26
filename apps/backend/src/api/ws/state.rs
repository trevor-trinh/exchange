//! WebSocket connection state management

use std::collections::HashSet;
use tokio::time::Instant;

use crate::models::domain::EngineEvent;
use crate::models::domain::Subscription;

// ============================================================================
// SocketState - Shared connection state
// ============================================================================

/// Shared state for a WebSocket connection
pub(crate) struct SocketState {
    pub(crate) subscriptions: SubscriptionSet,
    pub(crate) last_pong: Instant,
    pub(crate) last_subscription_change: Instant,
}

impl SocketState {
    pub(crate) fn new() -> Self {
        Self {
            subscriptions: SubscriptionSet::new(),
            last_pong: Instant::now(),
            last_subscription_change: Instant::now(),
        }
    }
}

// ============================================================================
// SubscriptionSet - Manages client subscriptions
// ============================================================================

/// Manages client subscriptions and determines which events to forward
#[derive(Debug, Default)]
pub(crate) struct SubscriptionSet {
    subs: HashSet<Subscription>,
}

impl SubscriptionSet {
    pub(crate) fn new() -> Self {
        Self {
            subs: HashSet::new(),
        }
    }

    pub(crate) fn subscribe(&mut self, sub: Subscription) -> bool {
        self.subs.insert(sub)
    }

    pub(crate) fn unsubscribe(&mut self, sub: &Subscription) -> bool {
        self.subs.remove(sub)
    }

    pub(crate) fn wants_event(&self, event: &EngineEvent) -> bool {
        match event {
            EngineEvent::TradeExecuted { trade } => {
                self.subs.contains(&Subscription::Trades {
                    market_id: trade.market_id.clone(),
                }) || self.subs.contains(&Subscription::User {
                    user_address: trade.buyer_address.clone(),
                }) || self.subs.contains(&Subscription::User {
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
            EngineEvent::BalanceUpdated { user_address, .. } => {
                self.subs.contains(&Subscription::User {
                    user_address: user_address.clone(),
                })
            }
            EngineEvent::OrderbookSnapshot { orderbook } => {
                self.subs.contains(&Subscription::Orderbook {
                    market_id: orderbook.market_id.clone(),
                })
            }
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.subs.is_empty()
    }
}
