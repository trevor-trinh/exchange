CREATE TABLE IF NOT EXISTS exchange.candles (
    market_id TEXT NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    open BIGINT NOT NULL,
    high BIGINT NOT NULL,
    low BIGINT NOT NULL,
    close BIGINT NOT NULL,
    volume BIGINT NOT NULL,
    PRIMARY KEY (market_id, timestamp)
);