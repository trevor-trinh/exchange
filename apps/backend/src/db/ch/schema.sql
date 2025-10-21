CREATE TABLE IF NOT EXISTS exchange.candles (
    market_id String,
    timestamp DateTime,
    open UInt128,
    high UInt128,
    low UInt128,
    close UInt128,
    volume UInt128
) ENGINE = MergeTree()
ORDER BY (market_id, timestamp)
PRIMARY KEY (market_id, timestamp);
