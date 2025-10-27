// holds orderbook for all markets

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::VecDeque;

use crate::errors::{ExchangeError, Result};
use crate::models::domain::{Match, Order, OrderStatus, OrderbookLevel, OrderbookSnapshot, Side};
use chrono::Utc;
use uuid::Uuid;

pub struct Orderbooks {
    // market id -> orderbook
    orderbooks: HashMap<String, Orderbook>,
}

impl Orderbooks {
    pub fn new() -> Self {
        Self {
            orderbooks: HashMap::new(),
        }
    }

    /// Get or create a mutable reference to an orderbook for a market
    /// Creates the orderbook if it doesn't exist
    pub fn get_or_create(&mut self, market_id: &str) -> &mut Orderbook {
        self.orderbooks
            .entry(market_id.to_string())
            .or_insert_with(|| Orderbook::new(market_id.to_string()))
    }

    /// Cancel an order across all markets
    /// Returns the cancelled order if found and ownership is verified
    pub fn cancel_order(&mut self, order_id: Uuid, user_address: &str) -> Result<Order> {
        // Search all markets for the order
        for orderbook in self.orderbooks.values_mut() {
            if let Some(order) = orderbook.remove_order(order_id) {
                // Verify ownership
                if order.user_address != user_address {
                    // Put the order back since ownership check failed
                    orderbook.add_order(order);
                    return Err(ExchangeError::OrderNotFound); // Return not found for security
                }
                return Ok(order);
            }
        }

        Err(ExchangeError::OrderNotFound)
    }

    /// Generate snapshots for all markets
    pub fn snapshots(&self) -> Vec<OrderbookSnapshot> {
        self.orderbooks
            .values()
            .map(|orderbook| orderbook.snapshot())
            .collect()
    }
}

pub struct Orderbook {
    pub market_id: String,
    pub bids: BTreeMap<u128, VecDeque<Order>>, // Descending price (highest first)
    pub asks: BTreeMap<u128, VecDeque<Order>>, // Ascending price (lowest first)
}

impl Orderbook {
    pub fn new(market_id: String) -> Self {
        Self {
            market_id,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    /// Apply matches to the orderbook
    /// - Updates filled amounts on maker orders
    /// - Removes fully filled orders
    /// - Adds remaining taker order if not fully filled
    pub fn apply_matches(&mut self, taker_order: &Order, matches: &[Match]) {
        // Update maker orders that were matched
        for m in matches {
            self.update_order_fill(m.maker_order_id, m.size);
        }

        // Add taker order to book if not fully filled
        let total_matched: u128 = matches.iter().map(|m| m.size).sum();
        if total_matched < taker_order.size {
            let mut remaining_order = taker_order.clone();
            remaining_order.filled_size = total_matched;
            remaining_order.status = if total_matched > 0 {
                OrderStatus::PartiallyFilled
            } else {
                OrderStatus::Pending
            };
            self.add_order(remaining_order);
        }
    }

    /// Update an order's filled amount, remove if fully filled
    fn update_order_fill(&mut self, order_id: Uuid, fill_size: u128) {
        // Search both bids and asks
        for (_, orders) in self.bids.iter_mut().chain(self.asks.iter_mut()) {
            if let Some(pos) = orders.iter().position(|o| o.id == order_id) {
                let order = &mut orders[pos];
                order.filled_size += fill_size;
                order.updated_at = Utc::now();

                // Remove if fully filled
                if order.filled_size >= order.size {
                    order.status = OrderStatus::Filled;
                    orders.remove(pos);
                }
                return;
            }
        }
    }

    /// Add an order to the orderbook
    pub fn add_order(&mut self, order: Order) {
        let levels = match order.side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };

        levels
            .entry(order.price)
            .or_insert_with(VecDeque::new)
            .push_back(order);
    }

    /// Remove an order from the orderbook by ID (for cancellation)
    pub fn remove_order(&mut self, order_id: Uuid) -> Option<Order> {
        // Search bids
        for (_, orders) in self.bids.iter_mut() {
            if let Some(pos) = orders.iter().position(|o| o.id == order_id) {
                return Some(orders.remove(pos).unwrap());
            }
        }

        // Search asks
        for (_, orders) in self.asks.iter_mut() {
            if let Some(pos) = orders.iter().position(|o| o.id == order_id) {
                return Some(orders.remove(pos).unwrap());
            }
        }

        None
    }

    /// Get maker orders from a list of matches
    /// Returns orders in the same order as matches
    pub fn get_maker_orders(&self, matches: &[Match]) -> Vec<Order> {
        matches
            .iter()
            .filter_map(|m| {
                // Find maker order in orderbook
                self.bids
                    .values()
                    .chain(self.asks.values())
                    .flatten()
                    .find(|o| o.id == m.maker_order_id)
                    .cloned()
            })
            .collect()
    }

    /// Generate a snapshot of the current orderbook state
    pub fn snapshot(&self) -> OrderbookSnapshot {
        // Aggregate bids by price level (highest to lowest)
        let bids: Vec<OrderbookLevel> = self
            .bids
            .iter()
            .rev() // BTreeMap is ascending, we want descending for bids
            .map(|(price, orders)| {
                let total_size: u128 = orders.iter().map(|o| o.size - o.filled_size).sum();
                OrderbookLevel {
                    price: *price,
                    size: total_size,
                }
            })
            .filter(|level| level.size > 0) // Only include levels with size
            .collect();

        // Aggregate asks by price level (lowest to highest)
        let asks: Vec<OrderbookLevel> = self
            .asks
            .iter()
            .map(|(price, orders)| {
                let total_size: u128 = orders.iter().map(|o| o.size - o.filled_size).sum();
                OrderbookLevel {
                    price: *price,
                    size: total_size,
                }
            })
            .filter(|level| level.size > 0) // Only include levels with size
            .collect();

        OrderbookSnapshot {
            market_id: self.market_id.clone(),
            bids,
            asks,
            timestamp: Utc::now(),
        }
    }
}
