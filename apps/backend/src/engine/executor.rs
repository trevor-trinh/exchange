// executes trades and persists to database

use crate::db::Db;
use crate::errors::Result;
use crate::models::domain::{Match, Order, OrderStatus, Side, Trade};
use chrono::Utc;
use uuid::Uuid;

pub struct Executor;

impl Executor {
    /// Execute a vector of matches
    /// - Creates trade records
    /// - Updates order fill status
    /// - Persists everything to database atomically
    /// Returns the executed trades
    pub async fn execute(db: Db, matches: Vec<Match>, taker_order: &Order) -> Result<Vec<Trade>> {
        if matches.is_empty() {
            return Ok(vec![]);
        }

        // Begin transaction for atomic execution
        let mut tx = db.begin_transaction().await?;
        let mut trades = Vec::new();

        // Process each match within transaction
        for m in &matches {
            let maker_order = &m.maker_order;

            // Determine buyer and seller based on sides
            let (buyer_address, seller_address, buyer_order_id, seller_order_id) =
                match taker_order.side {
                    Side::Buy => (
                        taker_order.user_address.clone(),
                        maker_order.user_address.clone(),
                        taker_order.id,
                        maker_order.id,
                    ),
                    Side::Sell => (
                        maker_order.user_address.clone(),
                        taker_order.user_address.clone(),
                        maker_order.id,
                        taker_order.id,
                    ),
                };

            // Create trade record
            let trade = Trade {
                id: Uuid::new_v4(),
                market_id: taker_order.market_id.clone(),
                buyer_address: buyer_address.clone(),
                seller_address: seller_address.clone(),
                buyer_order_id,
                seller_order_id,
                price: m.price,
                size: m.size,
                timestamp: Utc::now(),
            };

            // Update maker order fill status (in transaction)
            let maker_new_filled = maker_order.filled_size + m.size;
            let maker_status = if maker_new_filled >= maker_order.size {
                OrderStatus::Filled
            } else {
                OrderStatus::PartiallyFilled
            };
            db.update_order_fill_tx(&mut tx, maker_order.id, maker_new_filled, maker_status)
                .await?;

            // Insert trade into PostgreSQL (in transaction)
            db.create_trade_tx(&mut tx, &trade).await?;

            trades.push(trade);
        }

        // Update taker order fill status (in transaction)
        let taker_total_filled: u128 = matches.iter().map(|m| m.size).sum();
        let taker_new_filled = taker_order.filled_size + taker_total_filled;
        let taker_status = if taker_new_filled >= taker_order.size {
            OrderStatus::Filled
        } else if taker_new_filled > 0 {
            OrderStatus::PartiallyFilled
        } else {
            OrderStatus::Pending
        };
        db.update_order_fill_tx(&mut tx, taker_order.id, taker_new_filled, taker_status)
            .await?;

        // Commit transaction - all or nothing!
        tx.commit().await?;

        // Insert trades into ClickHouse asynchronously (after commit)
        // This is non-critical, so failures won't affect the core trade execution
        for trade in &trades {
            let db_clone = db.clone();
            let trade_clone = trade.clone();
            tokio::spawn(async move {
                let _ = db_clone.insert_trade_to_clickhouse(&trade_clone).await;
            });
        }

        Ok(trades)
    }
}
