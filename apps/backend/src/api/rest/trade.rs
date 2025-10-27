use axum::{extract::State, http::StatusCode, response::Json};
use chrono::Utc;
use uuid::Uuid;

use crate::models::api::{TradeErrorResponse, TradeRequest, TradeResponse};
use crate::models::domain::{EngineRequest, Market, Order, OrderStatus};
use tokio::sync::oneshot;

/// Execute trades (place/cancel orders)
#[utoipa::path(
    post,
    path = "/api/trade",
    request_body = TradeRequest,
    responses(
        (status = 200, description = "Success", body = TradeResponse),
        (status = 400, description = "Invalid request parameters", body = TradeErrorResponse),
        (status = 401, description = "Invalid signature", body = TradeErrorResponse),
        (status = 404, description = "Order not found", body = TradeErrorResponse),
        (status = 500, description = "Internal server error", body = TradeErrorResponse)
    ),
    tag = "trade"
)]
pub async fn trade(
    State(state): State<crate::AppState>,
    Json(request): Json<TradeRequest>,
) -> Result<Json<TradeResponse>, (StatusCode, Json<TradeErrorResponse>)> {
    match request {
        TradeRequest::PlaceOrder {
            user_address,
            market_id,
            side,
            order_type,
            price,
            size,
            signature: _,
        } => {
            // TODO: Verify signature

            // Parse price and size from strings to u128
            let price_value = price.parse::<u128>().map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(TradeErrorResponse {
                        error: "Invalid price format".to_string(),
                        code: "INVALID_PRICE".to_string(),
                    }),
                )
            })?;

            let size_value = size.parse::<u128>().map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(TradeErrorResponse {
                        error: "Invalid size format".to_string(),
                        code: "INVALID_SIZE".to_string(),
                    }),
                )
            })?;

            // Get market config for validation
            let market = state.db.get_market(&market_id).await.map_err(|e| {
                (
                    StatusCode::NOT_FOUND,
                    Json(TradeErrorResponse {
                        error: format!("Market not found: {}", e),
                        code: "MARKET_NOT_FOUND".to_string(),
                    }),
                )
            })?;

            // Validate order parameters
            validate_order_params(price_value, size_value, &market)?;

            // Create order
            let order = Order {
                id: Uuid::new_v4(),
                user_address,
                market_id,
                side,
                order_type,
                price: price_value,
                size: size_value,
                filled_size: 0,
                status: OrderStatus::Pending,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            // Send to matching engine
            let (response_tx, response_rx) = oneshot::channel();
            state
                .engine_tx
                .send(EngineRequest::PlaceOrder { order, response_tx })
                .await
                .map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(TradeErrorResponse {
                            error: "Failed to send order to engine".to_string(),
                            code: "ENGINE_ERROR".to_string(),
                        }),
                    )
                })?;

            // Wait for response
            let result = response_rx.await.map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(TradeErrorResponse {
                        error: "Failed to receive response from engine".to_string(),
                        code: "ENGINE_ERROR".to_string(),
                    }),
                )
            })?;

            let placed = result.map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(TradeErrorResponse {
                        error: e.to_string(),
                        code: "PLACE_ORDER_ERROR".to_string(),
                    }),
                )
            })?;

            Ok(Json(TradeResponse::PlaceOrder {
                order: placed.order,
                trades: placed.trades,
            }))
        }
        TradeRequest::CancelOrder {
            user_address,
            order_id,
            signature: _,
        } => {
            // TODO: Verify signature

            // Parse order_id
            let order_uuid = Uuid::parse_str(&order_id).map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(TradeErrorResponse {
                        error: "Invalid order ID format".to_string(),
                        code: "INVALID_ORDER_ID".to_string(),
                    }),
                )
            })?;

            // Send to matching engine
            let (response_tx, response_rx) = oneshot::channel();
            state
                .engine_tx
                .send(EngineRequest::CancelOrder {
                    order_id: order_uuid,
                    user_address,
                    response_tx,
                })
                .await
                .map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(TradeErrorResponse {
                            error: "Failed to send cancel request to engine".to_string(),
                            code: "ENGINE_ERROR".to_string(),
                        }),
                    )
                })?;

            // Wait for response
            let result = response_rx.await.map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(TradeErrorResponse {
                        error: "Failed to receive response from engine".to_string(),
                        code: "ENGINE_ERROR".to_string(),
                    }),
                )
            })?;

            let cancelled = result.map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(TradeErrorResponse {
                        error: e.to_string(),
                        code: "CANCEL_ORDER_ERROR".to_string(),
                    }),
                )
            })?;

            Ok(Json(TradeResponse::CancelOrder {
                order_id: cancelled.order_id,
            }))
        }
    }
}

/// Validate order parameters against market configuration
fn validate_order_params(
    price: u128,
    size: u128,
    market: &Market,
) -> Result<(), (StatusCode, Json<TradeErrorResponse>)> {
    // Validate tick size (price must be multiple of tick_size)
    if price % market.tick_size != 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(TradeErrorResponse {
                error: format!(
                    "Price {} is not a multiple of tick size {}",
                    price, market.tick_size
                ),
                code: "INVALID_TICK_SIZE".to_string(),
            }),
        ));
    }

    // Validate lot size (size must be multiple of lot_size)
    if size % market.lot_size != 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(TradeErrorResponse {
                error: format!(
                    "Size {} is not a multiple of lot size {}",
                    size, market.lot_size
                ),
                code: "INVALID_LOT_SIZE".to_string(),
            }),
        ));
    }

    // Validate minimum order size
    if size < market.min_size {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(TradeErrorResponse {
                error: format!(
                    "Size {} is below minimum order size {}",
                    size, market.min_size
                ),
                code: "BELOW_MIN_SIZE".to_string(),
            }),
        ));
    }

    Ok(())
}
