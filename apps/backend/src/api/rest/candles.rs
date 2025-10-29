use crate::AppState;
use axum::{
    extract::{Query, State},
    Json,
};
use clickhouse::Row;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CandlesQuery {
    pub market_id: String,
    pub interval: String, // 1m, 5m, 15m, 1h, 1d
    pub from: i64,        // Unix timestamp in seconds
    pub to: i64,          // Unix timestamp in seconds
}

#[derive(Debug, Serialize, Deserialize, Row, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Candle {
    pub timestamp: u32,
    pub open: u128,
    pub high: u128,
    pub low: u128,
    pub close: u128,
    pub volume: u128,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CandlesResponse {
    pub candles: Vec<Candle>,
}

/// Get OHLCV candles for a market
///
/// GET /api/candles?market_id=BTC/USDC&interval=1m&from=1234567890&to=1234567899
#[utoipa::path(
    get,
    path = "/api/candles",
    params(
        ("market_id" = String, Query, description = "Market ID"),
        ("interval" = String, Query, description = "Candle interval: 1m, 5m, 15m, 1h, 1d"),
        ("from" = i64, Query, description = "Start timestamp (Unix seconds)"),
        ("to" = i64, Query, description = "End timestamp (Unix seconds)")
    ),
    responses(
        (status = 200, description = "Candles retrieved successfully", body = CandlesResponse),
        (status = 400, description = "Invalid parameters"),
        (status = 500, description = "Internal server error")
    ),
    tag = "market-data"
)]
pub async fn get_candles(
    State(state): State<AppState>,
    Query(params): Query<CandlesQuery>,
) -> Result<Json<CandlesResponse>, String> {
    // Validate interval
    if !["1m", "5m", "15m", "1h", "1d"].contains(&params.interval.as_str()) {
        return Err("Invalid interval. Must be one of: 1m, 5m, 15m, 1h, 1d".to_string());
    }

    // Query ClickHouse for candles
    let query = format!(
        "SELECT
            toUnixTimestamp(timestamp) as timestamp,
            open,
            high,
            low,
            close,
            volume
        FROM exchange.candles
        WHERE market_id = '{}'
          AND interval = '{}'
          AND timestamp >= toDateTime({})
          AND timestamp <= toDateTime({})
        ORDER BY timestamp ASC",
        params.market_id, params.interval, params.from, params.to
    );

    let candles: Vec<Candle> = state
        .db
        .clickhouse
        .query(&query)
        .fetch_all()
        .await
        .map_err(|e| format!("Failed to query candles: {}", e))?;

    Ok(Json(CandlesResponse { candles }))
}
