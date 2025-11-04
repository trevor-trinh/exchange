use chrono::{DateTime, Utc};
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::str::FromStr;
use tokio::sync::oneshot;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::errors::ExchangeError;
use crate::models::api::{OrderCancelled, OrderPlaced, OrdersCancelled};
// ============================================================================
// ENUMS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
    Limit,
    Market,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Pending,
    Filled,
    PartiallyFilled,
    Cancelled,
}

// ============================================================================
// ENUM STRING CONVERSIONS
// ============================================================================

impl Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Side::Buy => "buy",
                Side::Sell => "sell",
            }
        )
    }
}

impl FromStr for Side {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "buy" => Ok(Side::Buy),
            "sell" => Ok(Side::Sell),
            _ => Err(format!("Invalid side: {}", s)),
        }
    }
}

impl Display for OrderType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                OrderType::Limit => "limit",
                OrderType::Market => "market",
            }
        )
    }
}

impl FromStr for OrderType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "limit" => Ok(OrderType::Limit),
            "market" => Ok(OrderType::Market),
            _ => Err(format!("Invalid order type: {}", s)),
        }
    }
}

impl Display for OrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                OrderStatus::Pending => "pending",
                OrderStatus::Filled => "filled",
                OrderStatus::PartiallyFilled => "partially_filled",
                OrderStatus::Cancelled => "cancelled",
            }
        )
    }
}

impl FromStr for OrderStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(OrderStatus::Pending),
            "filled" => Ok(OrderStatus::Filled),
            "partially_filled" => Ok(OrderStatus::PartiallyFilled),
            "cancelled" => Ok(OrderStatus::Cancelled),
            _ => Err(format!("Invalid order status: {}", s)),
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Trade {
    pub id: Uuid,
    pub market_id: String,
    pub buyer_address: String,
    pub seller_address: String,
    pub buyer_order_id: Uuid,
    pub seller_order_id: Uuid,
    pub price: u128,
    pub size: u128,
    pub side: Side, // Taker's side (determines if trade is "buy" or "sell" on tape)
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Balance {
    pub user_address: String,
    pub token_ticker: String,
    pub amount: u128,
    pub open_interest: u128,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub market_id: String,
    pub timestamp: DateTime<Utc>,
    pub open: u128,
    pub high: u128,
    pub low: u128,
    pub close: u128,
    pub volume: u128,
}

// ============================================================================
// MATCHING ENGINE TYPES
// ============================================================================

/// Represents a match between two orders
#[derive(Debug, Clone)]
pub struct Match {
    pub maker_order: Order, // Full maker order for easy access
    pub price: u128,
    pub size: u128,
}

// ============================================================================
// ORDERBOOK TYPES
// ============================================================================

/// Represents a price level in the orderbook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookLevel {
    pub price: u128,
    pub size: u128,
}

/// Snapshot of an orderbook at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookSnapshot {
    pub market_id: String,
    pub bids: Vec<OrderbookLevel>, // Sorted by price descending (highest first)
    pub asks: Vec<OrderbookLevel>, // Sorted by price ascending (lowest first)
    pub timestamp: DateTime<Utc>,
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
    CancelAllOrders {
        user_address: String,
        market_id: Option<String>,
        response_tx: oneshot::Sender<Result<OrdersCancelled, ExchangeError>>,
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
    BalanceUpdated {
        balance: Balance,
    },
    OrderbookSnapshot {
        orderbook: OrderbookSnapshot,
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
    UserFills { user_address: String },
    UserOrders { user_address: String },
    UserBalances { user_address: String },
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
                SubscriptionChannel::UserFills => {
                    user_address.as_ref().map(|addr| Subscription::UserFills {
                        user_address: addr.clone(),
                    })
                }
                SubscriptionChannel::UserOrders => {
                    user_address.as_ref().map(|addr| Subscription::UserOrders {
                        user_address: addr.clone(),
                    })
                }
                SubscriptionChannel::UserBalances => {
                    user_address.as_ref().map(|addr| Subscription::UserBalances {
                        user_address: addr.clone(),
                    })
                }
            },
            ClientMessage::Ping => None,
        }
    }
}
