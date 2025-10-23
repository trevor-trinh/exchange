use bigdecimal::BigDecimal;

use crate::db::Db;
use crate::models::{db::MarketRow, domain::Market};
use crate::utils::BigDecimalExt;

impl Db {
    /// Create a new market
    pub async fn create_market(
        &self,
        base_ticker: String,
        quote_ticker: String,
        tick_size: u128,
        lot_size: u128,
        min_size: u128,
        maker_fee_bps: i32,
        taker_fee_bps: i32,
    ) -> Result<Market, sqlx::Error> {
        let row = sqlx::query_as!(
            MarketRow,
            "INSERT INTO markets (base_ticker, quote_ticker, tick_size, lot_size, min_size, maker_fee_bps, taker_fee_bps) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id, base_ticker, quote_ticker, tick_size, lot_size, min_size, maker_fee_bps, taker_fee_bps",
            base_ticker,
            quote_ticker,
            BigDecimal::from(tick_size),
            BigDecimal::from(lot_size),
            BigDecimal::from(min_size),
            maker_fee_bps,
            taker_fee_bps
        )
        .fetch_one(&self.postgres)
        .await?;

        Ok(Market {
            id: row.id,
            base_ticker: row.base_ticker,
            quote_ticker: row.quote_ticker,
            tick_size: row.tick_size.to_u128(),
            lot_size: row.lot_size.to_u128(),
            min_size: row.min_size.to_u128(),
            maker_fee_bps: row.maker_fee_bps,
            taker_fee_bps: row.taker_fee_bps,
        })
    }

    /// Get a market by id
    pub async fn get_market(&self, market_id: &str) -> Result<Market, sqlx::Error> {
        let row: MarketRow =
            sqlx::query_as!(MarketRow, "SELECT id, base_ticker, quote_ticker, tick_size, lot_size, min_size, maker_fee_bps, taker_fee_bps FROM markets WHERE id = $1", market_id)
                .fetch_one(&self.postgres)
                .await?;

        Ok(Market {
            id: row.id,
            base_ticker: row.base_ticker,
            quote_ticker: row.quote_ticker,
            tick_size: row.tick_size.to_u128(),
            lot_size: row.lot_size.to_u128(),
            min_size: row.min_size.to_u128(),
            maker_fee_bps: row.maker_fee_bps,
            taker_fee_bps: row.taker_fee_bps,
        })
    }
}
