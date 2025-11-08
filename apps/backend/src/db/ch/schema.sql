-- Create database if it doesn't exist
CREATE DATABASE IF NOT EXISTS exchange;

-- OPTIMIZED CANDLE ARCHITECTURE
-- Store only 1-minute candles, ClickHouse materialized views auto-aggregate to larger intervals
-- Simplest and cleanest: Rust does 1 aggregation, ClickHouse handles the rest

-- 1. Base table: Store ONLY 1-minute candles (single source of truth)
CREATE TABLE IF NOT EXISTS exchange.candles_1m (
    market_id LowCardinality(String),
    timestamp DateTime CODEC(DoubleDelta, ZSTD(3)),
    open UInt128 CODEC(ZSTD(3)),
    high UInt128 CODEC(ZSTD(3)),
    low UInt128 CODEC(ZSTD(3)),
    close UInt128 CODEC(ZSTD(3)),
    volume UInt128 CODEC(ZSTD(3)),
    trade_count UInt32 CODEC(ZSTD(3))
) ENGINE = ReplacingMergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (market_id, timestamp)
PRIMARY KEY (market_id, timestamp)
TTL timestamp + INTERVAL 90 DAY
SETTINGS index_granularity = 8192;

-- 2. Materialized view: 5-minute candles (auto-aggregated from 1m)
CREATE MATERIALIZED VIEW IF NOT EXISTS exchange.candles_5m_mv
ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (market_id, timestamp)
PRIMARY KEY (market_id, timestamp)
TTL timestamp + INTERVAL 180 DAY
AS SELECT
    market_id,
    toStartOfInterval(candles_1m.timestamp, INTERVAL 5 MINUTE) as timestamp,
    argMin(candles_1m.open, candles_1m.timestamp) as open,
    max(candles_1m.high) as high,
    min(candles_1m.low) as low,
    argMax(candles_1m.close, candles_1m.timestamp) as close,
    sum(candles_1m.volume) as volume,
    sum(candles_1m.trade_count) as trade_count
FROM exchange.candles_1m
GROUP BY
    market_id,
    toStartOfInterval(candles_1m.timestamp, INTERVAL 5 MINUTE);

-- 3. Materialized view: 15-minute candles
CREATE MATERIALIZED VIEW IF NOT EXISTS exchange.candles_15m_mv
ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (market_id, timestamp)
PRIMARY KEY (market_id, timestamp)
TTL timestamp + INTERVAL 365 DAY
AS SELECT
    market_id,
    toStartOfInterval(candles_1m.timestamp, INTERVAL 15 MINUTE) as timestamp,
    argMin(candles_1m.open, candles_1m.timestamp) as open,
    max(candles_1m.high) as high,
    min(candles_1m.low) as low,
    argMax(candles_1m.close, candles_1m.timestamp) as close,
    sum(candles_1m.volume) as volume,
    sum(candles_1m.trade_count) as trade_count
FROM exchange.candles_1m
GROUP BY
    market_id,
    toStartOfInterval(candles_1m.timestamp, INTERVAL 15 MINUTE);

-- 4. Materialized view: 1-hour candles
CREATE MATERIALIZED VIEW IF NOT EXISTS exchange.candles_1h_mv
ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (market_id, timestamp)
PRIMARY KEY (market_id, timestamp)
TTL timestamp + INTERVAL 730 DAY
AS SELECT
    market_id,
    toStartOfInterval(candles_1m.timestamp, INTERVAL 1 HOUR) as timestamp,
    argMin(candles_1m.open, candles_1m.timestamp) as open,
    max(candles_1m.high) as high,
    min(candles_1m.low) as low,
    argMax(candles_1m.close, candles_1m.timestamp) as close,
    sum(candles_1m.volume) as volume,
    sum(candles_1m.trade_count) as trade_count
FROM exchange.candles_1m
GROUP BY
    market_id,
    toStartOfInterval(candles_1m.timestamp, INTERVAL 1 HOUR);

-- 5. Materialized view: 1-day candles
CREATE MATERIALIZED VIEW IF NOT EXISTS exchange.candles_1d_mv
ENGINE = MergeTree()
PARTITION BY toYear(timestamp)
ORDER BY (market_id, timestamp)
PRIMARY KEY (market_id, timestamp)
TTL timestamp + INTERVAL 3650 DAY
AS SELECT
    market_id,
    toStartOfDay(candles_1m.timestamp) as timestamp,
    argMin(candles_1m.open, candles_1m.timestamp) as open,
    max(candles_1m.high) as high,
    min(candles_1m.low) as low,
    argMax(candles_1m.close, candles_1m.timestamp) as close,
    sum(candles_1m.volume) as volume,
    sum(candles_1m.trade_count) as trade_count
FROM exchange.candles_1m
GROUP BY
    market_id,
    toStartOfDay(candles_1m.timestamp);
