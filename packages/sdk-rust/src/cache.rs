//! Cache service for storing and retrieving markets and tokens
//!
//! Provides in-memory caching of market and token data to avoid
//! repeated REST API calls.

use backend::models::{api::ApiMarket, domain::Token};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::logger::Logger;

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub markets: usize,
    pub tokens: usize,
    pub initialized: bool,
}

/// Thread-safe cache service for markets and tokens
#[derive(Clone)]
pub struct CacheService {
    tokens: Arc<RwLock<HashMap<String, Token>>>,
    markets: Arc<RwLock<HashMap<String, ApiMarket>>>,
    initialized: Arc<RwLock<bool>>,
    logger: Arc<dyn Logger>,
}

impl CacheService {
    /// Create a new cache service
    pub fn new(logger: Arc<dyn Logger>) -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
            markets: Arc::new(RwLock::new(HashMap::new())),
            initialized: Arc::new(RwLock::new(false)),
            logger,
        }
    }

    // ===== Tokens =====

    /// Set tokens in the cache
    pub fn set_tokens(&self, tokens: Vec<Token>) {
        let mut cache = self.tokens.write().unwrap();
        cache.clear();
        for token in tokens.iter() {
            cache.insert(token.ticker.clone(), token.clone());
        }
        self.logger
            .debug(&format!("Cached {} tokens", tokens.len()));
    }

    /// Get a token by ticker
    pub fn get_token(&self, ticker: &str) -> Option<Token> {
        self.tokens.read().unwrap().get(ticker).cloned()
    }

    /// Get all tokens
    pub fn get_all_tokens(&self) -> Vec<Token> {
        self.tokens.read().unwrap().values().cloned().collect()
    }

    /// Check if token exists in cache
    pub fn has_token(&self, ticker: &str) -> bool {
        self.tokens.read().unwrap().contains_key(ticker)
    }

    // ===== Markets =====

    /// Set markets in the cache
    pub fn set_markets(&self, markets: Vec<ApiMarket>) {
        let mut cache = self.markets.write().unwrap();
        cache.clear();
        for market in markets.iter() {
            cache.insert(market.id.clone(), market.clone());
        }
        self.logger
            .debug(&format!("Cached {} markets", markets.len()));
    }

    /// Get a market by ID
    pub fn get_market(&self, market_id: &str) -> Option<ApiMarket> {
        self.markets.read().unwrap().get(market_id).cloned()
    }

    /// Get all markets
    pub fn get_all_markets(&self) -> Vec<ApiMarket> {
        self.markets.read().unwrap().values().cloned().collect()
    }

    /// Check if market exists in cache
    pub fn has_market(&self, market_id: &str) -> bool {
        self.markets.read().unwrap().contains_key(market_id)
    }

    // ===== Cache State =====

    /// Check if cache is ready (initialized and has data)
    pub fn is_ready(&self) -> bool {
        let initialized = *self.initialized.read().unwrap();
        let has_tokens = !self.tokens.read().unwrap().is_empty();
        let has_markets = !self.markets.read().unwrap().is_empty();
        initialized && has_tokens && has_markets
    }

    /// Mark cache as initialized
    pub fn mark_initialized(&self) {
        *self.initialized.write().unwrap() = true;
        let markets_count = self.markets.read().unwrap().len();
        let tokens_count = self.tokens.read().unwrap().len();
        self.logger.info(&format!(
            "Cache initialized: {} markets, {} tokens",
            markets_count, tokens_count
        ));
    }

    /// Clear the cache
    pub fn clear(&self) {
        self.tokens.write().unwrap().clear();
        self.markets.write().unwrap().clear();
        *self.initialized.write().unwrap() = false;
        self.logger.debug("Cache cleared");
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        CacheStats {
            markets: self.markets.read().unwrap().len(),
            tokens: self.tokens.read().unwrap().len(),
            initialized: *self.initialized.read().unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logger::NoopLogger;

    fn create_test_token(ticker: &str) -> Token {
        Token {
            ticker: ticker.to_string(),
            decimals: 6,
            name: format!("{} Token", ticker),
        }
    }

    fn create_test_market(id: &str, base: &str, quote: &str) -> ApiMarket {
        ApiMarket {
            id: id.to_string(),
            base_ticker: base.to_string(),
            quote_ticker: quote.to_string(),
            tick_size: "1000000".to_string(),
            lot_size: "1000000".to_string(),
            min_size: "1000000".to_string(),
            maker_fee_bps: 10,
            taker_fee_bps: 20,
        }
    }

    #[test]
    fn test_cache_tokens() {
        let cache = CacheService::new(Arc::new(NoopLogger));

        let tokens = vec![create_test_token("BTC"), create_test_token("USDC")];
        cache.set_tokens(tokens);

        assert!(cache.has_token("BTC"));
        assert!(cache.has_token("USDC"));
        assert!(!cache.has_token("ETH"));

        assert_eq!(cache.get_token("BTC").unwrap().ticker, "BTC");
        assert_eq!(cache.get_all_tokens().len(), 2);
    }

    #[test]
    fn test_cache_markets() {
        let cache = CacheService::new(Arc::new(NoopLogger));

        let markets = vec![
            create_test_market("BTC/USDC", "BTC", "USDC"),
            create_test_market("ETH/USDC", "ETH", "USDC"),
        ];
        cache.set_markets(markets);

        assert!(cache.has_market("BTC/USDC"));
        assert!(cache.has_market("ETH/USDC"));
        assert!(!cache.has_market("BTC/ETH"));

        assert_eq!(cache.get_market("BTC/USDC").unwrap().id, "BTC/USDC");
        assert_eq!(cache.get_all_markets().len(), 2);
    }

    #[test]
    fn test_cache_ready() {
        let cache = CacheService::new(Arc::new(NoopLogger));

        assert!(!cache.is_ready());

        cache.set_tokens(vec![create_test_token("BTC")]);
        assert!(!cache.is_ready()); // Still not ready (no markets)

        cache.set_markets(vec![create_test_market("BTC/USDC", "BTC", "USDC")]);
        assert!(!cache.is_ready()); // Still not ready (not marked initialized)

        cache.mark_initialized();
        assert!(cache.is_ready()); // Now ready
    }

    #[test]
    fn test_cache_clear() {
        let cache = CacheService::new(Arc::new(NoopLogger));

        cache.set_tokens(vec![create_test_token("BTC")]);
        cache.set_markets(vec![create_test_market("BTC/USDC", "BTC", "USDC")]);
        cache.mark_initialized();

        assert!(cache.is_ready());

        cache.clear();

        assert!(!cache.is_ready());
        assert_eq!(cache.get_all_tokens().len(), 0);
        assert_eq!(cache.get_all_markets().len(), 0);
    }

    #[test]
    fn test_cache_stats() {
        let cache = CacheService::new(Arc::new(NoopLogger));

        cache.set_tokens(vec![create_test_token("BTC"), create_test_token("USDC")]);
        cache.set_markets(vec![create_test_market("BTC/USDC", "BTC", "USDC")]);
        cache.mark_initialized();

        let stats = cache.get_stats();
        assert_eq!(stats.tokens, 2);
        assert_eq!(stats.markets, 1);
        assert!(stats.initialized);
    }
}
