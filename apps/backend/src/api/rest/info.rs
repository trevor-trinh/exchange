use axum::{extract::State, response::Json};

use crate::errors::{ErrorResponse, Result};
use crate::models::api::{InfoRequest, InfoResponse};

/// Get information about tokens, markets, etc.
#[utoipa::path(
    post,
    path = "/api/info",
    request_body = InfoRequest,
    responses(
        (status = 200, description = "Success", body = InfoResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "info"
)]
pub async fn info(
    State(_state): State<crate::AppState>,
    Json(request): Json<InfoRequest>,
) -> Result<Json<InfoResponse>> {
    match request {
        InfoRequest::TokenDetails { ticker } => {
            let token = _state.db.get_token(&ticker).await?;
            Ok(Json(InfoResponse::TokenDetails { token }))
        }
        InfoRequest::MarketDetails { market_id } => {
            let market = _state.db.get_market(&market_id).await?;
            Ok(Json(InfoResponse::MarketDetails {
                market: market.into(),
            }))
        }
        InfoRequest::AllMarkets => {
            let markets = _state.db.list_markets().await?;
            Ok(Json(InfoResponse::AllMarkets {
                markets: markets.into_iter().map(|m| m.into()).collect(),
            }))
        }
        InfoRequest::AllTokens => {
            let tokens = _state.db.list_tokens().await?;
            Ok(Json(InfoResponse::AllTokens { tokens }))
        }
    }
}
