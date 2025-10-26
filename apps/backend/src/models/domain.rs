use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::errors::ExchangeError;
use crate::models::api::{OrderCancelled, OrderPlaced};
// ============================================================================
// ENUMS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
    Limit,
    Market,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct User {
    pub address: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Token {
    pub ticker: String,
    pub decimals: u8,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Market {
    pub id: String, // Generated as "base_ticker/quote_ticker"
    pub base_ticker: String,
    pub quote_ticker: String,
    #[serde(
        serialize_with = "serialize_u128_as_string",
        deserialize_with = "deserialize_u128_from_string"
    )]
    pub tick_size: u128, // Minimum price increment in quote atoms
    #[serde(
        serialize_with = "serialize_u128_as_string",
        deserialize_with = "deserialize_u128_from_string"
    )]
    pub lot_size: u128, // Minimum size increment in base atoms
    #[serde(
        serialize_with = "serialize_u128_as_string",
        deserialize_with = "deserialize_u128_from_string"
    )]
    pub min_size: u128, // Minimum order size in base atoms
    pub maker_fee_bps: i32, // Maker fee in basis points (0-10000)
    pub taker_fee_bps: i32, // Taker fee in basis points (0-10000)
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Order {
    pub id: Uuid,
    pub user_address: String,
    pub market_id: String, // Generated as "base_ticker/quote_ticker"
    #[serde(
        serialize_with = "serialize_u128_as_string",
        deserialize_with = "deserialize_u128_from_string"
    )]
    pub price: u128,
    #[serde(
        serialize_with = "serialize_u128_as_string",
        deserialize_with = "deserialize_u128_from_string"
    )]
    pub size: u128,
    pub side: Side,
    pub order_type: OrderType,
    pub status: OrderStatus,
    #[serde(
        serialize_with = "serialize_u128_as_string",
        deserialize_with = "deserialize_u128_from_string"
    )]
    pub filled_size: u128,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Trade {
    pub id: Uuid,
    pub market_id: String,
    pub buyer_address: String,
    pub seller_address: String,
    pub buyer_order_id: Uuid,
    pub seller_order_id: Uuid,
    #[serde(
        serialize_with = "serialize_u128_as_string",
        deserialize_with = "deserialize_u128_from_string"
    )]
    pub price: u128,
    #[serde(
        serialize_with = "serialize_u128_as_string",
        deserialize_with = "deserialize_u128_from_string"
    )]
    pub size: u128,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Balance {
    pub user_address: String,
    pub token_ticker: String,
    #[serde(
        serialize_with = "serialize_u128_as_string",
        deserialize_with = "deserialize_u128_from_string"
    )]
    pub amount: u128,
    #[serde(
        serialize_with = "serialize_u128_as_string",
        deserialize_with = "deserialize_u128_from_string"
    )]
    pub open_interest: u128,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Candle {
    pub market_id: String,
    pub timestamp: DateTime<Utc>,
    #[serde(
        serialize_with = "serialize_u128_as_string",
        deserialize_with = "deserialize_u128_from_string"
    )]
    pub open: u128,
    #[serde(
        serialize_with = "serialize_u128_as_string",
        deserialize_with = "deserialize_u128_from_string"
    )]
    pub high: u128,
    #[serde(
        serialize_with = "serialize_u128_as_string",
        deserialize_with = "deserialize_u128_from_string"
    )]
    pub low: u128,
    #[serde(
        serialize_with = "serialize_u128_as_string",
        deserialize_with = "deserialize_u128_from_string"
    )]
    pub close: u128,
    #[serde(
        serialize_with = "serialize_u128_as_string",
        deserialize_with = "deserialize_u128_from_string"
    )]
    pub volume: u128,
}

// Helper functions for serializing u128 as strings (JSON doesn't support u128)
fn serialize_u128_as_string<S>(value: &u128, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}

fn deserialize_u128_from_string<'de, D>(deserializer: D) -> Result<u128, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}

// ============================================================================
// MATCHING ENGINE TYPES
// ============================================================================

/// Represents a match between two orders
#[derive(Debug, Clone)]
pub struct Match {
    pub maker_order_id: Uuid,
    pub taker_order_id: Uuid,
    pub price: u128,
    pub size: u128,
}

// ============================================================================
// ENGINE REQUEST/RESPONSE TYPES
// ============================================================================

/// Requests sent from REST API to matching engine
/// Each request includes a oneshot channel for synchronous response
pub enum EngineRequest {
    PlaceOrder {
        order: Order,
        response_tx: oneshot::Sender<Result<OrderPlaced, ExchangeError>>,
    },
    CancelOrder {
        order_id: Uuid,
        user_address: String,
        response_tx: oneshot::Sender<Result<OrderCancelled, ExchangeError>>,
    },
}

/// Events broadcast from matching engine to WebSocket clients
/// These are asynchronous notifications that don't require a response
#[derive(Debug, Clone)]
pub enum EngineEvent {
    TradeExecuted {
        trade: Trade,
    },
    OrderPlaced {
        order: Order,
    },
    OrderCancelled {
        order_id: Uuid,
        user_address: String,
    },
    OrderbookChanged {
        market_id: String,
        bids: Vec<(u128, u128)>, // (price, total_size)
        asks: Vec<(u128, u128)>,
    },
    BalanceUpdated {
        user_address: String,
        token_ticker: String,
        available: u128,
        locked: u128,
    },
}

// ============================================================================
// SUBSCRIPTION TYPES
// ============================================================================

/// Represents what real-time data a WebSocket client wants to receive
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Subscription {
    Trades { market_id: String },
    Orderbook { market_id: String },
    Candles { market_id: String },
    User { user_address: String },
}

impl Subscription {
    /// Convert a client message to a domain subscription
    pub fn from_message(msg: &crate::models::api::ClientMessage) -> Option<Self> {
        use crate::models::api::{ClientMessage, SubscriptionChannel};

        match msg {
            ClientMessage::Subscribe {
                channel,
                market_id,
                user_address,
            }
            | ClientMessage::Unsubscribe {
                channel,
                market_id,
                user_address,
            } => match channel {
                SubscriptionChannel::Trades => market_id.as_ref().map(|id| Subscription::Trades {
                    market_id: id.clone(),
                }),
                SubscriptionChannel::Orderbook => {
                    market_id.as_ref().map(|id| Subscription::Orderbook {
                        market_id: id.clone(),
                    })
                }
                SubscriptionChannel::User => user_address.as_ref().map(|addr| Subscription::User {
                    user_address: addr.clone(),
                }),
            },
            ClientMessage::Ping => None,
        }
    }
}
