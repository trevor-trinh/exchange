use axum::{routing::get, Router};
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::db::Db;
use crate::models::ApiResponse;

pub mod health;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Exchange API",
        version = "0.1.0",
        description = "API for the Exchange"
    ),
    paths(
        health::health_check,
    ),
    components(
        schemas(ApiResponse)
    ),
    tags(
        (name = "api", description = "General API endpoints")
    )
)]
pub struct ApiDoc;

pub fn create_app(db: Db) -> Router {
    Router::new()
        .route("/api/health", get(health::health_check))
        .merge(SwaggerUi::new("/api/docs").url("/api/openapi.json", ApiDoc::openapi()))
        .layer(CorsLayer::permissive())
        .with_state(db)
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
