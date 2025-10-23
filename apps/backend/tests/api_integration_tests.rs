use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt; // for `oneshot` and `ready`

mod utils;
use utils::TestDb;

/// Helper function to make a GET request to the API
async fn get_request(app: axum::Router, uri: &str) -> (StatusCode, Value) {
    let response = app
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);

    (status, json)
}

#[tokio::test]
async fn test_health_endpoint() {
    let test_db = TestDb::setup()
        .await
        .expect("Failed to setup test database");

    let app = backend::api::rest::create_app(test_db.db);

    let (status, body) = get_request(app, "/api/health").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["message"], "Backend is running!");
    assert!(body["timestamp"].is_number());
}

#[tokio::test]
async fn test_health_endpoint_returns_json() {
    let test_db = TestDb::setup()
        .await
        .expect("Failed to setup test database");

    let app = backend::api::rest::create_app(test_db.db);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify content-type is application/json
    let content_type = response
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(content_type.contains("application/json"));

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert!(json.is_object());
    assert!(json.get("message").is_some());
    assert!(json.get("timestamp").is_some());
}

#[tokio::test]
async fn test_openapi_endpoint() {
    let test_db = TestDb::setup()
        .await
        .expect("Failed to setup test database");

    let app = backend::api::rest::create_app(test_db.db);

    let (status, body) = get_request(app, "/api/openapi.json").await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.get("openapi").is_some());
    assert!(body.get("info").is_some());
    assert!(body.get("paths").is_some());
}

#[tokio::test]
async fn test_swagger_ui_endpoint() {
    let test_db = TestDb::setup()
        .await
        .expect("Failed to setup test database");

    let app = backend::api::rest::create_app(test_db.db);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/docs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Swagger UI should return 200 or redirect
    assert!(response.status() == StatusCode::OK || response.status().is_redirection());
}

#[tokio::test]
async fn test_404_endpoint() {
    let test_db = TestDb::setup()
        .await
        .expect("Failed to setup test database");

    let app = backend::api::rest::create_app(test_db.db);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
