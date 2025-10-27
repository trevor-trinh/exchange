use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::domain::{Balance, Market, Order, OrderType, Side, Token, Trade};

// ============================================================================
// REST API TYPES
// ============================================================================

#[derive(Serialize, ToSchema)]
pub struct ApiResponse {
    pub message: String,
    pub timestamp: u64,
}

/// Response after successfully placing an order
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct OrderPlaced {
    pub order: Order,
    pub trades: Vec<Trade>,
}

/// Response after successfully cancelling an order
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct OrderCancelled {
    pub order_id: String, // UUID as string for OpenAPI compatibility
}

// ============================================================================
// INFO API TYPES
// ============================================================================

/// Info request with type discriminator
#[derive(Debug, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InfoRequest {
    TokenDetails { ticker: String },
    MarketDetails { market_id: String },
    AllMarkets,
    AllTokens,
}

/// Info response with type discriminator
#[derive(Debug, Serialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InfoResponse {
    TokenDetails { token: Token },
    MarketDetails { market: Market },
    AllMarkets { markets: Vec<Market> },
    AllTokens { tokens: Vec<Token> },
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InfoErrorResponse {
    pub error: String,
    pub code: String,
}

// ============================================================================
// USER API TYPES
// ============================================================================

/// User request with type discriminator
#[derive(Debug, Deserialize, ToSchema)]
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
#[derive(Debug, Serialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UserResponse {
    Orders { orders: Vec<Order> },
    Balances { balances: Vec<Balance> },
    Trades { trades: Vec<Trade> },
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserErrorResponse {
    pub error: String,
    pub code: String,
}

// ============================================================================
// TRADE API TYPES
// ============================================================================

/// Trade request with type discriminator
#[derive(Debug, Deserialize, ToSchema)]
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
}

/// Trade response with type discriminator
#[derive(Debug, Serialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TradeResponse {
    PlaceOrder { order: Order, trades: Vec<Trade> },
    CancelOrder { order_id: String },
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TradeErrorResponse {
    pub error: String,
    pub code: String,
}

// ============================================================================
// DRIP API TYPES
// ============================================================================

/// Drip request with type discriminator
#[derive(Debug, Deserialize, ToSchema)]
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
#[derive(Debug, Serialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DripResponse {
    Faucet {
        user_address: String,
        token_ticker: String,
        amount: String,
        new_balance: String,
    },
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DripErrorResponse {
    pub error: String,
    pub code: String,
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

#[derive(Debug, Serialize)]
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
        market_id: String,
        price: String,
        size: String,
        side: String,
        timestamp: i64,
    },
    TradeExecuted {
        trade: super::domain::Trade,
    },
    OrderbookSnapshot {
        orderbook: OrderbookData,
    },
    OrderUpdate {
        order_id: String,
        status: String,
        filled_size: String,
    },
    BalanceUpdate {
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

#[derive(Debug, Serialize)]
pub struct OrderbookData {
    pub market_id: String,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
}
