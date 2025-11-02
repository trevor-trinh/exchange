use crate::db::Db;
use crate::errors::{ExchangeError, Result};
use crate::models::{
    api::ApiCandle,
    db::{CandleInsertRow, CandleRow, ClickHouseTradeRow},
    domain::{Candle, Trade},
};
use chrono::{DateTime, Utc};

impl Db {
    /// Insert a trade into ClickHouse for tick data
    /// This will automatically trigger the materialized view to create 1m candles
    pub async fn insert_trade_to_clickhouse(&self, trade: &Trade) -> Result<()> {
        let trade_row = ClickHouseTradeRow {
            id: trade.id.to_string(),
            market_id: trade.market_id.clone(),
            buyer_address: trade.buyer_address.clone(),
            seller_address: trade.seller_address.clone(),
            buyer_order_id: trade.buyer_order_id.to_string(),
            seller_order_id: trade.seller_order_id.to_string(),
            price: trade.price,
            size: trade.size,
            side: match trade.side {
                crate::models::domain::Side::Buy => "buy".to_string(),
                crate::models::domain::Side::Sell => "sell".to_string(),
            },
            timestamp: trade.timestamp.timestamp() as u32,
        };

        let mut insert = self
            .clickhouse
            .insert::<ClickHouseTradeRow>("trades")
            .await?;
        insert.write(&trade_row).await?;
        insert.end().await?;

        Ok(())
    }

    /// Insert a candle directly (for backfilling or manual inserts)
    pub async fn insert_candle(
        &self,
        market_id: String,
        timestamp: DateTime<Utc>,
        interval: String,
        ohlcv: (u128, u128, u128, u128, u128), // (open, high, low, close, volume)
    ) -> Result<()> {
        let candle_row = CandleInsertRow {
            market_id,
            timestamp: timestamp.timestamp() as u32,
            trade_time: timestamp.timestamp() as u32, // Use same timestamp for manual inserts
            interval,
            open: ohlcv.0,
            high: ohlcv.1,
            low: ohlcv.2,
            close: ohlcv.3,
            volume: ohlcv.4,
        };

        let mut insert = self.clickhouse.insert::<CandleInsertRow>("candles").await?;
        insert.write(&candle_row).await?;
        insert.end().await?;

        Ok(())
    }

    /// Get candles for a market at a specific interval
    pub async fn get_candles(
        &self,
        market_id: &str,
        interval: &str, // '1m', '5m', '15m', '1h', '1d'
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Candle>> {
        // Aggregate candles from multiple trade rows
        // Use trade_time to determine first (open) and last (close) trades
        let candles = self
            .clickhouse
            .query(
                "SELECT
                market_id,
                timestamp,
                interval,
                argMin(open, trade_time) as open,
                max(high) as high,
                min(low) as low,
                argMax(close, trade_time) as close,
                sum(volume) as volume
            FROM candles
            WHERE market_id = ? AND interval = ? AND timestamp >= ? AND timestamp < ?
            GROUP BY market_id, timestamp, interval
            ORDER BY timestamp ASC",
            )
            .bind(market_id)
            .bind(interval)
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
    /// Returns candles as ApiCandle with timestamp aggregation and optional limit
    pub async fn get_candles_for_api(
        &self,
        market_id: &str,
        interval: &str,
        from: i64,
        to: i64,
        count_back: Option<usize>,
    ) -> Result<Vec<ApiCandle>> {
        // Build the base query with aggregation
        let mut query = format!(
            "SELECT
                toUnixTimestamp(timestamp) as timestamp,
                argMin(open, trade_time) as open,
                max(high) as high,
                min(low) as low,
                argMax(close, trade_time) as close,
                sum(volume) as volume
            FROM exchange.candles
            WHERE market_id = '{}'
              AND interval = '{}'
              AND timestamp >= toDateTime({})
              AND timestamp <= toDateTime({})
            GROUP BY timestamp
            ORDER BY timestamp",
            market_id, interval, from, to
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

    /// Get recent trades for a market (tick data)
    pub async fn get_recent_trades(&self, market_id: &str, limit: u32) -> Result<Vec<Trade>> {
        let limit = std::cmp::min(limit, 1000);

        let trades = self
            .clickhouse
            .query("SELECT id, market_id, buyer_address, seller_address, buyer_order_id, seller_order_id, price, size, timestamp FROM trades WHERE market_id = ? ORDER BY timestamp DESC LIMIT ?")
            .bind(market_id)
            .bind(limit)
            .fetch_all::<ClickHouseTradeRow>()
            .await?;

        Ok(trades
            .into_iter()
            .filter_map(|row| {
                Some(Trade {
                    id: uuid::Uuid::parse_str(&row.id).ok()?,
                    market_id: row.market_id,
                    buyer_address: row.buyer_address,
                    seller_address: row.seller_address,
                    buyer_order_id: uuid::Uuid::parse_str(&row.buyer_order_id).ok()?,
                    seller_order_id: uuid::Uuid::parse_str(&row.seller_order_id).ok()?,
                    price: row.price,
                    size: row.size,
                    side: if row.side == "buy" {
                        crate::models::domain::Side::Buy
                    } else {
                        crate::models::domain::Side::Sell
                    },
                    timestamp: DateTime::from_timestamp(row.timestamp as i64, 0)
                        .unwrap_or(DateTime::UNIX_EPOCH),
                })
            })
            .collect())
    }
}
