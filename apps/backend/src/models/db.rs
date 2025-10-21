use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

// ============================================================================
// DATABASE ROW TYPES
// ============================================================================

#[derive(Debug, Clone, FromRow)]
pub struct UserRow {
    pub address: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct TokenRow {
    pub ticker: String,
    pub decimals: u8,
    pub name: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct MarketRow {
    pub id: String,
    pub base_ticker: String,
    pub quote_ticker: String,
    pub tick_size: u128,
    pub lot_size: u128,
    pub min_size: u64,
    pub maker_fee_bps: i32,
    pub taker_fee_bps: i32,
}

#[derive(Debug, Clone, FromRow)]
pub struct OrderRow {
    pub id: Uuid,
    pub user_address: String,
    pub market_id: String,
    pub price: u128,
    pub size: u128,
    pub side: String, // Custom type 'side' in DB, stored as TEXT
    #[sqlx(rename = "type")]
    pub order_type: String, // Custom type 'order_type' in DB
    pub status: String, // Custom type 'order_status' in DB
    pub filled_size: u128,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct TradeRow {
    pub id: Uuid,
    pub market_id: String,
    pub buyer_address: String,
    pub seller_address: String,
    pub buyer_order_id: Uuid,
    pub seller_order_id: Uuid,
    pub price: u128,
    pub size: u128,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct BalanceRow {
    pub user_address: String,
    pub token_ticker: String,
    pub amount: u128,
    pub open_interest: u128,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct CandleRow {
    pub market_id: String,
    pub timestamp: DateTime<Utc>,
    pub open: u128,
    pub high: u128,
    pub low: u128,
    pub close: u128,
    pub volume: u128,
}
