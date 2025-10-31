use serde::{Deserialize, Serialize};

/// Hyperliquid subscription request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionRequest {
    pub method: String, // "subscribe"
    pub subscription: Subscription,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    #[serde(rename = "type")]
    pub sub_type: String, // "l2Book" or "trades"
    pub coin: String, // e.g., "BTC" for BTC-PERP (perpetual futures by default)
    #[serde(rename = "nSigFigs", skip_serializing_if = "Option::is_none")]
    pub n_sig_figs: Option<u8>, // Optional: 2-5 for aggregated levels, null for full precision
}

/// Hyperliquid L2 book snapshot/update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2BookData {
    pub coin: String,
    pub time: u64,
    pub levels: Vec<Vec<L2Level>>, // [bids, asks]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2Level {
    pub px: String, // price
    pub sz: String, // size
    pub n: u32,     // number of orders
}

/// Hyperliquid trade data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeData {
    pub coin: String,
    pub side: String, // "A" (ask/sell) or "B" (bid/buy)
    pub px: String,   // price
    pub sz: String,   // size
    pub time: u64,
    pub hash: String,
}

/// Hyperliquid WebSocket message wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperliquidMessage {
    pub channel: String,
    pub data: serde_json::Value,
}

/// Combined message type for our bot
#[derive(Debug, Clone)]
pub enum HlMessage {
    L2Book(L2BookData),
    Trade(Vec<TradeData>),
}
