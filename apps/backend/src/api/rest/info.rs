use axum::{extract::State, response::Json};

use crate::models::api::{InfoErrorResponse, InfoRequest, InfoResponse};

/// Get information about tokens, markets, etc.
#[utoipa::path(
    post,
    path = "/api/info",
    request_body = InfoRequest,
    responses(
        (status = 200, description = "Success", body = InfoResponse),
        (status = 400, description = "Invalid request", body = InfoErrorResponse),
        (status = 404, description = "Resource not found", body = InfoErrorResponse),
        (status = 500, description = "Internal server error", body = InfoErrorResponse)
    ),
    tag = "info"
)]
pub async fn info(
    State(_state): State<crate::AppState>,
    Json(request): Json<InfoRequest>,
) -> Result<Json<InfoResponse>, Json<InfoErrorResponse>> {
    match request {
        InfoRequest::TokenDetails { ticker } => {
            let token = _state.db.get_token(&ticker).await.map_err(|e| {
                Json(InfoErrorResponse {
                    error: format!("Failed to get token: {}", e),
                    code: "TOKEN_NOT_FOUND".to_string(),
                })
            })?;
            Ok(Json(InfoResponse::TokenDetails { token }))
        }
        InfoRequest::MarketDetails { market_id } => {
            let market = _state.db.get_market(&market_id).await.map_err(|e| {
                Json(InfoErrorResponse {
                    error: format!("Failed to get market: {}", e),
                    code: "MARKET_NOT_FOUND".to_string(),
                })
            })?;
            Ok(Json(InfoResponse::MarketDetails {
                market: market.into(),
            }))
        }
        InfoRequest::AllMarkets => {
            let markets = _state.db.list_markets().await.map_err(|e| {
                Json(InfoErrorResponse {
                    error: format!("Failed to list markets: {}", e),
                    code: "LIST_MARKETS_ERROR".to_string(),
                })
            })?;
            Ok(Json(InfoResponse::AllMarkets {
                markets: markets.into_iter().map(|m| m.into()).collect(),
            }))
        }
        InfoRequest::AllTokens => {
            let tokens = _state.db.list_tokens().await.map_err(|e| {
                Json(InfoErrorResponse {
                    error: format!("Failed to list tokens: {}", e),
                    code: "LIST_TOKENS_ERROR".to_string(),
                })
            })?;
            Ok(Json(InfoResponse::AllTokens { tokens }))
        }
    }
}
