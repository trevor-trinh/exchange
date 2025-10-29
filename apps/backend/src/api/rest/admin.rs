use crate::models::api::{AdminErrorResponse, AdminRequest, AdminResponse};
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
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "admin"
)]
pub async fn admin_handler(
    State(state): State<AppState>,
    Json(request): Json<AdminRequest>,
) -> Result<Json<AdminResponse>, Json<AdminErrorResponse>> {
    match request {
        AdminRequest::CreateToken {
            ticker,
            decimals,
            name,
        } => {
            let token = state
                .db
                .create_token(ticker, decimals, name)
                .await
                .map_err(|e| {
                    Json(AdminErrorResponse {
                        error: e.to_string(),
                        code: "CREATE_TOKEN_ERROR".to_string(),
                    })
                })?;

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
            let tick_size_u128 = tick_size.parse::<u128>().map_err(|e| {
                Json(AdminErrorResponse {
                    error: format!("Invalid tick_size: {}", e),
                    code: "INVALID_TICK_SIZE".to_string(),
                })
            })?;

            let lot_size_u128 = lot_size.parse::<u128>().map_err(|e| {
                Json(AdminErrorResponse {
                    error: format!("Invalid lot_size: {}", e),
                    code: "INVALID_LOT_SIZE".to_string(),
                })
            })?;

            let min_size_u128 = min_size.parse::<u128>().map_err(|e| {
                Json(AdminErrorResponse {
                    error: format!("Invalid min_size: {}", e),
                    code: "INVALID_MIN_SIZE".to_string(),
                })
            })?;

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
                .await
                .map_err(|e| {
                    Json(AdminErrorResponse {
                        error: e.to_string(),
                        code: "CREATE_MARKET_ERROR".to_string(),
                    })
                })?;

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
            let amount_u128 = amount.parse::<u128>().map_err(|e| {
                Json(AdminErrorResponse {
                    error: format!("Invalid amount: {}", e),
                    code: "INVALID_AMOUNT".to_string(),
                })
            })?;

            // Create user if doesn't exist
            let _ = state.db.create_user(user_address.clone()).await;

            // Add balance
            let balance = state
                .db
                .add_balance(&user_address, &token_ticker, amount_u128)
                .await
                .map_err(|e| {
                    Json(AdminErrorResponse {
                        error: e.to_string(),
                        code: "FAUCET_ERROR".to_string(),
                    })
                })?;

            Ok(Json(AdminResponse::Faucet {
                user_address,
                token_ticker,
                amount,
                new_balance: balance.amount.to_string(),
            }))
        }
    }
}
