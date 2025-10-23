use crate::db::Db;
use crate::models::{db::CandleRow, domain::Candle};
use chrono::{DateTime, Utc};

// - insert_candle(client: &Client, candle: Candle) -> Result<(), clickhouse::error::Error>
// - get_candles(client: &Client, market_id: &str, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Candle>, clickhouse::error::Error>

// TODO: this is probably going to need to insert a trade not a candle
impl Db {
    /// Insert a new candle
    pub async fn insert_candle(
        &self,
        market_id: String,
        timestamp: DateTime<Utc>,
        open: u128,
        high: u128,
        low: u128,
        close: u128,
        volume: u128,
    ) -> Result<(), clickhouse::error::Error> {
        let candle_row = CandleRow {
            market_id,
            timestamp,
            open,
            high,
            low,
            close,
            volume,
        };

        let mut insert = self.clickhouse.insert::<CandleRow>("candles").await?;
        insert.write(&candle_row).await?;
        insert.end().await?;

        Ok(())
    }

    /// Get candles for a market
    pub async fn get_candles(
        &self,
        market_id: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Candle>, clickhouse::error::Error> {
        let candles = self
            .clickhouse
            .query("SELECT market_id, timestamp, open, high, low, close, volume FROM candles WHERE market_id = ? AND timestamp >= ? AND timestamp <= ?")
            .bind(market_id)
            .bind(start)
            .bind(end)
            .fetch_all::<CandleRow>()
            .await?;

        Ok(candles
            .into_iter()
            .map(|row| Candle {
                market_id: row.market_id,
                timestamp: row.timestamp,
                open: row.open,
                high: row.high,
                low: row.low,
                close: row.close,
                volume: row.volume,
            })
            .collect())
    }
}
