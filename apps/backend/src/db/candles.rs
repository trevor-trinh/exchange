use crate::db::Db;
use crate::models::{db::CandleRow, domain::Candle};
use chrono::{DateTime, Utc};

// TODO: this is probably going to need to insert a trade not a candle
impl Db {
    /// Insert a new candle
    pub async fn insert_candle(
        &self,
        market_id: String,
        timestamp: DateTime<Utc>,
        ohlcv: (u128, u128, u128, u128, u128), // (open, high, low, close, volume)
    ) -> Result<(), clickhouse::error::Error> {
        let candle_row = CandleRow {
            market_id,
            timestamp: timestamp.timestamp() as u32,
            open: ohlcv.0,
            high: ohlcv.1,
            low: ohlcv.2,
            close: ohlcv.3,
            volume: ohlcv.4,
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
            .query("SELECT market_id, timestamp, open, high, low, close, volume FROM candles WHERE market_id = ? AND timestamp >= ? AND timestamp < ? ORDER BY timestamp ASC")
            .bind(market_id)
            .bind(start.timestamp() as u32)
            .bind(end.timestamp() as u32)
            .fetch_all::<CandleRow>()
            .await?;

        Ok(candles
            .into_iter()
            .map(|row| Candle {
                market_id: row.market_id,
                timestamp: DateTime::from_timestamp(row.timestamp as i64, 0)
                    .unwrap_or(DateTime::UNIX_EPOCH),
                open: row.open,
                high: row.high,
                low: row.low,
                close: row.close,
                volume: row.volume,
            })
            .collect())
    }
}
