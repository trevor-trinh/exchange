CREATE TABLE IF NOT EXISTS users (
    address TEXT PRIMARY KEY,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS tokens (
    ticker TEXT PRIMARY KEY,
    decimals INT NOT NULL,
    name TEXT NOT NULL
);

CREATE TYPE side AS ENUM ('buy', 'sell');
CREATE TYPE order_type AS ENUM ('limit', 'market');
CREATE TYPE order_status AS ENUM ('pending', 'filled', 'partially_filled', 'cancelled');

CREATE TABLE IF NOT EXISTS markets (
    id TEXT GENERATED ALWAYS AS (base_ticker || '/' || quote_ticker) STORED PRIMARY KEY,
    base_ticker TEXT NOT NULL REFERENCES tokens(ticker),
    quote_ticker TEXT NOT NULL REFERENCES tokens(ticker),
    tick_size BIGINT NOT NULL CHECK (tick_size > 0), -- in quote token decimals
    lot_size BIGINT NOT NULL CHECK (lot_size > 0), -- in base token decimals
    min_size BIGINT NOT NULL CHECK (min_size > 0), -- in base token decimals
    maker_fee_bps INT NOT NULL CHECK (maker_fee_bps >= 0 AND maker_fee_bps <= 10000), -- basis points (0-100%)
    taker_fee_bps INT NOT NULL CHECK (taker_fee_bps >= 0 AND taker_fee_bps <= 10000), -- basis points (0-100%)
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'suspended'))
);

CREATE TABLE IF NOT EXISTS orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_address TEXT NOT NULL REFERENCES users(address),
    market_id TEXT NOT NULL REFERENCES markets(id),
    price BIGINT NOT NULL CHECK (price > 0), -- in quote token decimals
    size BIGINT NOT NULL CHECK (size > 0), -- in base token decimals
    side side NOT NULL,
    type order_type NOT NULL,
    status order_status NOT NULL,
    filled_size BIGINT NOT NULL DEFAULT 0 CHECK (filled_size >= 0 AND filled_size <= size), -- in base token decimals
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS trades (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    market_id TEXT NOT NULL REFERENCES markets(id),
    buyer_address TEXT NOT NULL REFERENCES users(address),
    seller_address TEXT NOT NULL REFERENCES users(address),
    buyer_order_id UUID NOT NULL REFERENCES orders(id),
    seller_order_id UUID NOT NULL REFERENCES orders(id),
    price BIGINT NOT NULL CHECK (price > 0), -- in quote token decimals
    size BIGINT NOT NULL CHECK (size > 0), -- in base token decimals
    timestamp TIMESTAMP NOT NULL,
    CHECK (buyer_address != seller_address) -- self trading prevention
);

CREATE TABLE IF NOT EXISTS balances (
    user_address TEXT NOT NULL REFERENCES users(address),
    token_ticker TEXT NOT NULL REFERENCES tokens(ticker),
    amount BIGINT NOT NULL DEFAULT 0 CHECK (amount >= 0),
    open_interest BIGINT NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    PRIMARY KEY (user_address, token_ticker)
);

-- Performance indexes for efficient queries

-- Orders table indexes
CREATE INDEX IF NOT EXISTS idx_orders_user_status ON orders(user_address, status);
CREATE INDEX IF NOT EXISTS idx_orders_market_side_price ON orders(market_id, side, price);
CREATE INDEX IF NOT EXISTS idx_orders_market_status ON orders(market_id, status);
CREATE INDEX IF NOT EXISTS idx_orders_created_at ON orders(created_at);

-- Trades table indexes
CREATE INDEX IF NOT EXISTS idx_trades_market_timestamp ON trades(market_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_trades_buyer_address ON trades(buyer_address);
CREATE INDEX IF NOT EXISTS idx_trades_seller_address ON trades(seller_address);
CREATE INDEX IF NOT EXISTS idx_trades_timestamp ON trades(timestamp);

-- Balances table indexes
CREATE INDEX IF NOT EXISTS idx_balances_user_address ON balances(user_address);
CREATE INDEX IF NOT EXISTS idx_balances_token_ticker ON balances(token_ticker);

-- Markets table indexes
CREATE INDEX IF NOT EXISTS idx_markets_base_quote ON markets(base_ticker, quote_ticker);
CREATE INDEX IF NOT EXISTS idx_markets_status ON markets(status);

-- Users table indexes
CREATE INDEX IF NOT EXISTS idx_users_created_at ON users(created_at);

