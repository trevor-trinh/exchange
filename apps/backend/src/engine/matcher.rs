// matches orders using price-time priority

use crate::engine::orderbook::Orderbook;
use crate::models::domain::{Match, Order, OrderType, Side};

pub struct Matcher;

impl Matcher {
    /// Match a taker order against the orderbook
    /// Returns a vector of matches (maker orders matched with taker order)
    /// Does NOT modify the orderbook - just reads it
    pub fn match_order(taker_order: &Order, orderbook: &Orderbook) -> Vec<Match> {
        let mut matches = Vec::new();
        let mut remaining_size = taker_order.size - taker_order.filled_size;

        // Iterate through price levels in order (BTreeMap is sorted)
        // For asks: ascending (lowest price first)
        // For bids: descending (highest price first) - need to reverse
        let level_iter: Box<dyn Iterator<Item = (&u128, &_)>> = match taker_order.side {
            Side::Buy => Box::new(orderbook.asks.iter()), // Lowest ask first
            Side::Sell => Box::new(orderbook.bids.iter().rev()), // Highest bid first
        };

        for (price, orders) in level_iter {
            if remaining_size == 0 {
                break;
            }

            // Check if this price level can match
            if !Self::can_match_price(taker_order, *price) {
                break; // No more matches possible at this or worse prices
            }

            // Match against orders at this level (FIFO - time priority)
            for maker_order in orders {
                if remaining_size == 0 {
                    break;
                }

                // Skip self-trading: don't match orders from the same user
                if maker_order.user_address == taker_order.user_address {
                    continue;
                }

                // Calculate match size (minimum of what's needed and what's available)
                let maker_remaining = maker_order.size - maker_order.filled_size;
                let match_size = remaining_size.min(maker_remaining);

                matches.push(Match {
                    maker_order: maker_order.clone(),
                    price: *price, // Match at maker's price (price-time priority)
                    size: match_size,
                });

                remaining_size -= match_size;
            }
        }

        matches
    }

    /// Check if a taker order can match at the given maker price
    fn can_match_price(taker: &Order, maker_price: u128) -> bool {
        match (taker.side, taker.order_type) {
            // Buy limit: can match if willing to pay >= maker's asking price
            // considered a taker order if above lowest ask
            (Side::Buy, OrderType::Limit) => taker.price >= maker_price,
            // Buy market: match at any price
            (Side::Buy, OrderType::Market) => true,
            // Sell limit: can match if willing to accept <= maker's bid price
            // considered a taker order if below highest bid
            (Side::Sell, OrderType::Limit) => taker.price <= maker_price,
            // Sell market: match at any price
            (Side::Sell, OrderType::Market) => true,
        }
    }
}
