-- Create database if it doesn't exist
CREATE DATABASE IF NOT EXISTS exchange;

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
    side String,
    timestamp DateTime
) ENGINE = MergeTree()
ORDER BY (market_id, timestamp)
PRIMARY KEY (market_id, timestamp);

-- Candles table for aggregated OHLCV data
-- interval values: '1m', '5m', '15m', '1h', '1d'
-- Materialized views insert one row per trade, queries must GROUP BY to aggregate
CREATE TABLE IF NOT EXISTS exchange.candles (
    market_id String,
    timestamp DateTime,      -- Bucket timestamp (start of interval)
    trade_time DateTime,     -- Original trade timestamp (for ordering)
    interval String,
    open UInt128,
    high UInt128,
    low UInt128,
    close UInt128,
    volume UInt128
) ENGINE = MergeTree()
ORDER BY (market_id, interval, timestamp, trade_time)
PRIMARY KEY (market_id, interval, timestamp);

CREATE MATERIALIZED VIEW IF NOT EXISTS exchange.candles_1m_mv
TO exchange.candles
AS SELECT
    market_id,
    toStartOfMinute(t.timestamp) as timestamp,
    t.timestamp as trade_time,
    '1m' as interval,
    t.price as open,
    t.price as high,
    t.price as low,
    t.price as close,
    t.size as volume
FROM exchange.trades AS t;

CREATE MATERIALIZED VIEW IF NOT EXISTS exchange.candles_5m_mv
TO exchange.candles
AS SELECT
    market_id,
    toStartOfInterval(t.timestamp, INTERVAL 5 MINUTE) as timestamp,
    t.timestamp as trade_time,
    '5m' as interval,
    t.price as open,
    t.price as high,
    t.price as low,
    t.price as close,
    t.size as volume
FROM exchange.trades AS t;

CREATE MATERIALIZED VIEW IF NOT EXISTS exchange.candles_15m_mv
TO exchange.candles
AS SELECT
    market_id,
    toStartOfInterval(t.timestamp, INTERVAL 15 MINUTE) as timestamp,
    t.timestamp as trade_time,
    '15m' as interval,
    t.price as open,
    t.price as high,
    t.price as low,
    t.price as close,
    t.size as volume
FROM exchange.trades AS t;

CREATE MATERIALIZED VIEW IF NOT EXISTS exchange.candles_1h_mv
TO exchange.candles
AS SELECT
    market_id,
    toStartOfHour(t.timestamp) as timestamp,
    t.timestamp as trade_time,
    '1h' as interval,
    t.price as open,
    t.price as high,
    t.price as low,
    t.price as close,
    t.size as volume
FROM exchange.trades AS t;

CREATE MATERIALIZED VIEW IF NOT EXISTS exchange.candles_1d_mv
TO exchange.candles
AS SELECT
    market_id,
    toStartOfDay(t.timestamp) as timestamp,
    t.timestamp as trade_time,
    '1d' as interval,
    t.price as open,
    t.price as high,
    t.price as low,
    t.price as close,
    t.size as volume
FROM exchange.trades AS t;
