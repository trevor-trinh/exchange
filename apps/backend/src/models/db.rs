use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use clickhouse::Row;
use serde::{Deserialize, Serialize};
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
    pub tick_size: BigDecimal,
    pub lot_size: BigDecimal,
    pub min_size: BigDecimal,
    pub maker_fee_bps: i32,
    pub taker_fee_bps: i32,
}

#[derive(Debug, Clone, FromRow)]
pub struct OrderRow {
    pub id: Uuid,
    pub user_address: String,
    pub market_id: String,
    pub price: BigDecimal,
    pub size: BigDecimal,
    pub side: String, // Custom type 'side' in DB, stored as TEXT
    #[sqlx(rename = "type")]
    pub order_type: String, // Custom type 'order_type' in DB
    pub status: String, // Custom type 'order_status' in DB
    pub filled_size: BigDecimal,
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
    pub price: BigDecimal,
    pub size: BigDecimal,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct BalanceRow {
    pub user_address: String,
    pub token_ticker: String,
    pub amount: BigDecimal,
    pub open_interest: BigDecimal,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct CandleRow {
    pub market_id: String,
    pub timestamp: DateTime<Utc>,
    pub open: u128,
    pub high: u128,
    pub low: u128,
    pub close: u128,
    pub volume: u128,
}
