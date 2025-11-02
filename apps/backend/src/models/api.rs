use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::domain::{OrderStatus, OrderType, Side, Token};

// ============================================================================
// REST API TYPES
// ============================================================================

#[derive(Serialize, ToSchema)]
pub struct ApiResponse {
    pub message: String,
    pub timestamp: u64,
}

/// Response after successfully placing an order
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OrderPlaced {
    pub order: ApiOrder,
    pub trades: Vec<ApiTrade>,
}

/// Response after successfully cancelling an order
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OrderCancelled {
    pub order_id: String, // UUID as string for OpenAPI compatibility
}

/// Response after successfully cancelling all orders
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OrdersCancelled {
    pub cancelled_order_ids: Vec<String>, // UUIDs as strings for OpenAPI compatibility
    pub count: usize,
}

// ============================================================================
// INFO API TYPES
// ============================================================================

/// Info request with type discriminator
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InfoRequest {
    TokenDetails { ticker: String },
    MarketDetails { market_id: String },
    AllMarkets,
    AllTokens,
}

/// Info response with type discriminator
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InfoResponse {
    TokenDetails { token: Token },
    MarketDetails { market: ApiMarket },
    AllMarkets { markets: Vec<ApiMarket> },
    AllTokens { tokens: Vec<Token> },
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct InfoErrorResponse {
    pub error: String,
    pub code: String,
}

// ============================================================================
// USER API TYPES
// ============================================================================

/// User request with type discriminator
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UserRequest {
    Orders {
        user_address: String,
        market_id: Option<String>,
        status: Option<String>,
        limit: Option<u32>,
    },
    Balances {
        user_address: String,
    },
    Trades {
        user_address: String,
        market_id: Option<String>,
        limit: Option<u32>,
    },
}

/// User response with type discriminator
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UserResponse {
    Orders { orders: Vec<ApiOrder> },
    Balances { balances: Vec<ApiBalance> },
    Trades { trades: Vec<ApiTrade> },
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserErrorResponse {
    pub error: String,
    pub code: String,
}

// ============================================================================
// TRADE API TYPES
// ============================================================================

/// Trade request with type discriminator
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TradeRequest {
    PlaceOrder {
        user_address: String,
        market_id: String,
        side: Side,
        order_type: OrderType,
        price: String,     // u128 as string
        size: String,      // u128 as string
        signature: String, // Cryptographic signature for authentication
    },
    CancelOrder {
        user_address: String,
        order_id: String,  // UUID as string
        signature: String, // Cryptographic signature for authentication
    },
    CancelAllOrders {
        user_address: String,
        market_id: Option<String>, // Optional: cancel only for specific market
        signature: String,         // Cryptographic signature for authentication
    },
}

/// Trade response with type discriminator
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TradeResponse {
    PlaceOrder {
        order: ApiOrder,
        trades: Vec<ApiTrade>,
    },
    CancelOrder {
        order_id: String,
    },
    CancelAllOrders {
        cancelled_order_ids: Vec<String>,
        count: usize,
    },
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TradeErrorResponse {
    pub error: String,
    pub code: String,
}

// ============================================================================
// DRIP API TYPES
// ============================================================================

/// Drip request with type discriminator
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DripRequest {
    Faucet {
        user_address: String,
        token_ticker: String,
        amount: String,    // u128 as string
        signature: String, // Cryptographic signature for authentication
    },
}

/// Drip response with type discriminator
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DripResponse {
    Faucet {
        user_address: String,
        token_ticker: String,
        amount: String,
        new_balance: String,
    },
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DripErrorResponse {
    pub error: String,
    pub code: String,
}

// ============================================================================
// ADMIN API TYPES
// ============================================================================

/// Admin request with type discriminator
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AdminRequest {
    CreateToken {
        ticker: String,
        decimals: u8,
        name: String,
    },
    CreateMarket {
        base_ticker: String,
        quote_ticker: String,
        tick_size: String, // u128 as string
        lot_size: String,  // u128 as string
        min_size: String,  // u128 as string
        maker_fee_bps: i32,
        taker_fee_bps: i32,
    },
    Faucet {
        user_address: String,
        token_ticker: String,
        amount: String,
        signature: String,
    },
}

/// Admin response with type discriminator
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AdminResponse {
    CreateToken {
        token: Token,
    },
    CreateMarket {
        market: ApiMarket,
    },
    Faucet {
        user_address: String,
        token_ticker: String,
        amount: String,
        new_balance: String,
    },
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AdminErrorResponse {
    pub error: String,
    pub code: String,
}

// ============================================================================
// CANDLES API TYPES
// ============================================================================

/// Request for OHLCV candles
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CandlesRequest {
    pub market_id: String,
    pub interval: String, // 1m, 5m, 15m, 1h, 1d
    pub from: i64,        // Unix timestamp in seconds
    pub to: i64,          // Unix timestamp in seconds
    #[serde(default)]
    pub count_back: Option<usize>, // Limit results to N most recent bars before 'to'
}

/// OHLCV candle data
#[derive(Debug, Serialize, Deserialize, clickhouse::Row, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ApiCandle {
    pub timestamp: u32,
    pub open: u128,
    pub high: u128,
    pub low: u128,
    pub close: u128,
    pub volume: u128,
}

/// Response containing candles
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CandlesResponse {
    pub candles: Vec<ApiCandle>,
}

// ============================================================================
// WEBSOCKET MESSAGE TYPES (Client → Server)
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Subscribe {
        channel: SubscriptionChannel,
        #[serde(skip_serializing_if = "Option::is_none")]
        market_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        user_address: Option<String>,
    },
    Unsubscribe {
        channel: SubscriptionChannel,
        #[serde(skip_serializing_if = "Option::is_none")]
        market_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        user_address: Option<String>,
    },
    Ping,
}

/// Channel types for WebSocket subscriptions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionChannel {
    Trades,
    Orderbook,
    User,
}

// ============================================================================
// WEBSOCKET MESSAGE TYPES (Server → Client)
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    // Subscription acknowledgments
    Subscribed {
        channel: SubscriptionChannel,
        #[serde(skip_serializing_if = "Option::is_none")]
        market_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        user_address: Option<String>,
    },
    Unsubscribed {
        channel: SubscriptionChannel,
        #[serde(skip_serializing_if = "Option::is_none")]
        market_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        user_address: Option<String>,
    },

    // Real-time data updates
    Trade {
        trade: TradeData,
    },
    Orderbook {
        orderbook: OrderbookData,
    },
    Order {
        order_id: String,
        status: String,
        filled_size: String,
    },
    Balance {
        token_ticker: String,
        available: String,
        locked: String,
    },
    Candle {
        market_id: String,
        timestamp: i64,
        open: String,
        high: String,
        low: String,
        close: String,
        volume: String,
    },

    // Connection management
    Error {
        message: String,
    },
    Pong,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceLevel {
    pub price: String,
    pub size: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderbookData {
    pub market_id: String,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
}

/// Trade data for WebSocket messages (API layer with String fields)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeData {
    pub id: String, // UUID as string
    pub market_id: String,
    pub buyer_address: String,
    pub seller_address: String,
    pub buyer_order_id: String,  // UUID as string
    pub seller_order_id: String, // UUID as string
    pub price: String,           // u128 as string
    pub size: String,            // u128 as string
    pub timestamp: i64,          // Unix timestamp for WebSocket compatibility
}

// ============================================================================
// API DTOs (for HTTP responses)
// ============================================================================

/// API representation of Market with String fields for JSON compatibility
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiMarket {
    pub id: String,
    pub base_ticker: String,
    pub quote_ticker: String,
    pub tick_size: String, // u128 as string
    pub lot_size: String,  // u128 as string
    pub min_size: String,  // u128 as string
    pub maker_fee_bps: i32,
    pub taker_fee_bps: i32,
}

/// API representation of Order with String fields for JSON compatibility
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiOrder {
    pub id: String, // UUID as string
    pub user_address: String,
    pub market_id: String,
    pub price: String, // u128 as string
    pub size: String,  // u128 as string
    pub side: Side,
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub filled_size: String, // u128 as string
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// API representation of Trade with String fields for JSON compatibility
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiTrade {
    pub id: String, // UUID as string
    pub market_id: String,
    pub buyer_address: String,
    pub seller_address: String,
    pub buyer_order_id: String,    // UUID as string
    pub seller_order_id: String,   // UUID as string
    pub price: String,             // u128 as string
    pub size: String,              // u128 as string
    pub side: super::domain::Side, // Taker's side (determines if trade is "buy" or "sell" on tape)
    pub timestamp: DateTime<Utc>,
}

/// API representation of Balance with String fields for JSON compatibility
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiBalance {
    pub user_address: String,
    pub token_ticker: String,
    pub amount: String,        // u128 as string
    pub open_interest: String, // u128 as string
    pub updated_at: DateTime<Utc>,
}

// Conversion implementations from domain to API types
impl From<super::domain::Market> for ApiMarket {
    fn from(m: super::domain::Market) -> Self {
        Self {
            id: m.id,
            base_ticker: m.base_ticker,
            quote_ticker: m.quote_ticker,
            tick_size: m.tick_size.to_string(),
            lot_size: m.lot_size.to_string(),
            min_size: m.min_size.to_string(),
            maker_fee_bps: m.maker_fee_bps,
            taker_fee_bps: m.taker_fee_bps,
        }
    }
}

impl From<super::domain::Order> for ApiOrder {
    fn from(o: super::domain::Order) -> Self {
        Self {
            id: o.id.to_string(),
            user_address: o.user_address,
            market_id: o.market_id,
            price: o.price.to_string(),
            size: o.size.to_string(),
            side: o.side,
            order_type: o.order_type,
            status: o.status,
            filled_size: o.filled_size.to_string(),
            created_at: o.created_at,
            updated_at: o.updated_at,
        }
    }
}

impl From<super::domain::Trade> for ApiTrade {
    fn from(t: super::domain::Trade) -> Self {
        Self {
            id: t.id.to_string(),
            market_id: t.market_id,
            buyer_address: t.buyer_address,
            seller_address: t.seller_address,
            buyer_order_id: t.buyer_order_id.to_string(),
            seller_order_id: t.seller_order_id.to_string(),
            price: t.price.to_string(),
            size: t.size.to_string(),
            side: t.side,
            timestamp: t.timestamp,
        }
    }
}

impl From<super::domain::Balance> for ApiBalance {
    fn from(b: super::domain::Balance) -> Self {
        Self {
            user_address: b.user_address,
            token_ticker: b.token_ticker,
            amount: b.amount.to_string(),
            open_interest: b.open_interest.to_string(),
            updated_at: b.updated_at,
        }
    }
}

// Reverse conversions from API to domain types (for SDK)
impl TryFrom<ApiMarket> for super::domain::Market {
    type Error = std::num::ParseIntError;

    fn try_from(m: ApiMarket) -> Result<Self, Self::Error> {
        Ok(Self {
            id: m.id,
            base_ticker: m.base_ticker,
            quote_ticker: m.quote_ticker,
            tick_size: m.tick_size.parse()?,
            lot_size: m.lot_size.parse()?,
            min_size: m.min_size.parse()?,
            maker_fee_bps: m.maker_fee_bps,
            taker_fee_bps: m.taker_fee_bps,
        })
    }
}

impl TryFrom<ApiOrder> for super::domain::Order {
    type Error = Box<dyn std::error::Error>;

    fn try_from(o: ApiOrder) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Uuid::parse_str(&o.id)?,
            user_address: o.user_address,
            market_id: o.market_id,
            price: o.price.parse()?,
            size: o.size.parse()?,
            side: o.side,
            order_type: o.order_type,
            status: o.status,
            filled_size: o.filled_size.parse()?,
            created_at: o.created_at,
            updated_at: o.updated_at,
        })
    }
}

impl TryFrom<ApiTrade> for super::domain::Trade {
    type Error = Box<dyn std::error::Error>;

    fn try_from(t: ApiTrade) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Uuid::parse_str(&t.id)?,
            market_id: t.market_id,
            buyer_address: t.buyer_address,
            seller_address: t.seller_address,
            buyer_order_id: Uuid::parse_str(&t.buyer_order_id)?,
            seller_order_id: Uuid::parse_str(&t.seller_order_id)?,
            price: t.price.parse()?,
            size: t.size.parse()?,
            side: t.side,
            timestamp: t.timestamp,
        })
    }
}

impl TryFrom<ApiBalance> for super::domain::Balance {
    type Error = std::num::ParseIntError;

    fn try_from(b: ApiBalance) -> Result<Self, Self::Error> {
        Ok(Self {
            user_address: b.user_address,
            token_ticker: b.token_ticker,
            amount: b.amount.parse()?,
            open_interest: b.open_interest.parse()?,
            updated_at: b.updated_at,
        })
    }
}
