use crate::error::{SdkError, SdkResult};
use backend::models::{
    api::*,
    domain::*,
};
use reqwest::Client;

/// REST API client for the exchange
#[derive(Clone)]
pub struct ExchangeClient {
    base_url: String,
    client: Client,
}

impl ExchangeClient {
    /// Create a new client with the given base URL
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: Client::new(),
        }
    }

    /// Health check
    pub async fn health(&self) -> SdkResult<String> {
        let url = format!("{}/api/health", self.base_url);
        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            Ok(response.text().await?)
        } else {
            Err(SdkError::ApiError {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            })
        }
    }

    // ===== Info Endpoints =====

    /// Get token details
    pub async fn get_token(&self, ticker: &str) -> SdkResult<Token> {
        let request = InfoRequest::TokenDetails {
            ticker: ticker.to_string(),
        };
        let response = self.post_info(request).await?;

        match response {
            InfoResponse::TokenDetails { token } => Ok(token),
            _ => Err(SdkError::InvalidResponse("Expected TokenDetails".to_string())),
        }
    }

    /// Get market details
    pub async fn get_market(&self, market_id: &str) -> SdkResult<Market> {
        let request = InfoRequest::MarketDetails {
            market_id: market_id.to_string(),
        };
        let response = self.post_info(request).await?;

        match response {
            InfoResponse::MarketDetails { market } => Ok(market),
            _ => Err(SdkError::InvalidResponse("Expected MarketDetails".to_string())),
        }
    }

    /// Get all markets
    pub async fn get_markets(&self) -> SdkResult<Vec<Market>> {
        let request = InfoRequest::AllMarkets;
        let response = self.post_info(request).await?;

        match response {
            InfoResponse::AllMarkets { markets } => Ok(markets),
            _ => Err(SdkError::InvalidResponse("Expected AllMarkets".to_string())),
        }
    }

    /// Get all tokens
    pub async fn get_tokens(&self) -> SdkResult<Vec<Token>> {
        let request = InfoRequest::AllTokens;
        let response = self.post_info(request).await?;

        match response {
            InfoResponse::AllTokens { tokens } => Ok(tokens),
            _ => Err(SdkError::InvalidResponse("Expected AllTokens".to_string())),
        }
    }

    // ===== User Endpoints =====

    /// Get user orders
    pub async fn get_orders(&self, user_address: &str, market_id: Option<String>) -> SdkResult<Vec<Order>> {
        let request = UserRequest::Orders {
            user_address: user_address.to_string(),
            market_id,
            status: None,
            limit: None,
        };
        let response = self.post_user(request).await?;

        match response {
            UserResponse::Orders { orders } => Ok(orders),
            _ => Err(SdkError::InvalidResponse("Expected Orders".to_string())),
        }
    }

    /// Get user balances
    pub async fn get_balances(&self, user_address: &str) -> SdkResult<Vec<Balance>> {
        let request = UserRequest::Balances {
            user_address: user_address.to_string(),
        };
        let response = self.post_user(request).await?;

        match response {
            UserResponse::Balances { balances } => Ok(balances),
            _ => Err(SdkError::InvalidResponse("Expected Balances".to_string())),
        }
    }

    /// Get user trades
    pub async fn get_trades(&self, user_address: &str, market_id: Option<String>) -> SdkResult<Vec<Trade>> {
        let request = UserRequest::Trades {
            user_address: user_address.to_string(),
            market_id,
            limit: None,
        };
        let response = self.post_user(request).await?;

        match response {
            UserResponse::Trades { trades } => Ok(trades),
            _ => Err(SdkError::InvalidResponse("Expected Trades".to_string())),
        }
    }

    // ===== Trade Endpoints =====

    /// Place an order
    pub async fn place_order(
        &self,
        user_address: String,
        market_id: String,
        side: Side,
        order_type: OrderType,
        price: String,
        size: String,
        signature: String,
    ) -> SdkResult<OrderPlaced> {
        let request = TradeRequest::PlaceOrder {
            user_address,
            market_id,
            side,
            order_type,
            price,
            size,
            signature,
        };
        let response = self.post_trade(request).await?;

        match response {
            TradeResponse::PlaceOrder { order, trades } => Ok(OrderPlaced { order, trades }),
            _ => Err(SdkError::InvalidResponse("Expected PlaceOrder".to_string())),
        }
    }

    /// Cancel an order
    pub async fn cancel_order(
        &self,
        user_address: String,
        order_id: String,
        signature: String,
    ) -> SdkResult<OrderCancelled> {
        let request = TradeRequest::CancelOrder {
            user_address,
            order_id,
            signature,
        };
        let response = self.post_trade(request).await?;

        match response {
            TradeResponse::CancelOrder { order_id } => Ok(OrderCancelled { order_id }),
            _ => Err(SdkError::InvalidResponse("Expected CancelOrder".to_string())),
        }
    }

    // ===== Drip/Faucet Endpoint =====

    /// Request testnet tokens from faucet
    pub async fn faucet(&self, user_address: String, token_ticker: String, amount: String, signature: String) -> SdkResult<(String, String, String, String)> {
        let request = DripRequest::Faucet {
            user_address,
            token_ticker,
            amount,
            signature,
        };
        let response = self.post_drip(request).await?;

        match response {
            DripResponse::Faucet { user_address, token_ticker, amount, new_balance } => {
                Ok((user_address, token_ticker, amount, new_balance))
            }
        }
    }

    // ===== Admin Endpoints (Test/Dev Only) =====

    /// Create a token (admin)
    pub async fn admin_create_token(&self, ticker: String, decimals: u8, name: String) -> SdkResult<Token> {
        let request = backend::models::api::AdminRequest::CreateToken {
            ticker,
            decimals,
            name,
        };
        let response = self.post_admin(request).await?;

        match response {
            backend::models::api::AdminResponse::CreateToken { token } => Ok(token),
            _ => Err(SdkError::InvalidResponse("Expected CreateToken".to_string())),
        }
    }

    /// Create a market (admin)
    pub async fn admin_create_market(
        &self,
        base_ticker: String,
        quote_ticker: String,
        tick_size: u128,
        lot_size: u128,
        min_size: u128,
        maker_fee_bps: i32,
        taker_fee_bps: i32,
    ) -> SdkResult<Market> {
        let request = backend::models::api::AdminRequest::CreateMarket {
            base_ticker,
            quote_ticker,
            tick_size,
            lot_size,
            min_size,
            maker_fee_bps,
            taker_fee_bps,
        };
        let response = self.post_admin(request).await?;

        match response {
            backend::models::api::AdminResponse::CreateMarket { market } => Ok(market),
            _ => Err(SdkError::InvalidResponse("Expected CreateMarket".to_string())),
        }
    }

    /// Faucet via admin endpoint
    pub async fn admin_faucet(&self, user_address: String, token_ticker: String, amount: String) -> SdkResult<String> {
        let request = backend::models::api::AdminRequest::Faucet {
            user_address,
            token_ticker,
            amount,
            signature: "admin".to_string(),
        };
        let response = self.post_admin(request).await?;

        match response {
            backend::models::api::AdminResponse::Faucet { new_balance, .. } => Ok(new_balance),
            _ => Err(SdkError::InvalidResponse("Expected Faucet".to_string())),
        }
    }

    // ===== Internal Helper Methods =====

    async fn post_info(&self, request: InfoRequest) -> SdkResult<InfoResponse> {
        let url = format!("{}/api/info", self.base_url);
        let response = self.client.post(&url).json(&request).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let error: serde_json::Value = response.json().await?;
            Err(SdkError::ApiError {
                status: error.get("code").and_then(|v| v.as_str()).unwrap_or("500").parse().unwrap_or(500),
                message: error.get("error").and_then(|v| v.as_str()).unwrap_or("Unknown error").to_string(),
            })
        }
    }

    async fn post_user(&self, request: UserRequest) -> SdkResult<UserResponse> {
        let url = format!("{}/api/user", self.base_url);
        let response = self.client.post(&url).json(&request).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let error: serde_json::Value = response.json().await?;
            Err(SdkError::ApiError {
                status: error.get("code").and_then(|v| v.as_str()).unwrap_or("500").parse().unwrap_or(500),
                message: error.get("error").and_then(|v| v.as_str()).unwrap_or("Unknown error").to_string(),
            })
        }
    }

    async fn post_trade(&self, request: TradeRequest) -> SdkResult<TradeResponse> {
        let url = format!("{}/api/trade", self.base_url);
        let response = self.client.post(&url).json(&request).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let error: serde_json::Value = response.json().await?;
            Err(SdkError::ApiError {
                status: error.get("code").and_then(|v| v.as_str()).unwrap_or("500").parse().unwrap_or(500),
                message: error.get("error").and_then(|v| v.as_str()).unwrap_or("Unknown error").to_string(),
            })
        }
    }

    async fn post_drip(&self, request: DripRequest) -> SdkResult<DripResponse> {
        let url = format!("{}/api/drip", self.base_url);
        let response = self.client.post(&url).json(&request).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let error: serde_json::Value = response.json().await?;
            Err(SdkError::ApiError {
                status: error.get("code").and_then(|v| v.as_str()).unwrap_or("500").parse().unwrap_or(500),
                message: error.get("error").and_then(|v| v.as_str()).unwrap_or("Unknown error").to_string(),
            })
        }
    }

    async fn post_admin(&self, request: backend::models::api::AdminRequest) -> SdkResult<backend::models::api::AdminResponse> {
        let url = format!("{}/api/admin", self.base_url);
        let response = self.client.post(&url).json(&request).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let error: serde_json::Value = response.json().await?;
            Err(SdkError::ApiError {
                status: error.get("code").and_then(|v| v.as_str()).unwrap_or("500").parse().unwrap_or(500),
                message: error.get("error").and_then(|v| v.as_str()).unwrap_or("Unknown error").to_string(),
            })
        }
    }
}
