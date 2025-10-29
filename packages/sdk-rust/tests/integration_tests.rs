/// Basic SDK smoke tests
///
/// These are simple tests that verify basic SDK functionality.
/// More comprehensive tests are in trading_tests.rs, websocket_tests.rs, and error_tests.rs.

use exchange_sdk::ExchangeClient;
use exchange_test_utils::TestServer;

#[tokio::test]
async fn test_health_endpoint() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = ExchangeClient::new(&server.base_url);

    let result = client.health().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_markets_empty() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = ExchangeClient::new(&server.base_url);

    let markets = client.get_markets().await.expect("Failed to get markets");
    assert_eq!(markets.len(), 0);
}

#[tokio::test]
async fn test_get_tokens_empty() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = ExchangeClient::new(&server.base_url);

    let tokens = client.get_tokens().await.expect("Failed to get tokens");
    assert_eq!(tokens.len(), 0);
}
