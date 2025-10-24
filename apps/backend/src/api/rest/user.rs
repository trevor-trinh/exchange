use axum::{
    extract::{Query, State},
    response::Json,
};

use crate::models::api::{
    UserBalancesRequest, UserBalancesResponse, UserOrdersRequest, UserOrdersResponse,
    UserTradesRequest, UserTradesResponse,
};

/// Get user's orders
#[utoipa::path(
    get,
    path = "/api/user/orders",
    params(
        ("user_address" = String, Query, description = "User wallet address"),
        ("market_id" = Option<String>, Query, description = "Filter by market ID"),
        ("status" = Option<String>, Query, description = "Filter by order status"),
        ("limit" = Option<u32>, Query, description = "Maximum number of orders to return")
    ),
    responses(
        (status = 200, description = "User orders", body = UserOrdersResponse),
        (status = 400, description = "Invalid request parameters")
    ),
    tag = "user"
)]
pub async fn get_user_orders(
    State(state): State<crate::AppState>,
    Query(params): Query<UserOrdersRequest>,
) -> Result<Json<UserOrdersResponse>, axum::http::StatusCode> {
    todo!()
}

/// Get user's balances
#[utoipa::path(
    get,
    path = "/api/user/balances",
    params(
        ("user_address" = String, Query, description = "User wallet address")
    ),
    responses(
        (status = 200, description = "User balances", body = UserBalancesResponse),
        (status = 400, description = "Invalid request parameters")
    ),
    tag = "user"
)]
pub async fn get_user_balances(
    State(state): State<crate::AppState>,
    Query(params): Query<UserBalancesRequest>,
) -> Result<Json<UserBalancesResponse>, axum::http::StatusCode> {
    todo!()
}

/// Get user's trades
#[utoipa::path(
    get,
    path = "/api/user/trades",
    params(
        ("user_address" = String, Query, description = "User wallet address"),
        ("market_id" = Option<String>, Query, description = "Filter by market ID"),
        ("limit" = Option<u32>, Query, description = "Maximum number of trades to return")
    ),
    responses(
        (status = 200, description = "User trades", body = UserTradesResponse),
        (status = 400, description = "Invalid request parameters")
    ),
    tag = "user"
)]
pub async fn get_user_trades(
    State(state): State<crate::AppState>,
    Query(params): Query<UserTradesRequest>,
) -> Result<Json<UserTradesResponse>, axum::http::StatusCode> {
    todo!()
}
