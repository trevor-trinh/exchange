use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use clickhouse::Row;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::models::domain::{Balance, Market, Order, Token, Trade, User};
use crate::utils::BigDecimalExt;

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
    pub decimals: i32,
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

// ClickHouse-specific row types (for tick data and candles)
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct ClickHouseTradeRow {
    pub id: String, // UUID as string
    pub market_id: String,
    pub buyer_address: String,
    pub seller_address: String,
    pub buyer_order_id: String,  // UUID as string
    pub seller_order_id: String, // UUID as string
    pub price: u128,
    pub size: u128,
    pub timestamp: u32, // Unix timestamp
}

// Used for inserting candles into ClickHouse (includes trade_time)
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct CandleInsertRow {
    pub market_id: String,
    pub timestamp: u32, // ClickHouse DateTime is stored as Unix timestamp (u32) - bucket start
    pub trade_time: u32, // Original trade timestamp - used for ordering open/close
    pub interval: String, // '1m', '5m', '15m', '1h', '1d'
    pub open: u128,
    pub high: u128,
    pub low: u128,
    pub close: u128,
    pub volume: u128,
}

// Used for querying aggregated candles from ClickHouse (excludes trade_time)
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct CandleRow {
    pub market_id: String,
    pub timestamp: u32, // ClickHouse DateTime is stored as Unix timestamp (u32) - bucket start
    pub interval: String, // '1m', '5m', '15m', '1h', '1d'
    pub open: u128,
    pub high: u128,
    pub low: u128,
    pub close: u128,
    pub volume: u128,
}

// ============================================================================
// ROW TO DOMAIN TYPE CONVERSIONS
// ============================================================================

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        Self {
            address: row.address,
            created_at: row.created_at,
        }
    }
}

impl From<TokenRow> for Token {
    fn from(row: TokenRow) -> Self {
        Self {
            ticker: row.ticker,
            decimals: row.decimals as u8,
            name: row.name,
        }
    }
}

impl From<MarketRow> for Market {
    fn from(row: MarketRow) -> Self {
        Self {
            id: row.id,
            base_ticker: row.base_ticker,
            quote_ticker: row.quote_ticker,
            tick_size: row.tick_size.to_u128(),
            lot_size: row.lot_size.to_u128(),
            min_size: row.min_size.to_u128(),
            maker_fee_bps: row.maker_fee_bps,
            taker_fee_bps: row.taker_fee_bps,
        }
    }
}

impl From<OrderRow> for Order {
    fn from(row: OrderRow) -> Self {
        Self {
            id: row.id,
            user_address: row.user_address,
            market_id: row.market_id,
            price: row.price.to_u128(),
            size: row.size.to_u128(),
            side: row.side.parse().unwrap_or(crate::models::domain::Side::Buy),
            order_type: row
                .order_type
                .parse()
                .unwrap_or(crate::models::domain::OrderType::Limit),
            status: row
                .status
                .parse()
                .unwrap_or(crate::models::domain::OrderStatus::Pending),
            filled_size: row.filled_size.to_u128(),
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<TradeRow> for Trade {
    fn from(row: TradeRow) -> Self {
        Self {
            id: row.id,
            market_id: row.market_id,
            buyer_address: row.buyer_address,
            seller_address: row.seller_address,
            buyer_order_id: row.buyer_order_id,
            seller_order_id: row.seller_order_id,
            price: row.price.to_u128(),
            size: row.size.to_u128(),
            timestamp: row.timestamp,
        }
    }
}

impl From<BalanceRow> for Balance {
    fn from(row: BalanceRow) -> Self {
        Self {
            user_address: row.user_address,
            token_ticker: row.token_ticker,
            amount: row.amount.to_u128(),
            open_interest: row.open_interest.to_u128(),
            updated_at: row.updated_at,
        }
    }
}
