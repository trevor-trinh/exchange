use axum::{extract::State, response::Json};

use crate::errors::{ErrorResponse, Result};
use crate::models::api::{UserRequest, UserResponse};

/// Get user-specific data (orders, balances, trades)
#[utoipa::path(
    post,
    path = "/api/user",
    request_body = UserRequest,
    responses(
        (status = 200, description = "Success", body = UserResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 404, description = "User or resource not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "user"
)]
pub async fn user(
    State(state): State<crate::AppState>,
    Json(request): Json<UserRequest>,
) -> Result<Json<UserResponse>> {
    match request {
        UserRequest::Orders {
            user_address,
            market_id,
            status,
            limit,
        } => {
            // Parse status string to OrderStatus enum if provided
            use crate::models::domain::OrderStatus;
            let status_enum = status.as_ref().and_then(|s| match s.as_str() {
                "pending" => Some(OrderStatus::Pending),
                "partially_filled" => Some(OrderStatus::PartiallyFilled),
                "filled" => Some(OrderStatus::Filled),
                "cancelled" => Some(OrderStatus::Cancelled),
                _ => None,
            });

            let orders = state
                .db
                .get_user_orders(
                    &user_address,
                    market_id.as_deref(),
                    status_enum,
                    limit.unwrap_or(100),
                )
                .await?;

            Ok(Json(UserResponse::Orders {
                orders: orders.into_iter().map(|o| o.into()).collect(),
            }))
        }
        UserRequest::Balances { user_address } => {
            let balances = state.db.list_balances_by_user(&user_address).await?;

            Ok(Json(UserResponse::Balances {
                balances: balances.into_iter().map(|b| b.into()).collect(),
            }))
        }
        UserRequest::Trades {
            user_address,
            market_id,
            limit,
        } => {
            let trades = state
                .db
                .get_user_trades(&user_address, market_id.as_deref(), limit.unwrap_or(100))
                .await?;

            Ok(Json(UserResponse::Trades {
                trades: trades.into_iter().map(|t| t.into()).collect(),
            }))
        }
    }
}
