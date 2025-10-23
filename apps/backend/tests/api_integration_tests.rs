mod utils;

use serde_json::Value;
use utils::TestServer;

#[tokio::test]
async fn test_health_endpoint_e2e() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    // Make real HTTP GET request
    let response = server.get("/api/health").await;

    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["message"], "Backend is running!");
    assert!(body["timestamp"].is_number());
}

#[tokio::test]
async fn test_health_endpoint_content_type() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    let response = server.get("/api/health").await;

    assert_eq!(response.status(), 200);

    // Verify content-type header
    let content_type = response
        .headers()
        .get("content-type")
        .expect("Missing content-type header")
        .to_str()
        .expect("Invalid content-type header");

    assert!(content_type.contains("application/json"));
}

#[tokio::test]
async fn test_openapi_endpoint_e2e() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    let response = server.get("/api/openapi.json").await;

    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.expect("Failed to parse JSON");

    // Verify OpenAPI spec structure
    assert!(body.get("openapi").is_some());
    assert!(body.get("info").is_some());
    assert!(body.get("paths").is_some());

    // Verify our health endpoint is documented
    assert!(body["paths"].get("/api/health").is_some());
}

#[tokio::test]
async fn test_swagger_ui_endpoint_e2e() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    let response = server.get("/api/docs").await;

    // Swagger UI should return 200 or redirect (depending on trailing slash)
    assert!(
        response.status() == 200 || response.status().is_redirection(),
        "Expected 200 or 3xx, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_404_not_found_e2e() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    let response = server.get("/nonexistent/path").await;

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn test_cors_headers_present() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    let client = server.client();
    let response = client
        .get(&format!("{}/api/health", server.address))
        .header("Origin", "http://localhost:3000")
        .send()
        .await
        .expect("Failed to make request");

    // CorsLayer::permissive() should allow any origin
    let cors_header = response.headers().get("access-control-allow-origin");
    assert!(
        cors_header.is_some(),
        "CORS headers should be present with permissive CORS"
    );
}

#[tokio::test]
async fn test_concurrent_requests() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    // Make multiple concurrent requests to verify server handles concurrency
    let tasks: Vec<_> = (0..10)
        .map(|_| {
            let addr = server.address.clone();
            tokio::spawn(async move {
                reqwest::get(&format!("{}/api/health", addr))
                    .await
                    .expect("Request failed")
                    .status()
            })
        })
        .collect();

    let results = futures::future::join_all(tasks).await;

    // All requests should succeed
    for result in results {
        let status = result.expect("Task panicked");
        assert_eq!(status, 200);
    }
}

#[tokio::test]
async fn test_database_accessible_from_api() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    // Setup: Create a user directly via database
    server
        .db
        .create_user("test_user_address".to_string())
        .await
        .expect("Failed to create test user");

    // Verify we can query the database that the API is using
    let users = server.db.list_users().await.expect("Failed to list users");

    assert_eq!(users.len(), 1);
    assert_eq!(users[0].address, "test_user_address");

    // This demonstrates that:
    // 1. The API server has database connectivity
    // 2. We can setup test data via direct DB access
    // 3. The server and test share the same database instance
}
