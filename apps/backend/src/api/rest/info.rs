use axum::{
    extract::{Query, State},
    response::Json,
};

use crate::models::api::{
    AllMarketsResponse, AllTokensResponse, MarketInfoRequest, MarketInfoResponse, TokenInfoRequest,
    TokenInfoResponse,
};

/// Get information about a specific token
#[utoipa::path(
    get,
    path = "/api/info/token",
    params(
        ("ticker" = String, Query, description = "Token ticker symbol")
    ),
    responses(
        (status = 200, description = "Token information", body = TokenInfoResponse),
        (status = 404, description = "Token not found")
    ),
    tag = "info"
)]
pub async fn get_token_info(
    State(state): State<crate::AppState>,
    Query(params): Query<TokenInfoRequest>,
) -> Result<Json<TokenInfoResponse>, axum::http::StatusCode> {
    match state.db.get_token(&params.ticker).await {
        Ok(token) => Ok(Json(TokenInfoResponse { token })),
        Err(_) => Err(axum::http::StatusCode::NOT_FOUND),
    }
}

/// Get information about a specific market
#[utoipa::path(
    get,
    path = "/api/info/market",
    params(
        ("market_id" = String, Query, description = "Market ID (e.g., 'BTC/USD')")
    ),
    responses(
        (status = 200, description = "Market information", body = MarketInfoResponse),
        (status = 404, description = "Market not found")
    ),
    tag = "info"
)]
pub async fn get_market_info(
    State(state): State<crate::AppState>,
    Query(params): Query<MarketInfoRequest>,
) -> Result<Json<MarketInfoResponse>, axum::http::StatusCode> {
    match state.db.get_market(&params.market_id).await {
        Ok(market) => Ok(Json(MarketInfoResponse { market })),
        Err(_) => Err(axum::http::StatusCode::NOT_FOUND),
    }
}

/// Get all available tokens
#[utoipa::path(
    get,
    path = "/api/info/tokens",
    responses(
        (status = 200, description = "List of all tokens", body = AllTokensResponse)
    ),
    tag = "info"
)]
pub async fn get_all_tokens(
    State(state): State<crate::AppState>,
) -> Result<Json<AllTokensResponse>, axum::http::StatusCode> {
    match state.db.list_tokens().await {
        Ok(tokens) => Ok(Json(AllTokensResponse { tokens })),
        Err(_) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get all available markets
#[utoipa::path(
    get,
    path = "/api/info/markets",
    responses(
        (status = 200, description = "List of all markets", body = AllMarketsResponse)
    ),
    tag = "info"
)]
pub async fn get_all_markets(
    State(state): State<crate::AppState>,
) -> Result<Json<AllMarketsResponse>, axum::http::StatusCode> {
    match state.db.list_markets().await {
        Ok(markets) => Ok(Json(AllMarketsResponse { markets })),
        Err(_) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}
