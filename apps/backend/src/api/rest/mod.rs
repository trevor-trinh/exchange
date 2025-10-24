use axum::{
    routing::{get, post},
    Router,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::models::ApiResponse;

pub mod drip;
pub mod health;
pub mod info;
pub mod trade;
pub mod user;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Exchange API",
        version = "0.1.0",
        description = "API for the Exchange"
    ),
    paths(
        health::health_check,
        info::get_token_info,
        info::get_market_info,
        info::get_all_tokens,
        info::get_all_markets,
        user::get_user_orders,
        user::get_user_balances,
        user::get_user_trades,
        trade::place_order,
        trade::cancel_order,
        drip::drip_tokens,
        drip::get_drip_balance,
    ),
    components(
        schemas(
            ApiResponse,
            crate::models::api::TokenInfoRequest,
            crate::models::api::TokenInfoResponse,
            crate::models::api::MarketInfoRequest,
            crate::models::api::MarketInfoResponse,
            crate::models::api::AllTokensResponse,
            crate::models::api::AllMarketsResponse,
            crate::models::api::UserOrdersRequest,
            crate::models::api::UserOrdersResponse,
            crate::models::api::UserBalancesRequest,
            crate::models::api::UserBalancesResponse,
            crate::models::api::UserTradesRequest,
            crate::models::api::UserTradesResponse,
            crate::models::api::PlaceOrderRequest,
            crate::models::api::CancelOrderRequest,
            crate::models::api::OrderPlaced,
            crate::models::api::OrderCancelled,
            crate::models::api::TradeErrorResponse,
            crate::models::api::DripTokensRequest,
            crate::models::api::DripTokensResponse,
            crate::models::api::DripErrorResponse,
        )
    ),
    tags(
        (name = "api", description = "General API endpoints"),
        (name = "info", description = "Information endpoints"),
        (name = "user", description = "User data endpoints"),
        (name = "trade", description = "Trading endpoints"),
        (name = "drip", description = "Development/testing endpoints")
    )
)]
pub struct ApiDoc;

pub fn create_rest() -> Router<crate::AppState> {
    Router::new()
        // Health endpoint
        .route("/api/health", get(health::health_check))
        // Info endpoints
        .route("/api/info/token", get(info::get_token_info))
        .route("/api/info/market", get(info::get_market_info))
        .route("/api/info/tokens", get(info::get_all_tokens))
        .route("/api/info/markets", get(info::get_all_markets))
        // User endpoints
        .route("/api/user/orders", get(user::get_user_orders))
        .route("/api/user/balances", get(user::get_user_balances))
        .route("/api/user/trades", get(user::get_user_trades))
        // Trade endpoints
        .route("/api/trade/order", post(trade::place_order))
        .route("/api/trade/cancel", post(trade::cancel_order))
        // Drip endpoints (development/testing)
        .route("/api/drip/tokens", post(drip::drip_tokens))
        .route("/api/drip/balance", get(drip::get_drip_balance))
        // Documentation
        .merge(SwaggerUi::new("/api/docs").url("/api/openapi.json", ApiDoc::openapi()))
}

// request -> handler -> db -> response
// /info
// - token info
// - market info

// /user
// - orders
// - balances
// - trades

// /trade - signature required
// request -> handler -> me -> response
// - order
// - cancel

// /drip - signature required
// - drip amount of tokens
