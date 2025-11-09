use axum::{
    routing::{get, post},
    Router,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::models::ApiResponse;

pub mod admin;
pub mod candles;
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
        info::info,
        user::user,
        trade::trade,
        drip::drip,
        admin::admin_handler,
        candles::candles,
    ),
    components(
        schemas(
            ApiResponse,
            // Unified error response
            crate::errors::ErrorResponse,
            // Info types
            crate::models::api::InfoRequest,
            crate::models::api::InfoResponse,
            // User types
            crate::models::api::UserRequest,
            crate::models::api::UserResponse,
            // Trade types
            crate::models::api::TradeRequest,
            crate::models::api::TradeResponse,
            // Drip types
            crate::models::api::DripRequest,
            crate::models::api::DripResponse,
            // Admin types
            crate::models::api::AdminRequest,
            crate::models::api::AdminResponse,
            // Candles types
            crate::models::api::CandlesRequest,
            crate::models::api::ApiCandle,
            crate::models::api::CandlesResponse,
            // API types (only expose API layer in OpenAPI, not domain)
            crate::models::domain::Token,
            crate::models::api::ApiMarket,
            crate::models::api::ApiOrder,
            crate::models::api::ApiTrade,
            crate::models::api::ApiBalance,
            // Enums are shared between API and domain
            crate::models::domain::Side,
            crate::models::domain::OrderType,
            crate::models::domain::OrderStatus,
        )
    ),
    tags(
        (name = "api", description = "General API endpoints"),
        (name = "info", description = "Information endpoints"),
        (name = "user", description = "User data endpoints"),
        (name = "trade", description = "Trading endpoints"),
        (name = "drip", description = "Get free money"),
        (name = "admin", description = "Admin operations (test/dev only)"),
        (name = "candles", description = "OHLCV candle data")
    )
)]
pub struct ApiDoc;

pub fn create_rest() -> Router<crate::AppState> {
    Router::new()
        .route("/api/health", get(health::health_check))
        .route("/api/info", post(info::info))
        .route("/api/user", post(user::user))
        .route("/api/trade", post(trade::trade))
        .route("/api/candles", post(candles::candles))
        .route("/api/drip", post(drip::drip))
        .route("/api/admin", post(admin::admin_handler))
        .merge(SwaggerUi::new("/api/docs").url("/api/openapi.json", ApiDoc::openapi()))
}
