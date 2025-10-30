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

            // Lock balance based on order side
            // BUY orders: lock quote token (price * size)
            // SELL orders: lock base token (size)
            let (token_to_lock, amount_to_lock) = match side {
                crate::models::domain::Side::Buy => {
                    // For buy orders, need to lock quote token amount
                    let quote_amount = price_value.checked_mul(size_value).ok_or_else(|| {
                        (
                            StatusCode::BAD_REQUEST,
                            Json(TradeErrorResponse {
                                error: "Order value overflow".to_string(),
                                code: "ORDER_VALUE_OVERFLOW".to_string(),
                            }),
                        )
                    })?;
                    (market.quote_ticker.clone(), quote_amount)
                }
                crate::models::domain::Side::Sell => {
                    // For sell orders, need to lock base token amount
                    (market.base_ticker.clone(), size_value)
                }
            };

            // Lock the required balance (this will fail if insufficient balance)
            state
                .db
                .lock_balance(&user_address, &token_to_lock, amount_to_lock)
                .await
                .map_err(|e| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(TradeErrorResponse {
                            error: e.to_string(),
                            code: "INSUFFICIENT_BALANCE".to_string(),
                        }),
                    )
                })?;

            // Create order
            let order = Order {
                id: Uuid::new_v4(),
                user_address: user_address.clone(),
                market_id: market_id.clone(),
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
                // If order placement failed, unlock the balance
                let db_clone = state.db.clone();
                let user_addr = user_address.clone();
                let token = token_to_lock.clone();
                tokio::spawn(async move {
                    let _ = db_clone
                        .unlock_balance(&user_addr, &token, amount_to_lock)
                        .await;
                });

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

        TradeRequest::CancelAllOrders {
            user_address,
            market_id,
            signature: _,
        } => {
            // TODO: Verify signature

            // Create engine request
            let (response_tx, response_rx) = tokio::sync::oneshot::channel();
            let engine_request = EngineRequest::CancelAllOrders {
                user_address: user_address.clone(),
                market_id: market_id.clone(),
                response_tx,
            };

            // Send to engine
            state.engine_tx.send(engine_request).await.map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(TradeErrorResponse {
                        error: "Failed to send request to engine".to_string(),
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
                        code: "CANCEL_ALL_ORDERS_ERROR".to_string(),
                    }),
                )
            })?;

            Ok(Json(TradeResponse::CancelAllOrders {
                cancelled_order_ids: cancelled.cancelled_order_ids,
                count: cancelled.count,
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
