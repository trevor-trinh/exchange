use crate::db::Db;
use crate::errors::{ExchangeError, Result};
use crate::models::{
    api::ApiCandle,
    db::{CandleInsertRow, CandleRow},
    domain::{Candle, Trade},
};
use chrono::{DateTime, Timelike, Utc};
use std::collections::HashMap;

impl Db {
    /// Insert trades as 1-minute candles into ClickHouse
    /// ClickHouse materialized views automatically aggregate to larger intervals (5m, 15m, 1h, 1d)
    /// Uses buffered batch inserts for better performance
    pub async fn insert_trades_as_candles(&self, trades: &[Trade]) -> Result<()> {
        if trades.is_empty() {
            return Ok(());
        }

        // Aggregate trades into 1-minute candle buckets only
        // Key: (market_id, bucket_timestamp)
        let mut candle_buckets: HashMap<(String, u32), CandleAggregator> = HashMap::new();

        for trade in trades {
            let timestamp_secs = trade.timestamp.timestamp() as u32;
            let bucket_time = Self::round_to_minute(trade.timestamp);
            let bucket_timestamp = bucket_time.timestamp() as u32;
            let key = (trade.market_id.clone(), bucket_timestamp);

            candle_buckets
                .entry(key)
                .or_insert_with(|| CandleAggregator::new(trade.market_id.clone(), bucket_timestamp))
                .add_trade(trade.price, trade.size, timestamp_secs);
        }

        // Batch insert all 1m candles (materialized views handle the rest)
        if !candle_buckets.is_empty() {
            let mut insert = self
                .clickhouse
                .insert::<CandleInsertRow>("candles_1m")
                .await?;

            for candle in candle_buckets.values() {
                insert.write(&candle.to_insert_row()).await?;
            }

            insert.end().await?;
        }

        Ok(())
    }

    /// Insert a single 1-minute candle directly (for backfilling or manual inserts)
    pub async fn insert_candle(
        &self,
        market_id: String,
        timestamp: DateTime<Utc>,
        ohlcv: (u128, u128, u128, u128, u128), // (open, high, low, close, volume)
    ) -> Result<()> {
        let timestamp_secs = timestamp.timestamp() as u32;
        let candle_row = CandleInsertRow {
            market_id,
            timestamp: timestamp_secs,
            open: ohlcv.0,
            high: ohlcv.1,
            low: ohlcv.2,
            close: ohlcv.3,
            volume: ohlcv.4,
            trade_count: 1,
        };

        let mut insert = self
            .clickhouse
            .insert::<CandleInsertRow>("candles_1m")
            .await?;
        insert.write(&candle_row).await?;
        insert.end().await?;

        Ok(())
    }

    /// Get candles for a market at a specific interval
    /// Queries from the appropriate table/view based on interval
    pub async fn get_candles(
        &self,
        market_id: &str,
        interval: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Candle>> {
        // Determine which table to query based on interval
        let (table_name, use_final) = match interval {
            "1m" => ("candles_1m", true),     // ReplacingMergeTree needs FINAL
            "5m" => ("candles_5m_mv", false), // Materialized views don't support FINAL
            "15m" => ("candles_15m_mv", false),
            "1h" => ("candles_1h_mv", false),
            "1d" => ("candles_1d_mv", false),
            _ => {
                return Err(ExchangeError::InvalidParameter {
                    message: format!("Invalid interval: {}", interval),
                })
            }
        };

        // Use FINAL only for ReplacingMergeTree (candles_1m)
        let final_modifier = if use_final { "FINAL" } else { "" };
        let query = format!(
            "SELECT
                market_id,
                timestamp,
                open,
                high,
                low,
                close,
                volume,
                trade_count
            FROM exchange.{} {}
            WHERE market_id = ? AND timestamp >= ? AND timestamp < ?
            ORDER BY timestamp ASC",
            table_name, final_modifier
        );

        let candles = self
            .clickhouse
            .query(&query)
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

    /// Get candles for API with support for countBack parameter
    pub async fn get_candles_for_api(
        &self,
        market_id: &str,
        interval: &str,
        from: i64,
        to: i64,
        count_back: Option<usize>,
    ) -> Result<Vec<ApiCandle>> {
        // Determine which table to query based on interval
        let (table_name, use_final) = match interval {
            "1m" => ("candles_1m", true),     // ReplacingMergeTree needs FINAL
            "5m" => ("candles_5m_mv", false), // Materialized views don't support FINAL
            "15m" => ("candles_15m_mv", false),
            "1h" => ("candles_1h_mv", false),
            "1d" => ("candles_1d_mv", false),
            _ => {
                return Err(ExchangeError::InvalidParameter {
                    message: format!("Invalid interval: {}", interval),
                })
            }
        };

        // Build the base query - data is pre-aggregated in materialized views
        // Use FINAL only for ReplacingMergeTree (candles_1m)
        let final_modifier = if use_final { "FINAL" } else { "" };
        let mut query = format!(
            "SELECT
                toUnixTimestamp(timestamp) as timestamp,
                open,
                high,
                low,
                close,
                volume
            FROM exchange.{} {}
            WHERE market_id = '{}'
              AND timestamp >= toDateTime({})
              AND timestamp <= toDateTime({})
            ORDER BY timestamp",
            table_name, final_modifier, market_id, from, to
        );

        // Handle countBack: limit to N most recent bars
        if let Some(count_back) = count_back {
            if count_back > 0 {
                // Get the last N bars by ordering DESC and limiting
                query = format!("{} DESC LIMIT {}", query, count_back);
            } else {
                query = format!("{} ASC", query);
            }
        } else {
            query = format!("{} ASC", query);
        }

        let mut candles: Vec<ApiCandle> = self
            .clickhouse
            .query(&query)
            .fetch_all()
            .await
            .map_err(ExchangeError::ClickHouse)?;

        // If we used DESC for countBack, reverse to get ascending order
        if count_back.is_some() && count_back.unwrap() > 0 {
            candles.reverse();
        }

        Ok(candles)
    }

    /// Get recent trades is no longer supported since we don't store tick data
    /// Returns empty vec for backwards compatibility
    pub async fn get_recent_trades(&self, _market_id: &str, _limit: u32) -> Result<Vec<Trade>> {
        // Tick data is no longer stored - return empty
        Ok(Vec::new())
    }

    // Helper function to round timestamps to 1-minute boundary
    fn round_to_minute(dt: DateTime<Utc>) -> DateTime<Utc> {
        dt.with_second(0).unwrap().with_nanosecond(0).unwrap()
    }
}

/// Helper struct to aggregate trades into OHLCV candles (1-minute only)
#[derive(Debug)]
struct CandleAggregator {
    market_id: String,
    timestamp: u32,
    open: u128,
    high: u128,
    low: u128,
    close: u128,
    volume: u128,
    trade_count: u32,
    last_trade_time: u32,
}

impl CandleAggregator {
    fn new(market_id: String, timestamp: u32) -> Self {
        Self {
            market_id,
            timestamp,
            open: 0,
            high: 0,
            low: u128::MAX,
            close: 0,
            volume: 0,
            trade_count: 0,
            last_trade_time: 0,
        }
    }

    fn add_trade(&mut self, price: u128, size: u128, trade_time: u32) {
        // First trade sets the open price
        if self.trade_count == 0 {
            self.open = price;
        }

        // Update high and low
        if price > self.high {
            self.high = price;
        }
        if price < self.low {
            self.low = price;
        }

        // Last trade in time order sets the close
        if trade_time >= self.last_trade_time {
            self.close = price;
            self.last_trade_time = trade_time;
        }

        // Accumulate volume
        self.volume += size;
        self.trade_count += 1;
    }

    fn to_insert_row(&self) -> CandleInsertRow {
        CandleInsertRow {
            market_id: self.market_id.clone(),
            timestamp: self.timestamp,
            open: self.open,
            high: self.high,
            low: self.low,
            close: self.close,
            volume: self.volume,
            trade_count: self.trade_count,
        }
    }
}
