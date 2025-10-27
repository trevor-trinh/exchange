// executes trades and persists to database

use crate::db::Db;
use crate::errors::Result;
use crate::models::domain::{Match, Order, Side, Trade};
use chrono::Utc;
use uuid::Uuid;

pub struct Executor;

impl Executor {
    /// Execute a vector of matches
    /// - Creates trade records
    /// - Updates order fill status (TODO)
    /// - Transfers balances (TODO)
    /// - Persists everything in a transaction (TODO)
    /// Returns the executed trades
    pub async fn execute(
        db: Db,
        matches: Vec<Match>,
        taker_order: &Order,
        maker_orders: &[Order],
    ) -> Result<Vec<Trade>> {
        if matches.is_empty() {
            return Ok(vec![]);
        }

        // TODO: Start transaction
        // let mut tx = self.db.postgres.begin().await?;

        let mut trades = Vec::new();

        // Get market info for fee calculation (will be used for settle_trade)
        let _market = db.get_market(&taker_order.market_id).await?;

        for m in matches {
            // Find the maker order
            let maker_order = maker_orders
                .iter()
                .find(|o| o.id == m.maker_order_id)
                .expect("Maker order must exist");

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

            // TODO: Update orders in database
            // self.update_order_fill(&mut tx, m.maker_order_id, m.size).await?;
            // self.update_order_fill(&mut tx, m.taker_order_id, m.size).await?;

            // TODO: Settle trade (transfer balances with fees)
            // self.settle_trade(&mut tx, &trade, &market).await?;

            // TODO: Insert trade into database
            // self.insert_trade(&mut tx, &trade).await?;

            trades.push(trade);
        }

        // TODO: Commit transaction
        // tx.commit().await?;

        Ok(trades)
    }

    // TODO: Implement these helper methods once DB methods are available

    // async fn update_order_fill(
    //     &self,
    //     tx: &mut Transaction<'_, Postgres>,
    //     order_id: Uuid,
    //     fill_size: u128,
    // ) -> Result<()> {
    //     // Update order.filled_size
    //     // Update order.status if fully filled
    //     // Update order.updated_at
    //     Ok(())
    // }

    // async fn settle_trade(
    //     &self,
    //     tx: &mut Transaction<'_, Postgres>,
    //     trade: &Trade,
    //     market: &Market,
    // ) -> Result<()> {
    //     // Calculate amounts
    //     let base_amount = trade.size;
    //     let quote_amount = trade.size * trade.price; // TODO: Handle decimals properly
    //
    //     // Calculate fees
    //     let maker_fee = (quote_amount * market.maker_fee_bps as u128) / 10_000;
    //     let taker_fee = (quote_amount * market.taker_fee_bps as u128) / 10_000;
    //
    //     // Transfer base token: seller -> buyer
    //     // Transfer quote token: buyer -> seller (minus fees)
    //     // Collect fees to fee account
    //
    //     Ok(())
    // }

    // async fn insert_trade(
    //     &self,
    //     tx: &mut Transaction<'_, Postgres>,
    //     trade: &Trade,
    // ) -> Result<()> {
    //     // Insert trade into database
    //     Ok(())
    // }
}
