use axum::{
    routing::{get, post},
    Router,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::models::ApiResponse;

pub mod admin;
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
    ),
    components(
        schemas(
            ApiResponse,
            // Info types
            crate::models::api::InfoRequest,
            crate::models::api::InfoResponse,
            crate::models::api::InfoErrorResponse,
            // User types
            crate::models::api::UserRequest,
            crate::models::api::UserResponse,
            crate::models::api::UserErrorResponse,
            // Trade types
            crate::models::api::TradeRequest,
            crate::models::api::TradeResponse,
            crate::models::api::TradeErrorResponse,
            // Drip types
            crate::models::api::DripRequest,
            crate::models::api::DripResponse,
            crate::models::api::DripErrorResponse,
            // Admin types
            crate::models::api::AdminRequest,
            crate::models::api::AdminResponse,
            crate::models::api::AdminErrorResponse,
            // Domain types
            crate::models::domain::Token,
            crate::models::domain::Market,
            crate::models::domain::Order,
            crate::models::domain::Trade,
            crate::models::domain::Balance,
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
        (name = "admin", description = "Admin operations (test/dev only)")
    )
)]
pub struct ApiDoc;

pub fn create_rest() -> Router<crate::AppState> {
    Router::new()
        .route("/api/health", get(health::health_check))
        .route("/api/info", post(info::info))
        .route("/api/user", post(user::user))
        .route("/api/trade", post(trade::trade))
        .route("/api/drip", post(drip::drip))
        .route("/api/admin", post(admin::admin_handler))
        .merge(SwaggerUi::new("/api/docs").url("/api/openapi.json", ApiDoc::openapi()))
}
