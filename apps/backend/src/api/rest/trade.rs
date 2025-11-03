use axum::{extract::State, response::Json};
use chrono::Utc;
use uuid::Uuid;

use crate::errors::{ErrorResponse, ExchangeError, Result};
use crate::models::api::{TradeRequest, TradeResponse};
use crate::models::domain::{EngineRequest, Market, Order, OrderStatus};
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

            // Get market config for validation
            let market = state.db.get_market(&market_id).await?;

            // Validate order parameters
            validate_order_params(price_value, size_value, &market)?;

            // Get token decimals for proper calculation
            let base_token = state.db.get_token(&market.base_ticker).await?;

            // Lock balance based on order side
            // BUY orders: lock quote token = (price_atoms * size_atoms) / 10^base_decimals
            // SELL orders: lock base token (size)
            let (token_to_lock, amount_to_lock) = match side {
                crate::models::domain::Side::Buy => {
                    // For buy orders, need to lock quote token amount
                    // quote_amount = (price_atoms * size_atoms) / 10^base_decimals
                    let divisor = 10u128.pow(base_token.decimals as u32);
                    let quote_amount = price_value
                        .checked_mul(size_value)
                        .and_then(|v| v.checked_div(divisor))
                        .ok_or(ExchangeError::OrderValueOverflow)?;
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
                .await?;

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
                .map_err(|_| ExchangeError::EngineSendFailed)?;

            // Wait for response
            let result = response_rx
                .await
                .map_err(|_| ExchangeError::EngineReceiveFailed)?;

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

                ExchangeError::InvalidParameter {
                    message: e.to_string(),
                }
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

/// Validate order parameters against market configuration
fn validate_order_params(price: u128, size: u128, market: &Market) -> Result<()> {
    // Validate tick size (price must be multiple of tick_size)
    if !price.is_multiple_of(market.tick_size) {
        return Err(ExchangeError::InvalidTickSize);
    }

    // Validate lot size (size must be multiple of lot_size)
    if !size.is_multiple_of(market.lot_size) {
        return Err(ExchangeError::InvalidLotSize);
    }

    // Validate minimum order size
    if size < market.min_size {
        return Err(ExchangeError::SizeBelowMinimum);
    }

    Ok(())
}
