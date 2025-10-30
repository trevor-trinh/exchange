/// Integration tests for bot orders using testcontainers
/// These tests verify end-to-end functionality including proper formatting for frontend display

use backend::models::domain::{OrderStatus, OrderType, Side};
use exchange_sdk::ExchangeClient;
use exchange_test_utils::TestServer;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::str::FromStr;

/// Helper to setup test market with proper configuration
async fn setup_test_market(server: &TestServer, market_id: &str) -> anyhow::Result<()> {
    let client = ExchangeClient::new(&server.base_url);

    // Create BTC token
    client.admin_create_token(
        "BTC".to_string(),
        6,
        "Bitcoin".to_string(),
    ).await?;

    // Create USDC token
    client.admin_create_token(
        "USDC".to_string(),
        6,
        "USD Coin".to_string(),
    ).await?;

    // Create market with proper constraints
    client.admin_create_market(
        "BTC".to_string(),
        "USDC".to_string(),
        1000000,  // tick_size: 1 USDC (6 decimals)
        1000,     // lot_size: 0.001 BTC (6 decimals)
        1000,     // min_size: 0.001 BTC
        5,        // maker_fee_bps: 0.05%
        10,       // taker_fee_bps: 0.10%
    ).await?;

    Ok(())
}

/// Test that bot orders with proper rounding are accepted
#[tokio::test]
async fn test_bot_order_with_proper_rounding() {
    let server = TestServer::start().await.expect("Failed to start test server");
    setup_test_market(&server, "BTC/USDC").await.expect("Failed to setup market");

    let client = ExchangeClient::new(&server.base_url);
    let user_address = "test_bot".to_string();
    let market_id = "BTC/USDC".to_string();

    // Fund the test bot (need enough for collateral)
    // Note: Collateral calculation uses raw values, so we need massive amounts
    client.admin_faucet(user_address.clone(), "BTC".to_string(), "100000000000000".to_string())
        .await
        .expect("Failed to fund BTC");
    client.admin_faucet(user_address.clone(), "USDC".to_string(), "100000000000000000".to_string())
        .await
        .expect("Failed to fund USDC");

    // Convert Hyperliquid price to our format (like bot does)
    let hl_price = Decimal::from_str("95000.0").unwrap(); // $95,000 (rounded to tick_size)
    let price_scaled = hl_price * Decimal::from(1_000_000);
    let price = price_scaled.to_u128().unwrap().to_string();

    // NOTE: place_order_with_rounding() only rounds SIZE, not PRICE!
    // Bots must pre-round prices to tick_size multiples themselves

    // Convert size with multiplier
    let hl_size = Decimal::from_str("0.1").unwrap(); // 0.1 BTC
    let size_multiplier = Decimal::from_str("0.1").unwrap(); // 10%
    let adjusted_size = hl_size * size_multiplier; // 0.01 BTC
    let size_scaled = adjusted_size * Decimal::from(1_000_000);
    let size = size_scaled.to_u128().unwrap().to_string();

    // Place order using bot's rounding helper
    let result = client.place_order_with_rounding(
        user_address.clone(),
        market_id,
        Side::Buy,
        OrderType::Limit,
        price,
        size,
        "test_bot".to_string(),
    ).await;

    assert!(result.is_ok(), "Order should be placed successfully: {:?}", result);
    let placed = result.unwrap();

    // Verify order was placed
    assert_eq!(placed.order.status, OrderStatus::Pending);

    // Verify the order size was properly rounded to lot_size
    // lot_size = 1000 (0.001 BTC), so 0.01 BTC = 10000 should be valid
    assert_eq!(placed.order.size, 10000);
}

/// Test that very small orders (below min_size) are rounded to 0
#[tokio::test]
async fn test_bot_order_below_min_size_rounded_to_zero() {
    let server = TestServer::start().await.expect("Failed to start test server");
    setup_test_market(&server, "BTC/USDC").await.expect("Failed to setup market");

    let client = ExchangeClient::new(&server.base_url);
    let user_address = "test_bot_small".to_string();
    let market_id = "BTC/USDC".to_string();

    // Fund the bot (massive amounts for collateral)
    client.admin_faucet(user_address.clone(), "BTC".to_string(), "100000000000000".to_string())
        .await
        .expect("Failed to fund BTC");
    client.admin_faucet(user_address.clone(), "USDC".to_string(), "100000000000000000".to_string())
        .await
        .expect("Failed to fund USDC");

    // Create an order that's too small after size_multiplier
    let hl_size = Decimal::from_str("0.005").unwrap(); // 0.005 BTC
    let size_multiplier = Decimal::from_str("0.05").unwrap(); // 5%
    let adjusted_size = hl_size * size_multiplier; // 0.00025 BTC
    let size_scaled = adjusted_size * Decimal::from(1_000_000);
    let size = size_scaled.to_u128().unwrap().to_string();
    // size = "250" which is below min_size (1000)

    let price = "95000000000".to_string(); // $95,000

    let result = client.place_order_with_rounding(
        user_address,
        market_id,
        Side::Buy,
        OrderType::Limit,
        price,
        size,
        "test_bot".to_string(),
    ).await;

    // Order should either be rejected OR rounded to 0
    match result {
        Ok(placed) => {
            // If accepted, size must be 0 (rounded down from 250)
            assert_eq!(placed.order.size, 0, "Order below min_size should be rounded to 0");
        }
        Err(_) => {
            // Or it might be rejected - both behaviors are acceptable
            // The SDK's place_order_with_rounding returns error for size=0
        }
    }
}

/// Test that orders with proper formatting display correctly
#[tokio::test]
async fn test_bot_order_displays_correctly() {
    let server = TestServer::start().await.expect("Failed to start test server");
    setup_test_market(&server, "BTC/USDC").await.expect("Failed to setup market");

    let client = ExchangeClient::new(&server.base_url);
    let user_address = "test_bot_display".to_string();
    let market_id = "BTC/USDC".to_string();

    // Fund the bot (massive amounts for collateral)
    client.admin_faucet(user_address.clone(), "BTC".to_string(), "100000000000000".to_string())
        .await
        .expect("Failed to fund BTC");
    client.admin_faucet(user_address.clone(), "USDC".to_string(), "100000000000000000".to_string())
        .await
        .expect("Failed to fund USDC");

    // Place a properly formatted order
    let price = "95000000000".to_string(); // $95,000
    let size = "10000".to_string(); // 0.01 BTC

    let result = client.place_order_with_rounding(
        user_address.clone(),
        market_id.clone(),
        Side::Buy,
        OrderType::Limit,
        price,
        size,
        "test_bot".to_string(),
    ).await;

    assert!(result.is_ok(), "Failed to place order: {:?}", result);
    let placed = result.unwrap();
    let order_id = placed.order.id;

    // Fetch the order back from the API
    let orders = client.get_orders(&user_address, Some(market_id))
        .await
        .expect("Failed to fetch orders");

    // Find our order
    let order = orders.iter().find(|o| o.id == order_id)
        .expect("Order not found in list");

    // Verify the order displays with correct precision
    // The frontend should display:
    // - Price: $95,000.00 (from 95000000000 with 6 decimals)
    // - Size: 0.01 BTC (from 10000 with 6 decimals)
    assert_eq!(order.price, 95000000000);
    assert_eq!(order.size, 10000);

    // For frontend display:
    let displayed_price = order.price as f64 / 1_000_000.0;
    let displayed_size = order.size as f64 / 1_000_000.0;

    assert!((displayed_price - 95000.0).abs() < 0.01, "Price display incorrect: {}", displayed_price);
    assert!((displayed_size - 0.01).abs() < 0.000001, "Size display incorrect: {}", displayed_size);
}

/// Test that fractional prices are rounded correctly to tick_size
#[tokio::test]
async fn test_bot_order_with_fractional_price() {
    let server = TestServer::start().await.expect("Failed to start test server");
    setup_test_market(&server, "BTC/USDC").await.expect("Failed to setup market");

    let client = ExchangeClient::new(&server.base_url);
    let user_address = "test_bot_frac".to_string();
    let market_id = "BTC/USDC".to_string();

    // Fund the bot (massive amounts for collateral)
    client.admin_faucet(user_address.clone(), "BTC".to_string(), "100000000000000".to_string())
        .await
        .expect("Failed to fund BTC");
    client.admin_faucet(user_address.clone(), "USDC".to_string(), "100000000000000000".to_string())
        .await
        .expect("Failed to fund USDC");

    // Price with fractional part: $95,123.789
    // Bot must round to tick_size BEFORE converting to raw format
    let hl_price = Decimal::from_str("95123.789").unwrap();
    let tick_size_decimal = Decimal::from_str("1.0").unwrap(); // 1 USDC
    let rounded_price = (hl_price / tick_size_decimal).round() * tick_size_decimal;
    let price_scaled = rounded_price * Decimal::from(1_000_000);
    let price = price_scaled.to_u128().unwrap().to_string();
    // price should now be "95124000000" (rounded to $95,124)

    let size = "10000".to_string(); // 0.01 BTC

    let result = client.place_order_with_rounding(
        user_address,
        market_id,
        Side::Buy,
        OrderType::Limit,
        price,
        size,
        "test_bot".to_string(),
    ).await;

    // Should succeed with pre-rounded price
    assert!(result.is_ok(), "Order should be placed with rounded price: {:?}", result);

    if let Ok(placed) = result {
        // Verify price was rounded to nearest tick_size
        let tick_size = 1000000u128;
        assert_eq!(placed.order.price % tick_size, 0,
            "Price should be rounded to tick_size multiple: {} % {} = {}",
            placed.order.price, tick_size, placed.order.price % tick_size);

        // $95,123.789 rounds to $95,124
        assert_eq!(placed.order.price, 95124000000,
            "Price should be rounded to $95,124, got: {}", placed.order.price);
    }
}

/// Test multiple bot orders in sequence (orderbook mirror scenario)
#[tokio::test]
async fn test_bot_multiple_orders() {
    let server = TestServer::start().await.expect("Failed to start test server");
    setup_test_market(&server, "BTC/USDC").await.expect("Failed to setup market");

    let client = ExchangeClient::new(&server.base_url);
    let user_address = "test_bot_multi".to_string();
    let market_id = "BTC/USDC".to_string();

    // Fund the bot generously
    client.admin_faucet(user_address.clone(), "BTC".to_string(), "100000000000000".to_string())
        .await
        .expect("Failed to fund BTC");
    client.admin_faucet(user_address.clone(), "USDC".to_string(), "100000000000000000".to_string())
        .await
        .expect("Failed to fund USDC");

    // Place multiple orders like orderbook_mirror bot would
    let orders = vec![
        ("94000000000", "5000"),  // $94,000, 0.005 BTC
        ("94500000000", "5000"),  // $94,500, 0.005 BTC
        ("95000000000", "10000"), // $95,000, 0.01 BTC
        ("95500000000", "5000"),  // $95,500, 0.005 BTC
        ("96000000000", "5000"),  // $96,000, 0.005 BTC
    ];

    for (price, size) in orders {
        let result = client.place_order_with_rounding(
            user_address.clone(),
            market_id.clone(),
            Side::Buy,
            OrderType::Limit,
            price.to_string(),
            size.to_string(),
            "test_bot".to_string(),
        ).await;

        assert!(result.is_ok(), "Failed to place order at {}: {:?}", price, result);
    }

    // Verify all orders are in the orderbook
    let orders = client.get_orders(&user_address, Some(market_id))
        .await
        .expect("Failed to fetch orders");

    assert!(orders.len() >= 5, "Should have at least 5 orders, got: {}", orders.len());

    // Verify orders are properly formatted for frontend display
    for order in orders.iter() {
        // All prices should be multiples of tick_size (1 USDC = 1000000)
        assert_eq!(order.price % 1000000, 0, "Price {} not rounded to tick_size", order.price);

        // All sizes should be multiples of lot_size (0.001 BTC = 1000)
        assert_eq!(order.size % 1000, 0, "Size {} not rounded to lot_size", order.size);

        // All sizes should be >= min_size (0.001 BTC = 1000)
        assert!(order.size >= 1000, "Size {} below min_size", order.size);
    }
}

/// Test that bot can cancel all orders efficiently
#[tokio::test]
async fn test_bot_cancel_all_orders() {
    let server = TestServer::start().await.expect("Failed to start test server");
    setup_test_market(&server, "BTC/USDC").await.expect("Failed to setup market");

    let client = ExchangeClient::new(&server.base_url);
    let user_address = "test_bot_cancel".to_string();
    let market_id = "BTC/USDC".to_string();

    // Fund the bot (massive amounts for collateral)
    client.admin_faucet(user_address.clone(), "BTC".to_string(), "100000000000000".to_string())
        .await
        .expect("Failed to fund BTC");
    client.admin_faucet(user_address.clone(), "USDC".to_string(), "100000000000000000".to_string())
        .await
        .expect("Failed to fund USDC");

    // Place multiple orders
    for i in 0..10 {
        let price = format!("{}", 95000000000u128 + (i * 1000000)); // $95,000 + $1 increments
        let size = "5000".to_string(); // 0.005 BTC

        client.place_order_with_rounding(
            user_address.clone(),
            market_id.clone(),
            Side::Buy,
            OrderType::Limit,
            price,
            size,
            "test_bot".to_string(),
        ).await.expect("Failed to place order");
    }

    // Verify orders were placed
    let orders_before = client.get_orders(&user_address, Some(market_id.clone()))
        .await
        .expect("Failed to fetch orders");
    assert_eq!(orders_before.len(), 10, "Should have 10 orders");

    // Cancel all orders (like orderbook_mirror bot does)
    let result = client.cancel_all_orders(
        user_address.clone(),
        Some(market_id.clone()),
        "test_bot".to_string(),
    ).await;

    assert!(result.is_ok(), "Failed to cancel all orders: {:?}", result);
    let cancelled = result.unwrap();
    assert_eq!(cancelled.count, 10, "Should have cancelled 10 orders");

    // Small delay to ensure cancellations are propagated
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify orders were cancelled
    let orders_after = client.get_orders(&user_address, Some(market_id))
        .await
        .expect("Failed to fetch orders");

    // Orders should be cancelled (status changed) or removed from pending list
    let pending_orders: Vec<_> = orders_after.iter()
        .filter(|o| o.status == OrderStatus::Pending)
        .collect();
    assert_eq!(pending_orders.len(), 0,
        "Should have 0 pending orders after cancellation, got {} (total: {})",
        pending_orders.len(), orders_after.len());
}
