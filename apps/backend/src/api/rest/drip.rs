use axum::{extract::State, response::Json};

use crate::models::api::{DripErrorResponse, DripTokensRequest, DripTokensResponse};

/// Drip tokens to a user (testing/development only)
#[utoipa::path(
    post,
    path = "/api/drip/tokens",
    request_body = DripTokensRequest,
    responses(
        (status = 200, description = "Tokens dripped successfully", body = DripTokensResponse),
        (status = 400, description = "Invalid request parameters", body = DripErrorResponse),
        (status = 401, description = "Invalid signature", body = DripErrorResponse),
        (status = 404, description = "Token not found", body = DripErrorResponse),
        (status = 500, description = "Internal server error", body = DripErrorResponse)
    ),
    tag = "drip"
)]
pub async fn drip_tokens(
    State(state): State<crate::AppState>,
    Json(payload): Json<DripTokensRequest>,
) -> Result<Json<DripTokensResponse>, Json<DripErrorResponse>> {
    todo!()
}
