// executes trades and persists to database

use crate::db::Db;
use crate::errors::Result;
use crate::models::domain::{Market, Match, Order, OrderStatus, Side, Trade};
use chrono::Utc;
use std::collections::HashSet;
use uuid::Uuid;

pub struct Executor;

/// Tracks affected balances that need to be broadcast after request completes
pub type AffectedBalances = HashSet<(String, String)>; // (user_address, token_ticker)

impl Executor {
    /// Execute a vector of matches
    /// - Creates trade records
    /// - Updates order fill status
    /// - Calculates and applies fees
    /// - Unlocks and transfers balances
    /// - Persists everything to database atomically
    /// - Returns the executed trades and affected balances
    pub async fn execute(
        db: Db,
        matches: Vec<Match>,
        taker_order: &Order,
        market: &Market,
    ) -> Result<(Vec<Trade>, AffectedBalances)> {
        if matches.is_empty() {
            return Ok((vec![], HashSet::new()));
        }

        // Get base token decimals for proper quote amount calculation
        let base_token = db.get_token(&market.base_ticker).await?;
        let base_decimals_divisor = 10u128.pow(base_token.decimals as u32);

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
                side: taker_order.side, // Trade side is the taker's side
                timestamp: Utc::now(),
            };

            // Calculate trade value in quote tokens
            // quote_amount = (price_atoms * size_atoms) / 10^base_decimals
            let quote_amount = m
                .price
                .checked_mul(m.size)
                .and_then(|v| v.checked_div(base_decimals_divisor))
                .ok_or_else(|| crate::errors::ExchangeError::InvalidParameter {
                    message: "Trade value overflow or calculation error".to_string(),
                })?;

            // Calculate fees (charged on what each party receives)
            // Buyer receives base tokens (size), pays taker fee if taker, maker fee if maker
            // Seller receives quote tokens (price * size), pays maker fee if maker, taker fee if taker
            let (buyer_fee_bps, seller_fee_bps) = match taker_order.side {
                Side::Buy => {
                    // Buyer is taker, seller is maker
                    (market.taker_fee_bps, market.maker_fee_bps)
                }
                Side::Sell => {
                    // Seller is taker, buyer is maker
                    (market.maker_fee_bps, market.taker_fee_bps)
                }
            };

            // Fee on base tokens (for buyer)
            let buyer_fee = (m.size as i128 * buyer_fee_bps as i128 / 10000) as u128;
            // Fee on quote tokens (for seller)
            let seller_fee = (quote_amount as i128 * seller_fee_bps as i128 / 10000) as u128;

            // Fee recipient address (hardcoded in db schema)
            const FEE_RECIPIENT: &str = "system";

            // Calculate amounts to unlock (what was locked when orders were placed)
            // Buyer locked quote_amount, seller locked size
            let buyer_unlock_amount = quote_amount;
            let seller_unlock_amount = m.size;

            // Unlock the locked amounts for both parties
            db.unlock_balance_tx(
                &mut tx,
                &buyer_address,
                &market.quote_ticker,
                buyer_unlock_amount,
            )
            .await?;
            db.unlock_balance_tx(
                &mut tx,
                &seller_address,
                &market.base_ticker,
                seller_unlock_amount,
            )
            .await?;

            // Transfer base tokens: seller -> buyer (minus buyer's fee)
            db.subtract_balance_tx(&mut tx, &seller_address, &market.base_ticker, m.size)
                .await?;
            let buyer_receives_base = m.size - buyer_fee;
            db.add_balance_tx(
                &mut tx,
                &buyer_address,
                &market.base_ticker,
                buyer_receives_base,
            )
            .await?;

            // Send buyer's fee to fee recipient (base tokens)
            if buyer_fee > 0 {
                db.add_balance_tx(&mut tx, FEE_RECIPIENT, &market.base_ticker, buyer_fee)
                    .await?;
            }

            // Transfer quote tokens: buyer -> seller (minus seller's fee)
            db.subtract_balance_tx(&mut tx, &buyer_address, &market.quote_ticker, quote_amount)
                .await?;
            let seller_receives_quote = quote_amount - seller_fee;
            db.add_balance_tx(
                &mut tx,
                &seller_address,
                &market.quote_ticker,
                seller_receives_quote,
            )
            .await?;

            // Send seller's fee to fee recipient (quote tokens)
            if seller_fee > 0 {
                db.add_balance_tx(&mut tx, FEE_RECIPIENT, &market.quote_ticker, seller_fee)
                    .await?;
            }

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

        // Collect affected balances (to be broadcast by engine after request completes)
        let mut affected_balances = HashSet::new();
        for trade in &trades {
            // Buyer balances (base and quote tokens)
            affected_balances.insert((trade.buyer_address.clone(), market.base_ticker.clone()));
            affected_balances.insert((trade.buyer_address.clone(), market.quote_ticker.clone()));

            // Seller balances (base and quote tokens)
            affected_balances.insert((trade.seller_address.clone(), market.base_ticker.clone()));
            affected_balances.insert((trade.seller_address.clone(), market.quote_ticker.clone()));

            // System fee recipient balances (base and quote tokens)
            affected_balances.insert(("system".to_string(), market.base_ticker.clone()));
            affected_balances.insert(("system".to_string(), market.quote_ticker.clone()));
        }

        // Insert trades into ClickHouse asynchronously (after commit)
        // This is non-critical, so failures won't affect the core trade execution
        for trade in &trades {
            let db_clone = db.clone();
            let trade_clone = trade.clone();
            tokio::spawn(async move {
                let _ = db_clone.insert_trade_to_clickhouse(&trade_clone).await;
            });
        }

        Ok((trades, affected_balances))
    }
}
