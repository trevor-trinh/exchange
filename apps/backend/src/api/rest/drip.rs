use axum::{extract::State, response::Json};

use crate::errors::{ErrorResponse, ExchangeError, Result};
use crate::models::api::{DripRequest, DripResponse};

/// Drip tokens to users (testing/development faucet)
#[utoipa::path(
    post,
    path = "/api/drip",
    request_body = DripRequest,
    responses(
        (status = 200, description = "Tokens dripped successfully", body = DripResponse),
        (status = 400, description = "Invalid request parameters", body = ErrorResponse),
        (status = 401, description = "Invalid signature", body = ErrorResponse),
        (status = 404, description = "Token not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "drip"
)]
pub async fn drip(
    State(state): State<crate::AppState>,
    Json(request): Json<DripRequest>,
) -> Result<Json<DripResponse>> {
    match request {
        DripRequest::Faucet {
            user_address,
            token_ticker,
            amount,
            signature: _,
        } => {
            // TODO: Verify signature (skip for dev/test faucet)

            // Parse amount from string to u128
            let amount_value = amount
                .parse::<u128>()
                .map_err(|_| ExchangeError::InvalidAmount)?;

            // Check token exists
            state.db.get_token(&token_ticker).await?;

            // Create user if doesn't exist
            let _ = state.db.create_user(user_address.clone()).await;

            // Add balance
            let new_balance = state
                .db
                .add_balance(&user_address, &token_ticker, amount_value)
                .await?;

            Ok(Json(DripResponse::Faucet {
                user_address,
                token_ticker,
                amount,
                new_balance: new_balance.amount.to_string(),
            }))
        }
    }
}
