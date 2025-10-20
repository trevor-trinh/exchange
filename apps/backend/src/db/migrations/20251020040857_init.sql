CREATE TABLE IF NOT EXISTS users (
    address TEXT PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS tokens (
    ticker TEXT PRIMARY KEY,
    decimals INT NOT NULL,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS markets (
    base_ticker TEXT NOT NULL REFERENCES tokens(ticker),
    quote_ticker TEXT NOT NULL REFERENCES tokens(ticker),
    PRIMARY KEY (base_ticker, quote_ticker)
);

CREATE TABLE IF NOT EXISTS orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_address TEXT NOT NULL REFERENCES users(address)
);

CREATE TABLE IF NOT EXISTS trades (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_address TEXT NOT NULL REFERENCES users(address),
    order_id UUID NOT NULL REFERENCES orders(id)
);

CREATE TABLE IF NOT EXISTS balances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_address TEXT NOT NULL REFERENCES users(address),
    token_ticker TEXT NOT NULL REFERENCES tokens(ticker),
    amount BIGINT NOT NULL,
    open_interest BIGINT NOT NULL
);

