CREATE TABLE IF NOT EXISTS exchange.candles (
    market_id TEXT NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    open NUMERIC NOT NULL,
    high NUMERIC NOT NULL,
    low NUMERIC NOT NULL,
    close NUMERIC NOT NULL,
    volume NUMERIC NOT NULL,
    PRIMARY KEY (market_id, timestamp)
);