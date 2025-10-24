use bigdecimal::BigDecimal;

use crate::db::Db;
use crate::errors::{ExchangeError, Result};
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
    ) -> Result<Market> {
        // Check if both tokens exist before creating the market
        self.get_token(&base_ticker)
            .await
            .map_err(|_| ExchangeError::TokenNotFound {
                ticker: base_ticker.clone(),
            })?;
        self.get_token(&quote_ticker)
            .await
            .map_err(|_| ExchangeError::TokenNotFound {
                ticker: quote_ticker.clone(),
            })?;

        // Manually construct the market ID as "base_ticker/quote_ticker"
        let id = format!("{}/{}", base_ticker, quote_ticker);

        let row = sqlx::query_as!(
            MarketRow,
            "INSERT INTO markets (id, base_ticker, quote_ticker, tick_size, lot_size, min_size, maker_fee_bps, taker_fee_bps) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id, base_ticker, quote_ticker, tick_size, lot_size, min_size, maker_fee_bps, taker_fee_bps",
            id,
            base_ticker,
            quote_ticker,
            BigDecimal::from(tick_size),
            BigDecimal::from(lot_size),
            BigDecimal::from(min_size),
            maker_fee_bps,
            taker_fee_bps
        )
        .fetch_one(&self.postgres)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_err) if db_err.constraint().is_some() => {
                ExchangeError::MarketAlreadyExists { market_id: id }
            }
            _ => ExchangeError::Database(e),
        })?;

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
    pub async fn get_market(&self, market_id: &str) -> Result<Market> {
        let row: MarketRow =
            sqlx::query_as!(MarketRow, "SELECT id, base_ticker, quote_ticker, tick_size, lot_size, min_size, maker_fee_bps, taker_fee_bps FROM markets WHERE id = $1", market_id)
                .fetch_one(&self.postgres)
                .await
                .map_err(ExchangeError::from)?;

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

    /// List all markets
    pub async fn list_markets(&self) -> Result<Vec<Market>> {
        let rows = sqlx::query_as!(
            MarketRow,
            "SELECT id, base_ticker, quote_ticker, tick_size, lot_size, min_size, maker_fee_bps, taker_fee_bps FROM markets ORDER BY id"
        )
        .fetch_all(&self.postgres)
        .await
        .map_err(ExchangeError::from)?;

        Ok(rows
            .into_iter()
            .map(|row| Market {
                id: row.id,
                base_ticker: row.base_ticker,
                quote_ticker: row.quote_ticker,
                tick_size: row.tick_size.to_u128(),
                lot_size: row.lot_size.to_u128(),
                min_size: row.min_size.to_u128(),
                maker_fee_bps: row.maker_fee_bps,
                taker_fee_bps: row.taker_fee_bps,
            })
            .collect())
    }
}
