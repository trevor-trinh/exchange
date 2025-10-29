-- Trades table for tick data (raw trades from the matching engine)
CREATE TABLE IF NOT EXISTS exchange.trades (
    id String,
    market_id String,
    buyer_address String,
    seller_address String,
    buyer_order_id String,
    seller_order_id String,
    price UInt128,
    size UInt128,
    timestamp DateTime
) ENGINE = MergeTree()
ORDER BY (market_id, timestamp)
PRIMARY KEY (market_id, timestamp);

-- Candles table for aggregated OHLCV data
-- interval values: '1m', '5m', '15m', '1h', '1d'
CREATE TABLE IF NOT EXISTS exchange.candles (
    market_id String,
    timestamp DateTime,
    interval String,
    open UInt128,
    high UInt128,
    low UInt128,
    close UInt128,
    volume UInt128
) ENGINE = MergeTree()
ORDER BY (market_id, interval, timestamp)
PRIMARY KEY (market_id, interval, timestamp);

-- Materialized view for 1-minute candles from trades
-- This auto-aggregates trades into 1-minute OHLCV bars
CREATE MATERIALIZED VIEW IF NOT EXISTS exchange.candles_1m_mv
TO exchange.candles
AS SELECT
    market_id,
    minute_bucket as timestamp,
    '1m' as interval,
    argMin(price, trade_time) as open,
    max(price) as high,
    min(price) as low,
    argMax(price, trade_time) as close,
    sum(size) as volume
FROM (
    SELECT
        market_id,
        price,
        size,
        timestamp as trade_time,
        toStartOfMinute(timestamp) as minute_bucket
    FROM exchange.trades
)
GROUP BY market_id, minute_bucket;
