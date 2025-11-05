use axum::{extract::State, response::Json};
use chrono::Utc;
use uuid::Uuid;

use crate::errors::{ErrorResponse, ExchangeError, Result};
use crate::models::api::{TradeRequest, TradeResponse};
use crate::models::domain::{EngineRequest, Order, OrderStatus};
use tokio::sync::oneshot;

/// Execute trades (place/cancel orders)
#[utoipa::path(
    post,
    path = "/api/trade",
    request_body = TradeRequest,
    responses(
        (status = 200, description = "Success", body = TradeResponse),
        (status = 400, description = "Invalid request parameters", body = ErrorResponse),
        (status = 401, description = "Invalid signature", body = ErrorResponse),
        (status = 404, description = "Order not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "trade"
)]
pub async fn trade(
    State(state): State<crate::AppState>,
    Json(request): Json<TradeRequest>,
) -> Result<Json<TradeResponse>> {
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
            let price_value = price
                .parse::<u128>()
                .map_err(|_| ExchangeError::InvalidPrice)?;
            let size_value = size
                .parse::<u128>()
                .map_err(|_| ExchangeError::InvalidSize)?;

            // Create order (validation and locking happens in engine)
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

            // Send to matching engine - engine handles validation and locking
            let (response_tx, response_rx) = oneshot::channel();
            state
                .engine_tx
                .send(EngineRequest::PlaceOrder { order, response_tx })
                .await
                .map_err(|_| ExchangeError::EngineSendFailed)?;

            // Wait for response
            let placed = response_rx
                .await
                .map_err(|_| ExchangeError::EngineReceiveFailed)??;

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
            let order_uuid = Uuid::parse_str(&order_id)?;

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
                .map_err(|_| ExchangeError::EngineSendFailed)?;

            // Wait for response
            let result = response_rx
                .await
                .map_err(|_| ExchangeError::EngineReceiveFailed)?;

            let cancelled = result.map_err(|e| ExchangeError::InvalidParameter {
                message: e.to_string(),
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
            state
                .engine_tx
                .send(engine_request)
                .await
                .map_err(|_| ExchangeError::EngineSendFailed)?;

            // Wait for response
            let result = response_rx
                .await
                .map_err(|_| ExchangeError::EngineReceiveFailed)?;

            let cancelled = result.map_err(|e| ExchangeError::InvalidParameter {
                message: e.to_string(),
            })?;

            Ok(Json(TradeResponse::CancelAllOrders {
                cancelled_order_ids: cancelled.cancelled_order_ids,
                count: cancelled.count,
            }))
        }
    }
}
