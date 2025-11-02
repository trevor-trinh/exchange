// process
// price time priority

pub mod executor;
pub mod matcher;
pub mod orderbook;

use crate::db::Db;
use crate::errors::ExchangeError;
use crate::models::api::{OrderCancelled, OrderPlaced, OrdersCancelled};
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
                EngineRequest::CancelAllOrders {
                    user_address,
                    market_id,
                    response_tx,
                } => {
                    let result = self.handle_cancel_all_orders(user_address, market_id).await;
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
        // Validate order against market config
        let market = self.db.get_market(&order.market_id).await?;
        Self::validate_order(&order, &market)?;

        // Persist initial order to database
        self.db.create_order(&order).await?;

        // Get matches from matcher and apply them
        let (matches, trades) = {
            let mut orderbooks = self.orderbooks.write().await;
            let orderbook = orderbooks.get_or_create(&order.market_id);

            // Match order against orderbook
            let matches = Matcher::match_order(&order, orderbook);

            // Execute trades if we have matches (also updates order status in DB)
            let trades = if !matches.is_empty() {
                Executor::execute(self.db.clone(), matches.clone(), &order, &market).await?
            } else {
                vec![]
            };

            // Update orderbook with executed trades
            orderbook.apply_trades(&order, &trades, &market);

            (matches, trades)
        };

        // Broadcast events
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

        // Handle unfilled/partially filled orders based on order type
        if order.filled_size < order.size {
            match order.order_type {
                crate::models::domain::OrderType::Market => {
                    // Market orders that don't fully fill are cancelled
                    // (IOC - Immediate or Cancel behavior)
                    order.status = if total_matched > 0 {
                        OrderStatus::PartiallyFilled
                    } else {
                        OrderStatus::Cancelled
                    };

                    // Update database with final status
                    self.db
                        .update_order_fill(order.id, order.filled_size, order.status)
                        .await?;

                    // Unlock the unfilled portion
                    let unfilled_size = order.size - order.filled_size;
                    if unfilled_size > 0 {
                        let (token_to_unlock, amount_to_unlock) = match order.side {
                            crate::models::domain::Side::Buy => {
                                let quote_amount = order
                                    .price
                                    .checked_mul(unfilled_size)
                                    .ok_or_else(|| ExchangeError::InvalidParameter {
                                        message: "Unlock amount overflow".to_string(),
                                    })?;
                                (market.quote_ticker.clone(), quote_amount)
                            }
                            crate::models::domain::Side::Sell => {
                                (market.base_ticker.clone(), unfilled_size)
                            }
                        };

                        self.db
                            .unlock_balance(&order.user_address, &token_to_unlock, amount_to_unlock)
                            .await?;
                    }
                }
                crate::models::domain::OrderType::Limit => {
                    // Limit orders stay on the book
                    let _ = self.event_tx.send(EngineEvent::OrderPlaced {
                        order: order.clone(),
                    });
                }
            }
        }

        Ok(OrderPlaced {
            order: order.into(),
            trades: trades.into_iter().map(|t| t.into()).collect(),
        })
    }

    /// Handle cancelling an order
    async fn handle_cancel_order(
        &mut self,
        order_id: uuid::Uuid,
        user_address: String,
    ) -> Result<OrderCancelled, ExchangeError> {
        // Cancel order using orderbooks method (handles search and ownership verification)
        let cancelled_order = {
            let mut orderbooks = self.orderbooks.write().await;
            orderbooks.cancel_order(order_id, &user_address)?
        };

        // Get market config to determine which token to unlock
        let market = self.db.get_market(&cancelled_order.market_id).await?;

        // Calculate unfilled amount that needs to be unlocked
        let unfilled_size = cancelled_order.size - cancelled_order.filled_size;

        if unfilled_size > 0 {
            // Determine which token and amount to unlock based on order side
            let (token_to_unlock, amount_to_unlock) = match cancelled_order.side {
                crate::models::domain::Side::Buy => {
                    // Buy order: unlock quote tokens (price * unfilled_size)
                    let quote_amount = cancelled_order
                        .price
                        .checked_mul(unfilled_size)
                        .ok_or_else(|| ExchangeError::InvalidParameter {
                            message: "Unlock amount overflow".to_string(),
                        })?;
                    (market.quote_ticker, quote_amount)
                }
                crate::models::domain::Side::Sell => {
                    // Sell order: unlock base tokens (unfilled_size)
                    (market.base_ticker, unfilled_size)
                }
            };

            // Unlock the balance
            self.db
                .unlock_balance(&user_address, &token_to_unlock, amount_to_unlock)
                .await?;
        }

        // Update order status in database
        self.db
            .update_order_fill(
                order_id,
                cancelled_order.filled_size,
                OrderStatus::Cancelled,
            )
            .await?;

        // Broadcast cancellation event
        let _ = self.event_tx.send(EngineEvent::OrderCancelled {
            order_id,
            user_address: user_address.clone(),
        });

        Ok(OrderCancelled {
            order_id: order_id.to_string(),
        })
    }

    /// Handle cancelling all orders for a user
    async fn handle_cancel_all_orders(
        &mut self,
        user_address: String,
        market_id: Option<String>,
    ) -> Result<OrdersCancelled, ExchangeError> {
        // Cancel all orders for the user using orderbooks method
        let cancelled_orders = {
            let mut orderbooks = self.orderbooks.write().await;
            orderbooks.cancel_all_orders(&user_address, market_id.as_deref())
        };

        let mut cancelled_order_ids = Vec::new();

        // Process each cancelled order
        // Continue processing even if individual unlocks fail to prevent orphaned locks
        for cancelled_order in cancelled_orders {
            let order_id = cancelled_order.id;

            // Get market config to determine which token to unlock
            let market = match self.db.get_market(&cancelled_order.market_id).await {
                Ok(m) => m,
                Err(e) => {
                    log::error!("Failed to get market for order {}: {}", order_id, e);
                    continue;
                }
            };

            // Calculate unfilled amount that needs to be unlocked
            let unfilled_size = cancelled_order.size - cancelled_order.filled_size;

            if unfilled_size > 0 {
                // Determine which token and amount to unlock based on order side
                let unlock_result = match cancelled_order.side {
                    crate::models::domain::Side::Buy => {
                        // Buy order: unlock quote tokens (price * unfilled_size)
                        match cancelled_order.price.checked_mul(unfilled_size) {
                            Some(quote_amount) => {
                                self.db
                                    .unlock_balance(
                                        &user_address,
                                        &market.quote_ticker,
                                        quote_amount,
                                    )
                                    .await
                            }
                            None => {
                                log::error!(
                                    "Overflow calculating unlock amount for order {}",
                                    order_id
                                );
                                continue;
                            }
                        }
                    }
                    crate::models::domain::Side::Sell => {
                        // Sell order: unlock base tokens (unfilled_size)
                        self.db
                            .unlock_balance(&user_address, &market.base_ticker, unfilled_size)
                            .await
                    }
                };

                // Log unlock failures but continue processing
                if let Err(e) = unlock_result {
                    log::error!("Failed to unlock balance for order {}: {}", order_id, e);
                }
            }

            // Update order status in database
            if let Err(e) = self
                .db
                .update_order_fill(
                    order_id,
                    cancelled_order.filled_size,
                    OrderStatus::Cancelled,
                )
                .await
            {
                log::error!("Failed to update order {} status: {}", order_id, e);
                continue;
            }

            // Broadcast cancellation event
            let _ = self.event_tx.send(EngineEvent::OrderCancelled {
                order_id,
                user_address: user_address.clone(),
            });

            cancelled_order_ids.push(order_id.to_string());
        }

        let count = cancelled_order_ids.len();

        Ok(OrdersCancelled {
            cancelled_order_ids,
            count,
        })
    }

    /// Spawn a background task that periodically broadcasts orderbook snapshots
    /// Snapshots are sent every 1s for all active markets
    fn spawn_snapshot_broadcaster(&self) -> JoinHandle<()> {
        let event_tx = self.event_tx.clone();
        let orderbooks = Arc::clone(&self.orderbooks);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(1000));
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

    /// Validate order against market configuration
    fn validate_order(
        order: &crate::models::domain::Order,
        market: &crate::models::domain::Market,
    ) -> Result<(), ExchangeError> {
        // Validate tick size for limit orders only (price matters for limit orders)
        if order.order_type == crate::models::domain::OrderType::Limit {
            if order.price % market.tick_size != 0 {
                return Err(ExchangeError::InvalidParameter {
                    message: format!(
                        "Price {} is not a multiple of tick size {}",
                        order.price, market.tick_size
                    ),
                });
            }
        }

        // Validate lot size (size must be multiple of lot_size)
        if order.size % market.lot_size != 0 {
            return Err(ExchangeError::InvalidParameter {
                message: format!(
                    "Size {} is not a multiple of lot size {}",
                    order.size, market.lot_size
                ),
            });
        }

        // Validate minimum order size
        if order.size < market.min_size {
            return Err(ExchangeError::InvalidParameter {
                message: format!(
                    "Size {} is below minimum order size {}",
                    order.size, market.min_size
                ),
            });
        }

        Ok(())
    }
}
