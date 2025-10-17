use axum::Router;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod api;
pub mod models;

use api::rest;
use models::ApiResponse;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Exchange API",
        version = "0.1.0",
        description = "API for the Exchange"
    ),
    paths(
        rest::health::health_check,
    ),
    components(
        schemas(ApiResponse)
    ),
    tags(
        (name = "api", description = "General API endpoints")
    )
)]
pub struct ApiDoc;

pub async fn create_app() -> Router {
    rest::create_routes()
        .merge(SwaggerUi::new("/api/docs").url("/api/openapi.json", ApiDoc::openapi()))
        .layer(CorsLayer::permissive())
}
