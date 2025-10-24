use axum::{extract::State, response::Json};
use uuid::Uuid;

use crate::models::api::{
    CancelOrderRequest, OrderCancelled, OrderPlaced, PlaceOrderRequest, TradeErrorResponse,
};
use crate::models::domain::{Order, OrderType};

/// Place a new order
#[utoipa::path(
    post,
    path = "/api/trade/order",
    request_body = PlaceOrderRequest,
    responses(
        (status = 200, description = "Order placed successfully", body = OrderPlaced),
        (status = 400, description = "Invalid order parameters", body = TradeErrorResponse),
        (status = 401, description = "Invalid signature", body = TradeErrorResponse),
        (status = 500, description = "Internal server error", body = TradeErrorResponse)
    ),
    tag = "trade"
)]
pub async fn place_order(
    State(state): State<crate::AppState>,
    Json(payload): Json<PlaceOrderRequest>,
) -> Result<Json<OrderPlaced>, Json<TradeErrorResponse>> {
    todo!()
}

/// Cancel an existing order
#[utoipa::path(
    post,
    path = "/api/trade/cancel",
    request_body = CancelOrderRequest,
    responses(
        (status = 200, description = "Order cancelled successfully", body = OrderCancelled),
        (status = 400, description = "Invalid request parameters", body = TradeErrorResponse),
        (status = 401, description = "Invalid signature", body = TradeErrorResponse),
        (status = 404, description = "Order not found", body = TradeErrorResponse),
        (status = 500, description = "Internal server error", body = TradeErrorResponse)
    ),
    tag = "trade"
)]
pub async fn cancel_order(
    State(state): State<crate::AppState>,
    Json(payload): Json<CancelOrderRequest>,
) -> Result<Json<OrderCancelled>, Json<TradeErrorResponse>> {
    todo!()
}
}
