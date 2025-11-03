//! Service for enhancing raw API data with display values
//!
//! Adds human-readable display values and formatting to raw atom-based data.

use backend::models::{
    api::{ApiBalance, ApiOrder, ApiTrade},
    domain::OrderbookLevel,
};
use chrono::{DateTime, Utc};
use std::sync::Arc;

use crate::{cache::CacheService, format::*, SdkError, SdkResult};

/// Enhanced trade with display values
#[derive(Debug, Clone)]
pub struct EnhancedTrade {
    pub id: String,
    pub market_id: String,
    pub buyer_address: String,
    pub seller_address: String,
    pub buyer_order_id: String,
    pub seller_order_id: String,
    pub price: u128,
    pub size: u128,
    pub side: String,
    pub timestamp: DateTime<Utc>,
    // Enhanced fields
    pub price_display: String,
    pub size_display: String,
    pub price_value: f64,
    pub size_value: f64,
}

/// Enhanced order with display values
#[derive(Debug, Clone)]
pub struct EnhancedOrder {
    pub id: String,
    pub user_address: String,
    pub market_id: String,
    pub side: String,
    pub order_type: String,
    pub price: u128,
    pub size: u128,
    pub filled_size: u128,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Enhanced fields
    pub price_display: String,
    pub size_display: String,
    pub filled_display: String,
    pub price_value: f64,
    pub size_value: f64,
    pub filled_value: f64,
}

/// Enhanced balance with display values
#[derive(Debug, Clone)]
pub struct EnhancedBalance {
    pub user_address: String,
    pub token_ticker: String,
    pub amount: u128,
    pub open_interest: u128,
    pub updated_at: DateTime<Utc>,
    // Enhanced fields
    pub amount_display: String,
    pub locked_display: String,
    pub amount_value: f64,
    pub locked_value: f64,
}

/// Enhanced orderbook level with display values
#[derive(Debug, Clone)]
pub struct EnhancedOrderbookLevel {
    pub price: u128,
    pub size: u128,
    // Enhanced fields
    pub price_display: String,
    pub size_display: String,
    pub price_value: f64,
    pub size_value: f64,
}

/// Service for enhancing raw data with display values
pub struct EnhancementService {
    cache: Arc<CacheService>,
}

impl EnhancementService {
    /// Create a new enhancement service
    pub fn new(cache: Arc<CacheService>) -> Self {
        Self { cache }
    }

    /// Enhance a trade with display values
    pub fn enhance_trade(&self, trade: ApiTrade) -> SdkResult<EnhancedTrade> {
        let market = self
            .cache
            .get_market(&trade.market_id)
            .ok_or_else(|| SdkError::Enhancement(format!("Market {} not found in cache. Available markets: {}. Call get_markets() first to populate cache.",
                trade.market_id,
                self.cache.get_all_markets().iter().map(|m| m.id.as_str()).collect::<Vec<_>>().join(", ")
            )))?;

        let base_token = self.cache.get_token(&market.base_ticker).ok_or_else(|| {
            SdkError::Enhancement(format!(
                "Token {} not found in cache. Call get_tokens() first.",
                market.base_ticker
            ))
        })?;

        let quote_token = self.cache.get_token(&market.quote_ticker).ok_or_else(|| {
            SdkError::Enhancement(format!(
                "Token {} not found in cache. Call get_tokens() first.",
                market.quote_ticker
            ))
        })?;

        // Parse price and size
        let price = trade
            .price
            .parse::<u128>()
            .map_err(|e| SdkError::Enhancement(format!("Failed to parse price: {}", e)))?;

        let size = trade
            .size
            .parse::<u128>()
            .map_err(|e| SdkError::Enhancement(format!("Failed to parse size: {}", e)))?;

        Ok(EnhancedTrade {
            id: trade.id,
            market_id: trade.market_id,
            buyer_address: trade.buyer_address,
            seller_address: trade.seller_address,
            buyer_order_id: trade.buyer_order_id,
            seller_order_id: trade.seller_order_id,
            price,
            size,
            side: trade.side.to_string(),
            timestamp: trade.timestamp,
            price_display: format_price(price, quote_token.decimals),
            size_display: format_size(size, base_token.decimals),
            price_value: to_display_value(price, quote_token.decimals),
            size_value: to_display_value(size, base_token.decimals),
        })
    }

    /// Enhance an order with display values
    pub fn enhance_order(&self, order: ApiOrder, market_id: &str) -> SdkResult<EnhancedOrder> {
        let market = self.cache.get_market(market_id).ok_or_else(|| {
            SdkError::Enhancement(format!(
                "Market {} not found in cache. Call get_markets() first.",
                market_id
            ))
        })?;

        let base_token = self.cache.get_token(&market.base_ticker).ok_or_else(|| {
            SdkError::Enhancement(format!(
                "Token {} not found in cache. Call get_tokens() first.",
                market.base_ticker
            ))
        })?;

        let quote_token = self.cache.get_token(&market.quote_ticker).ok_or_else(|| {
            SdkError::Enhancement(format!(
                "Token {} not found in cache. Call get_tokens() first.",
                market.quote_ticker
            ))
        })?;

        // Parse values
        let price = order
            .price
            .parse::<u128>()
            .map_err(|e| SdkError::Enhancement(format!("Failed to parse price: {}", e)))?;

        let size = order
            .size
            .parse::<u128>()
            .map_err(|e| SdkError::Enhancement(format!("Failed to parse size: {}", e)))?;

        let filled_size = order
            .filled_size
            .parse::<u128>()
            .map_err(|e| SdkError::Enhancement(format!("Failed to parse filled_size: {}", e)))?;

        Ok(EnhancedOrder {
            id: order.id,
            user_address: order.user_address,
            market_id: order.market_id,
            side: order.side.to_string(),
            order_type: order.order_type.to_string(),
            price,
            size,
            filled_size,
            status: order.status.to_string(),
            created_at: order.created_at,
            updated_at: order.updated_at,
            price_display: format_price(price, quote_token.decimals),
            size_display: format_size(size, base_token.decimals),
            filled_display: format_size(filled_size, base_token.decimals),
            price_value: to_display_value(price, quote_token.decimals),
            size_value: to_display_value(size, base_token.decimals),
            filled_value: to_display_value(filled_size, base_token.decimals),
        })
    }

    /// Enhance a balance with display values
    pub fn enhance_balance(&self, balance: ApiBalance) -> SdkResult<EnhancedBalance> {
        let token = self.cache.get_token(&balance.token_ticker).ok_or_else(|| {
            SdkError::Enhancement(format!(
                "Token {} not found in cache. Call get_tokens() first.",
                balance.token_ticker
            ))
        })?;

        // Parse values
        let amount = balance
            .amount
            .parse::<u128>()
            .map_err(|e| SdkError::Enhancement(format!("Failed to parse amount: {}", e)))?;

        let open_interest = balance
            .open_interest
            .parse::<u128>()
            .map_err(|e| SdkError::Enhancement(format!("Failed to parse open_interest: {}", e)))?;

        Ok(EnhancedBalance {
            user_address: balance.user_address,
            token_ticker: balance.token_ticker,
            amount,
            open_interest,
            updated_at: balance.updated_at,
            amount_display: format_size(amount, token.decimals),
            locked_display: format_size(open_interest, token.decimals),
            amount_value: to_display_value(amount, token.decimals),
            locked_value: to_display_value(open_interest, token.decimals),
        })
    }

    /// Enhance an orderbook level with display values
    pub fn enhance_orderbook_level(
        &self,
        level: &OrderbookLevel,
        market_id: &str,
    ) -> SdkResult<EnhancedOrderbookLevel> {
        let market = self.cache.get_market(market_id).ok_or_else(|| {
            SdkError::Enhancement(format!(
                "Market {} not found in cache. Call get_markets() first.",
                market_id
            ))
        })?;

        let base_token = self.cache.get_token(&market.base_ticker).ok_or_else(|| {
            SdkError::Enhancement(format!(
                "Token {} not found in cache. Call get_tokens() first.",
                market.base_ticker
            ))
        })?;

        let quote_token = self.cache.get_token(&market.quote_ticker).ok_or_else(|| {
            SdkError::Enhancement(format!(
                "Token {} not found in cache. Call get_tokens() first.",
                market.quote_ticker
            ))
        })?;

        Ok(EnhancedOrderbookLevel {
            price: level.price,
            size: level.size,
            price_display: format_price(level.price, quote_token.decimals),
            size_display: format_size(level.size, base_token.decimals),
            price_value: to_display_value(level.price, quote_token.decimals),
            size_value: to_display_value(level.size, base_token.decimals),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logger::NoopLogger;
    use backend::models::{api::ApiMarket, domain::Token};

    fn setup_cache() -> Arc<CacheService> {
        let cache = Arc::new(CacheService::new(Arc::new(NoopLogger)));

        // Add test tokens
        cache.set_tokens(vec![
            Token {
                ticker: "BTC".to_string(),
                decimals: 8,
                name: "Bitcoin".to_string(),
            },
            Token {
                ticker: "USDC".to_string(),
                decimals: 6,
                name: "USD Coin".to_string(),
            },
        ]);

        // Add test market
        cache.set_markets(vec![ApiMarket {
            id: "BTC/USDC".to_string(),
            base_ticker: "BTC".to_string(),
            quote_ticker: "USDC".to_string(),
            tick_size: "1000000".to_string(),
            lot_size: "1000000".to_string(),
            min_size: "1000000".to_string(),
            maker_fee_bps: 10,
            taker_fee_bps: 20,
        }]);

        cache.mark_initialized();
        cache
    }

    #[test]
    fn test_enhance_trade() {
        let cache = setup_cache();
        let enhancer = EnhancementService::new(cache);

        let trade = ApiTrade {
            id: "trade1".to_string(),
            market_id: "BTC/USDC".to_string(),
            buyer_address: "buyer".to_string(),
            seller_address: "seller".to_string(),
            buyer_order_id: "order1".to_string(),
            seller_order_id: "order2".to_string(),
            price: "50000000000".to_string(), // 50000 USDC (6 decimals)
            size: "100000000".to_string(),    // 1 BTC (8 decimals)
            side: backend::models::domain::Side::Buy,
            timestamp: Utc::now(),
        };

        let enhanced = enhancer.enhance_trade(trade).unwrap();
        assert_eq!(enhanced.price_value, 50000.0);
        assert_eq!(enhanced.size_value, 1.0);
        assert!(!enhanced.price_display.is_empty());
        assert!(!enhanced.size_display.is_empty());
    }

    #[test]
    fn test_enhance_balance() {
        let cache = setup_cache();
        let enhancer = EnhancementService::new(cache);

        let balance = ApiBalance {
            user_address: "user1".to_string(),
            token_ticker: "BTC".to_string(),
            amount: "100000000".to_string(),       // 1 BTC
            open_interest: "50000000".to_string(), // 0.5 BTC locked
            updated_at: Utc::now(),
        };

        let enhanced = enhancer.enhance_balance(balance).unwrap();
        assert_eq!(enhanced.amount_value, 1.0);
        assert_eq!(enhanced.locked_value, 0.5);
    }
}
