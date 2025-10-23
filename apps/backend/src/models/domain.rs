use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// ENUMS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
    Limit,
    Market,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Pending,
    Filled,
    PartiallyFilled,
    Cancelled,
}

// ============================================================================
// DOMAIN TYPES
// ============================================================================

#[derive(Debug, Clone)]
pub struct User {
    pub address: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub ticker: String,
    pub decimals: u8,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Market {
    pub id: String, // Generated as "base_ticker/quote_ticker"
    pub base_ticker: String,
    pub quote_ticker: String,
    pub tick_size: u128,    // Minimum price increment in quote atoms
    pub lot_size: u128,     // Minimum size increment in base atoms
    pub min_size: u128,     // Minimum order size in base atoms
    pub maker_fee_bps: i32, // Maker fee in basis points (0-10000)
    pub taker_fee_bps: i32, // Taker fee in basis points (0-10000)
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: Uuid,
    pub user_address: String,
    pub market_id: String, // Generated as "base_ticker/quote_ticker"
    pub price: u128,
    pub size: u128,
    pub side: Side,
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub filled_size: u128,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Trade {
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

#[derive(Debug, Clone)]
pub struct Balance {
    pub user_address: String,
    pub token_ticker: String,
    pub amount: u128,
    pub open_interest: u128,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Candle {
    pub market_id: String,
    pub timestamp: DateTime<Utc>,
    pub open: u128,
    pub high: u128,
    pub low: u128,
    pub close: u128,
    pub volume: u128,
}
