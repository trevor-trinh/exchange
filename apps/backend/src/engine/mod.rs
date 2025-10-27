// process
// price time priority

pub mod executor;
pub mod matcher;
pub mod orderbook;

use crate::db::Db;
use crate::errors::ExchangeError;
use crate::models::api::{OrderCancelled, OrderPlaced};
use crate::models::domain::{EngineEvent, EngineRequest, OrderStatus};
use executor::Executor;
use matcher::Matcher;
use orderbook::Orderbooks;

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::task::JoinHandle;

pub struct MatchingEngine {
    db: Db,
    orderbooks: Arc<RwLock<Orderbooks>>,

    engine_rx: mpsc::Receiver<EngineRequest>,
    event_tx: broadcast::Sender<EngineEvent>,
}

impl MatchingEngine {
    pub fn new(
        db: Db,
        engine_rx: mpsc::Receiver<EngineRequest>,
        event_tx: broadcast::Sender<EngineEvent>,
    ) -> Self {
        Self {
            db: db.clone(),
            orderbooks: Arc::new(RwLock::new(Orderbooks::new())),
            engine_rx,
            event_tx,
        }
    }

    pub async fn run(mut self) {
        // Spawn background task for orderbook snapshots
        let snapshot_handle = self.spawn_snapshot_broadcaster();

        // Main event loop - process incoming requests
        while let Some(request) = self.engine_rx.recv().await {
            match request {
                EngineRequest::PlaceOrder { order, response_tx } => {
                    let result = self.handle_place_order(order).await;
                    let _ = response_tx.send(result);
                }
                EngineRequest::CancelOrder {
                    order_id,
                    user_address,
                    response_tx,
                } => {
                    let result = self.handle_cancel_order(order_id, user_address).await;
                    let _ = response_tx.send(result);
                }
            }
        }

        // Cleanup: abort the snapshot broadcaster when engine stops
        snapshot_handle.abort();
    }

    /// Handle placing a new order
    async fn handle_place_order(
        &mut self,
        mut order: crate::models::domain::Order,
    ) -> Result<OrderPlaced, ExchangeError> {
        // 1. Get matches from matcher and apply them
        let (matches, trades) = {
            let mut orderbooks = self.orderbooks.write().await;
            let orderbook = orderbooks.get_or_create(&order.market_id);

            // Match order against orderbook
            let matches = Matcher::match_order(&order, orderbook);

            // Execute trades if we have matches
            let trades = if !matches.is_empty() {
                let maker_orders = orderbook.get_maker_orders(&matches);
                Executor::execute(self.db.clone(), matches.clone(), &order, &maker_orders).await?
            } else {
                vec![]
            };

            // Update orderbook with matches
            orderbook.apply_matches(&order, &matches);

            (matches, trades)
        };

        // 6. Broadcast events
        for trade in &trades {
            let _ = self.event_tx.send(EngineEvent::TradeExecuted {
                trade: trade.clone(),
            });
        }

        // Update order status for response
        let total_matched: u128 = matches.iter().map(|m| m.size).sum();
        order.filled_size = total_matched;
        if total_matched >= order.size {
            order.status = OrderStatus::Filled;
        } else if total_matched > 0 {
            order.status = OrderStatus::PartiallyFilled;
        }

        // Only broadcast if order is still on the book
        if order.filled_size < order.size {
            let _ = self.event_tx.send(EngineEvent::OrderPlaced {
                order: order.clone(),
            });
        }

        Ok(OrderPlaced { order, trades })
    }

    /// Handle cancelling an order
    async fn handle_cancel_order(
        &mut self,
        order_id: uuid::Uuid,
        user_address: String,
    ) -> Result<OrderCancelled, ExchangeError> {
        // Cancel order using orderbooks method (handles search and ownership verification)
        let _cancelled_order = {
            let mut orderbooks = self.orderbooks.write().await;
            orderbooks.cancel_order(order_id, &user_address)?
        };

        // TODO: Release locked funds in database
        // self.db.release_order_funds(&cancelled_order).await?;

        // Broadcast cancellation event
        let _ = self.event_tx.send(EngineEvent::OrderCancelled {
            order_id,
            user_address: user_address.clone(),
        });

        Ok(OrderCancelled {
            order_id: order_id.to_string(),
        })
    }

    /// Spawn a background task that periodically broadcasts orderbook snapshots
    /// Snapshots are sent every 100ms for all active markets
    fn spawn_snapshot_broadcaster(&self) -> JoinHandle<()> {
        let event_tx = self.event_tx.clone();
        let orderbooks = Arc::clone(&self.orderbooks);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            loop {
                interval.tick().await;

                // Get snapshots for all markets
                let snapshots = {
                    let orderbooks_read = orderbooks.read().await;
                    orderbooks_read.snapshots()
                };

                // Broadcast each snapshot
                for snapshot in snapshots {
                    let _ = event_tx.send(EngineEvent::OrderbookSnapshot {
                        orderbook: snapshot,
                    });
                }
            }
        })
    }
}
