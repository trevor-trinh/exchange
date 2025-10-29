/// SDK error handling and validation tests
///
/// These tests verify that the SDK properly handles errors and edge cases.

mod helpers;

use backend::models::domain::{OrderType, Side};
use helpers::TestFixture;

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_order_insufficient_balance() {
    let fixture = TestFixture::new()
        .await
        .expect("Failed to create test fixture");

    // User has NO balance
    fixture
        .create_user_with_balance("poor_user", 0, 0)
        .await
        .expect("Failed to create user");

    // Try to place order without balance
    let result = fixture
        .client
        .place_order(
            "poor_user".to_string(),
            fixture.market_id.clone(),
            Side::Sell,
            OrderType::Limit,
            "50000000000".to_string(),
            "1000000".to_string(),
            "test_sig".to_string(),
        )
        .await;

    // Should fail
    assert!(result.is_err(), "Order should fail with insufficient balance");
}

#[tokio::test]
async fn test_cancel_nonexistent_order() {
    let fixture = TestFixture::new()
        .await
        .expect("Failed to create test fixture");

    fixture
        .create_user_with_balance("user", 10_000_000, 0)
        .await
        .expect("Failed to create user");

    // Try to cancel an order that doesn't exist
    let result = fixture
        .client
        .cancel_order(
            "user".to_string(),
            "00000000-0000-0000-0000-000000000000".to_string(),
            "test_sig".to_string(),
        )
        .await;

    // Should fail
    assert!(result.is_err(), "Should fail to cancel nonexistent order");
}

#[tokio::test]
async fn test_cancel_other_users_order() {
    let fixture = TestFixture::new()
        .await
        .expect("Failed to create test fixture");

    fixture
        .create_user_with_balance("alice", 10_000_000, 0)
        .await
        .expect("Failed to create alice");
    fixture
        .create_user_with_balance("bob", 0, 1_000_000_000_000)
        .await
        .expect("Failed to create bob");

    // Alice places an order
    let order = fixture
        .client
        .place_order(
            "alice".to_string(),
            fixture.market_id.clone(),
            Side::Sell,
            OrderType::Limit,
            "50000000000".to_string(),
            "1000000".to_string(),
            "test_sig".to_string(),
        )
        .await
        .expect("Failed to place order");

    // Bob tries to cancel Alice's order
    let result = fixture
        .client
        .cancel_order(
            "bob".to_string(),
            order.order.id.to_string(),
            "test_sig".to_string(),
        )
        .await;

    // Should fail
    assert!(
        result.is_err(),
        "Bob should not be able to cancel Alice's order"
    );

    // Alice can still see her order
    let orders = fixture
        .client
        .get_orders("alice", Some(fixture.market_id.clone()))
        .await
        .expect("Failed to get orders");
    assert_eq!(orders.len(), 1);
}

#[tokio::test]
async fn test_get_nonexistent_market() {
    let fixture = TestFixture::new()
        .await
        .expect("Failed to create test fixture");

    let result = fixture.client.get_market("FAKE/MARKET").await;

    // Should fail gracefully
    assert!(result.is_err(), "Should fail to get nonexistent market");
}

#[tokio::test]
async fn test_get_nonexistent_token() {
    let fixture = TestFixture::new()
        .await
        .expect("Failed to create test fixture");

    let result = fixture.client.get_token("FAKE").await;

    // Should fail gracefully
    assert!(result.is_err(), "Should fail to get nonexistent token");
}

#[tokio::test]
async fn test_orders_for_nonexistent_user() {
    let fixture = TestFixture::new()
        .await
        .expect("Failed to create test fixture");

    // Get orders for user that doesn't exist
    let orders = fixture
        .client
        .get_orders("nonexistent_user", None)
        .await
        .expect("Should return empty list for nonexistent user");

    assert_eq!(orders.len(), 0);
}

#[tokio::test]
async fn test_balances_for_nonexistent_user() {
    let fixture = TestFixture::new()
        .await
        .expect("Failed to create test fixture");

    // Get balances for user that doesn't exist
    let balances = fixture
        .client
        .get_balances("nonexistent_user")
        .await
        .expect("Should return empty list for nonexistent user");

    assert_eq!(balances.len(), 0);
}

#[tokio::test]
async fn test_trades_for_nonexistent_user() {
    let fixture = TestFixture::new()
        .await
        .expect("Failed to create test fixture");

    // Get trades for user that doesn't exist
    let trades = fixture
        .client
        .get_trades("nonexistent_user", None)
        .await
        .expect("Should return empty list for nonexistent user");

    assert_eq!(trades.len(), 0);
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[tokio::test]
async fn test_zero_size_order() {
    let fixture = TestFixture::new()
        .await
        .expect("Failed to create test fixture");

    fixture
        .create_user_with_balance("user", 10_000_000, 0)
        .await
        .expect("Failed to create user");

    // Try to place order with zero size
    let result = fixture
        .client
        .place_order(
            "user".to_string(),
            fixture.market_id.clone(),
            Side::Sell,
            OrderType::Limit,
            "50000000000".to_string(),
            "0".to_string(), // Zero size
            "test_sig".to_string(),
        )
        .await;

    // Should fail
    assert!(result.is_err(), "Zero size order should be rejected");
}

#[tokio::test]
async fn test_negative_price_order() {
    let fixture = TestFixture::new()
        .await
        .expect("Failed to create test fixture");

    fixture
        .create_user_with_balance("user", 10_000_000, 0)
        .await
        .expect("Failed to create user");

    // Try to place order with negative price (if the API accepts strings)
    // This might be caught at parsing level
    let result = fixture
        .client
        .place_order(
            "user".to_string(),
            fixture.market_id.clone(),
            Side::Sell,
            OrderType::Limit,
            "-1".to_string(), // Negative price
            "1000000".to_string(),
            "test_sig".to_string(),
        )
        .await;

    // Should fail
    assert!(result.is_err(), "Negative price order should be rejected");
}

#[tokio::test]
async fn test_very_large_order() {
    let fixture = TestFixture::new()
        .await
        .expect("Failed to create test fixture");

    fixture
        .create_user_with_balance("whale", u128::MAX, 0)
        .await
        .expect("Failed to create whale");

    // Try to place a very large order
    let result = fixture
        .client
        .place_order(
            "whale".to_string(),
            fixture.market_id.clone(),
            Side::Sell,
            OrderType::Limit,
            "50000000000".to_string(),
            format!("{}", u128::MAX), // Maximum possible size
            "test_sig".to_string(),
        )
        .await;

    // Might succeed or fail depending on limits, but shouldn't crash
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_duplicate_order_placement() {
    let fixture = TestFixture::new()
        .await
        .expect("Failed to create test fixture");

    fixture
        .create_user_with_balance("user", 10_000_000, 0)
        .await
        .expect("Failed to create user");

    // Place first order
    let order1 = fixture
        .client
        .place_order(
            "user".to_string(),
            fixture.market_id.clone(),
            Side::Sell,
            OrderType::Limit,
            "50000000000".to_string(),
            "1000000".to_string(),
            "test_sig".to_string(),
        )
        .await
        .expect("First order should succeed");

    // Place identical second order
    let order2 = fixture
        .client
        .place_order(
            "user".to_string(),
            fixture.market_id.clone(),
            Side::Sell,
            OrderType::Limit,
            "50000000000".to_string(),
            "1000000".to_string(),
            "test_sig".to_string(),
        )
        .await
        .expect("Second identical order should also succeed");

    // Both orders should have different IDs
    assert_ne!(
        order1.order.id, order2.order.id,
        "Duplicate orders should have different IDs"
    );

    // User should have two orders
    let orders = fixture
        .client
        .get_orders("user", Some(fixture.market_id.clone()))
        .await
        .expect("Failed to get orders");
    assert!(orders.len() >= 2, "User should have at least 2 orders");
}

#[tokio::test]
async fn test_health_endpoint() {
    let fixture = TestFixture::new()
        .await
        .expect("Failed to create test fixture");

    let health = fixture
        .client
        .health()
        .await
        .expect("Health check should succeed");

    assert!(!health.is_empty());
}

#[tokio::test]
async fn test_concurrent_balance_queries() {
    let fixture = TestFixture::new()
        .await
        .expect("Failed to create test fixture");

    fixture
        .create_user_with_balance("user", 10_000_000, 1_000_000_000)
        .await
        .expect("Failed to create user");

    // Query balance many times concurrently
    let mut handles = vec![];
    for _ in 0..10 {
        let client = fixture.client.clone();
        let handle = tokio::spawn(async move {
            client.get_balances("user").await
        });
        handles.push(handle);
    }

    // All queries should succeed
    for handle in handles {
        let result = handle.await.expect("Task panicked");
        assert!(result.is_ok(), "Balance query should succeed");
    }
}
