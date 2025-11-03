use crate::errors::{ErrorResponse, ExchangeError, Result};
use crate::models::api::{AdminRequest, AdminResponse};
use crate::AppState;
use axum::{extract::State, Json};

/// Admin endpoint for test/dev operations
///
/// POST /api/admin
///
/// Handles administrative operations like creating tokens, markets, and funding accounts.
/// In production, this endpoint should be protected or disabled.
#[utoipa::path(
    post,
    path = "/api/admin",
    request_body = AdminRequest,
    responses(
        (status = 200, description = "Admin operation successful", body = AdminResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "admin"
)]
pub async fn admin_handler(
    State(state): State<AppState>,
    Json(request): Json<AdminRequest>,
) -> Result<Json<AdminResponse>> {
    match request {
        AdminRequest::CreateToken {
            ticker,
            decimals,
            name,
        } => {
            let token = state.db.create_token(ticker, decimals, name).await?;

            Ok(Json(AdminResponse::CreateToken { token }))
        }

        AdminRequest::CreateMarket {
            base_ticker,
            quote_ticker,
            tick_size,
            lot_size,
            min_size,
            maker_fee_bps,
            taker_fee_bps,
        } => {
            // Parse string values to u128
            let tick_size_u128 = tick_size.parse::<u128>()?;
            let lot_size_u128 = lot_size.parse::<u128>()?;
            let min_size_u128 = min_size.parse::<u128>()?;

            let market = state
                .db
                .create_market(
                    base_ticker,
                    quote_ticker,
                    tick_size_u128,
                    lot_size_u128,
                    min_size_u128,
                    maker_fee_bps,
                    taker_fee_bps,
                )
                .await?;

            Ok(Json(AdminResponse::CreateMarket {
                market: market.into(),
            }))
        }

        AdminRequest::Faucet {
            user_address,
            token_ticker,
            amount,
            signature: _,
        } => {
            // Parse amount string to u128
            let amount_u128 = amount
                .parse::<u128>()
                .map_err(|_| ExchangeError::InvalidAmount)?;

            // Create user if doesn't exist
            let _ = state.db.create_user(user_address.clone()).await;

            // Add balance
            let balance = state
                .db
                .add_balance(&user_address, &token_ticker, amount_u128)
                .await?;

            Ok(Json(AdminResponse::Faucet {
                user_address,
                token_ticker,
                amount,
                new_balance: balance.amount.to_string(),
            }))
        }
    }
}
