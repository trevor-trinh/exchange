use crate::models::ApiResponse;
use axum::response::Json;

#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = 200, description = "Health check response", body = ApiResponse)
    )
)]
pub async fn health_check() -> Json<ApiResponse> {
    Json(ApiResponse {
        message: "Backend is running!".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    })
}
