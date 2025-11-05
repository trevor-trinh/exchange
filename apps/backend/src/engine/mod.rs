// process
// price time priority

pub mod executor;
pub mod matcher;
pub mod orderbook;

use crate::db::Db;
use crate::errors::ExchangeError;
use crate::models::api::{OrderCancelled, OrderPlaced, OrdersCancelled};
use crate::models::domain::{EngineEvent, EngineRequest, OrderStatus};
use executor::{AffectedBalances, Executor};
use matcher::Matcher;
use orderbook::Orderbooks;

use std::collections::HashSet;
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
            // Process request and collect affected balances
            let affected = match request {
                EngineRequest::PlaceOrder { order, response_tx } => {
                    let (result, affected) = self.handle_place_order(order).await;
                    let _ = response_tx.send(result);
                    affected
                }
                EngineRequest::CancelOrder {
                    order_id,
                    user_address,
                    response_tx,
                } => {
                    let (result, affected) = self.handle_cancel_order(order_id, user_address).await;
                    let _ = response_tx.send(result);
                    affected
                }
                EngineRequest::CancelAllOrders {
                    user_address,
                    market_id,
                    response_tx,
                } => {
                    let (result, affected) = self.handle_cancel_all_orders(user_address, market_id).await;
                    let _ = response_tx.send(result);
                    affected
                }
            };

            // Broadcast consolidated balance updates for all affected users
            // This ensures only one update per user-token pair per request
            for (user_address, token_ticker) in affected {
                if let Ok(balance) = self.db.get_balance(&user_address, &token_ticker).await {
                    let _ = self.event_tx.send(EngineEvent::BalanceUpdated { balance });
                }
            }
        }

        // Cleanup: abort the snapshot broadcaster when engine stops
        snapshot_handle.abort();
    }

    /// Handle placing a new order
    /// Returns the result and set of affected balances to broadcast
    async fn handle_place_order(
        &mut self,
        mut order: crate::models::domain::Order,
    ) -> (Result<OrderPlaced, ExchangeError>, AffectedBalances) {
        let mut affected = HashSet::new();

        // Validate order against market config
        let market = match self.db.get_market(&order.market_id).await {
            Ok(m) => m,
            Err(e) => return (Err(e), affected),
        };
        if let Err(e) = Self::validate_order(&order, &market) {
            return (Err(e), affected);
        }

        // Calculate and lock balance (after validation, before matching)
        let (token_to_lock, amount_to_lock) = match self.calculate_lock_amount(&order, &market).await {
            Ok(v) => v,
            Err(e) => return (Err(e), affected),
        };

        if let Err(e) = self.db
            .lock_balance(&order.user_address, &token_to_lock, amount_to_lock)
            .await
        {
            return (Err(e), affected);
        }

        // Track balance that was locked
        affected.insert((order.user_address.clone(), token_to_lock.clone()));

        // Persist initial order to database
        // If this fails, unlock balance before returning error
        if let Err(e) = self.db.create_order(&order).await {
            let _ = self
                .db
                .unlock_balance(&order.user_address, &token_to_lock, amount_to_lock)
                .await;
            return (Err(e), affected);
        }

        // Get matches from matcher and apply them
        let (matches, trades) = {
            let mut orderbooks = self.orderbooks.write().await;
            let orderbook = orderbooks.get_or_create(&order.market_id);

            // Match order against orderbook
            let matches = Matcher::match_order(&order, orderbook);

            // Execute trades if we have matches (also updates order status in DB)
            let (trades, executor_affected) = if !matches.is_empty() {
                match Executor::execute(
                    self.db.clone(),
                    matches.clone(),
                    &order,
                    &market,
                )
                .await
                {
                    Ok((trades, exec_affected)) => (trades, exec_affected),
                    Err(e) => {
                        // Execution failed - unlock the full order amount
                        let _ = self
                            .db
                            .unlock_balance(&order.user_address, &token_to_lock, amount_to_lock)
                            .await;
                        return (Err(e), affected);
                    }
                }
            } else {
                (vec![], HashSet::new())
            };

            // Track all balances affected by execution
            affected.extend(executor_affected);

            // Update orderbook with executed trades
            orderbook.apply_trades(&order, &trades, &market);

            (matches, trades)
        };

        // Broadcast trade events
        for trade in &trades {
            let _ = self.event_tx.send(EngineEvent::TradeExecuted {
                trade: trade.clone(),
            });
        }

        // Broadcast order fill updates for maker orders
        for m in &matches {
            let maker_order = &m.maker_order;
            let maker_new_filled = maker_order.filled_size + m.size;
            let maker_status = if maker_new_filled >= maker_order.size {
                OrderStatus::Filled
            } else {
                OrderStatus::PartiallyFilled
            };

            let _ = self.event_tx.send(EngineEvent::OrderPlaced {
                order: crate::models::domain::Order {
                    id: maker_order.id,
                    user_address: maker_order.user_address.clone(),
                    market_id: maker_order.market_id.clone(),
                    side: maker_order.side,
                    order_type: maker_order.order_type,
                    price: maker_order.price,
                    size: maker_order.size,
                    filled_size: maker_new_filled,
                    status: maker_status,
                    created_at: maker_order.created_at,
                    updated_at: chrono::Utc::now(),
                },
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

        // Broadcast taker order update if it got filled or partially filled
        if total_matched > 0 {
            let _ = self.event_tx.send(EngineEvent::OrderPlaced {
                order: order.clone(),
            });
        }

        // Handle unfilled/partially filled orders based on order type
        if order.filled_size < order.size {
            match order.order_type {
                crate::models::domain::OrderType::Market => {
                    // Market orders that don't fully fill are cancelled
                    // (IOC - Immediate or Cancel behavior)
                    // Market orders that execute (even partially) are marked as Filled
                    // since they cannot remain on the book
                    order.status = if total_matched > 0 {
                        OrderStatus::Filled
                    } else {
                        OrderStatus::Cancelled
                    };

                    // Update database with final status
                    if let Err(e) = self.db
                        .update_order_fill(order.id, order.filled_size, order.status)
                        .await
                    {
                        return (Err(e), affected);
                    }

                    // Unlock the unfilled portion
                    let unfilled_size = order.size - order.filled_size;
                    if unfilled_size > 0 {
                        let (token_to_unlock, amount_to_unlock) = match order.side {
                            crate::models::domain::Side::Buy => {
                                match order
                                    .price
                                    .checked_mul(unfilled_size)
                                {
                                    Some(quote_amount) => (market.quote_ticker.clone(), quote_amount),
                                    None => {
                                        return (Err(ExchangeError::InvalidParameter {
                                            message: "Unlock amount overflow".to_string(),
                                        }), affected);
                                    }
                                }
                            }
                            crate::models::domain::Side::Sell => {
                                (market.base_ticker.clone(), unfilled_size)
                            }
                        };

                        if let Err(e) = self.db
                            .unlock_balance(&order.user_address, &token_to_unlock, amount_to_unlock)
                            .await
                        {
                            return (Err(e), affected);
                        }

                        // Track unlocked balance
                        affected.insert((order.user_address.clone(), token_to_unlock));
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

        (
            Ok(OrderPlaced {
                order: order.into(),
                trades: trades.into_iter().map(|t| t.into()).collect(),
            }),
            affected,
        )
    }

    /// Handle cancelling an order
    /// Returns the result and set of affected balances to broadcast
    async fn handle_cancel_order(
        &mut self,
        order_id: uuid::Uuid,
        user_address: String,
    ) -> (Result<OrderCancelled, ExchangeError>, AffectedBalances) {
        let mut affected = HashSet::new();

        // Cancel order using orderbooks method (handles search and ownership verification)
        let cancelled_order = {
            let mut orderbooks = self.orderbooks.write().await;
            match orderbooks.cancel_order(order_id, &user_address) {
                Ok(order) => order,
                Err(e) => return (Err(e), affected),
            }
        };

        // Get market config to determine which token to unlock
        let market = match self.db.get_market(&cancelled_order.market_id).await {
            Ok(m) => m,
            Err(e) => return (Err(e), affected),
        };

        // Calculate unfilled amount that needs to be unlocked
        let unfilled_size = cancelled_order.size - cancelled_order.filled_size;

        if unfilled_size > 0 {
            // Determine which token and amount to unlock based on order side
            let (token_to_unlock, amount_to_unlock) = match cancelled_order.side {
                crate::models::domain::Side::Buy => {
                    // Buy order: unlock quote tokens (price * unfilled_size)
                    match cancelled_order.price.checked_mul(unfilled_size) {
                        Some(quote_amount) => (market.quote_ticker, quote_amount),
                        None => {
                            return (
                                Err(ExchangeError::InvalidParameter {
                                    message: "Unlock amount overflow".to_string(),
                                }),
                                affected,
                            );
                        }
                    }
                }
                crate::models::domain::Side::Sell => {
                    // Sell order: unlock base tokens (unfilled_size)
                    (market.base_ticker, unfilled_size)
                }
            };

            // Unlock the balance
            if let Err(e) = self
                .db
                .unlock_balance(&user_address, &token_to_unlock, amount_to_unlock)
                .await
            {
                return (Err(e), affected);
            }

            // Track unlocked balance
            affected.insert((user_address.clone(), token_to_unlock));
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
            return (Err(e), affected);
        }

        // Broadcast cancellation event
        let _ = self.event_tx.send(EngineEvent::OrderCancelled {
            order_id,
            user_address: user_address.clone(),
        });

        (
            Ok(OrderCancelled {
                order_id: order_id.to_string(),
            }),
            affected,
        )
    }

    /// Handle cancelling all orders for a user
    /// Returns the result and set of affected balances to broadcast
    async fn handle_cancel_all_orders(
        &mut self,
        user_address: String,
        market_id: Option<String>,
    ) -> (Result<OrdersCancelled, ExchangeError>, AffectedBalances) {
        let mut affected = HashSet::new();
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
                let (token_to_unlock, unlock_result) = match cancelled_order.side {
                    crate::models::domain::Side::Buy => {
                        // Buy order: unlock quote tokens (price * unfilled_size)
                        match cancelled_order.price.checked_mul(unfilled_size) {
                            Some(quote_amount) => {
                                let result = self
                                    .db
                                    .unlock_balance(
                                        &user_address,
                                        &market.quote_ticker,
                                        quote_amount,
                                    )
                                    .await;
                                (market.quote_ticker.clone(), result)
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
                        let result = self
                            .db
                            .unlock_balance(&user_address, &market.base_ticker, unfilled_size)
                            .await;
                        (market.base_ticker.clone(), result)
                    }
                };

                // Log unlock failures but continue processing
                if let Err(e) = unlock_result {
                    log::error!("Failed to unlock balance for order {}: {}", order_id, e);
                } else {
                    // Track unlocked balance
                    affected.insert((user_address.clone(), token_to_unlock));
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

        (
            Ok(OrdersCancelled {
                cancelled_order_ids,
                count,
            }),
            affected,
        )
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
        if order.order_type == crate::models::domain::OrderType::Limit
            && !order.price.is_multiple_of(market.tick_size)
        {
            return Err(ExchangeError::InvalidParameter {
                message: format!(
                    "Price {} is not a multiple of tick size {}",
                    order.price, market.tick_size
                ),
            });
        }

        // Validate lot size (size must be multiple of lot_size)
        if !order.size.is_multiple_of(market.lot_size) {
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

    /// Calculate which token and amount to lock for an order
    /// Returns (token_ticker, amount_to_lock)
    async fn calculate_lock_amount(
        &self,
        order: &crate::models::domain::Order,
        market: &crate::models::domain::Market,
    ) -> Result<(String, u128), ExchangeError> {
        match order.side {
            crate::models::domain::Side::Buy => {
                // For buy orders, lock quote tokens
                // quote_amount = (price_atoms * size_atoms) / 10^base_decimals
                let base_token = self.db.get_token(&market.base_ticker).await?;
                let divisor = 10u128.pow(base_token.decimals as u32);
                let quote_amount = order
                    .price
                    .checked_mul(order.size)
                    .and_then(|v| v.checked_div(divisor))
                    .ok_or_else(|| ExchangeError::InvalidParameter {
                        message: "Order value overflow when calculating lock amount".to_string(),
                    })?;
                Ok((market.quote_ticker.clone(), quote_amount))
            }
            crate::models::domain::Side::Sell => {
                // For sell orders, lock base tokens
                Ok((market.base_ticker.clone(), order.size))
            }
        }
    }
}
