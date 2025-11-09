use crate::models::api::{CandlesRequest, CandlesResponse};
use crate::AppState;
use axum::{extract::State, Json};

/// Get OHLCV candles for a market
///
/// POST /api/candles
#[utoipa::path(
    post,
    path = "/api/candles",
    request_body = CandlesRequest,
    responses(
        (status = 200, description = "Candles retrieved successfully", body = CandlesResponse),
        (status = 400, description = "Invalid parameters"),
        (status = 500, description = "Internal server error")
    ),
    tag = "candles"
)]
pub async fn candles(
    State(state): State<AppState>,
    Json(params): Json<CandlesRequest>,
) -> Result<Json<CandlesResponse>, String> {
    // Validate interval
    if !["1m", "5m", "15m", "1h", "1d"].contains(&params.interval.as_str()) {
        return Err("Invalid interval. Must be one of: 1m, 5m, 15m, 1h, 1d".to_string());
    }

    // Query candles through the db layer
    let candles = state
        .db
        .get_candles_for_api(
            &params.market_id,
            &params.interval,
            params.from,
            params.to,
            params.count_back,
        )
        .await
        .map_err(|e| format!("Failed to query candles: {}", e))?;

    Ok(Json(CandlesResponse { candles }))
}
