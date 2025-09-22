use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod api;
mod analytics;
mod chains;
mod contracts;
mod defi;
mod dex;
mod security;
mod wallets;

use crate::api::ApiState;

#[derive(OpenApi)]
#[openapi(
    paths(
        api::health::health_check,
        api::portfolio::get_portfolio,
        api::dex::get_swap_quote,
    ),
    components(schemas(
        api::models::HealthResponse,
        api::models::Portfolio,
        api::models::SwapQuote,
    )),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "portfolio", description = "Portfolio management"),
        (name = "dex", description = "DEX integration endpoints"),
    ),
    info(
        title = "Blockchain Demo API",
        version = "0.1.0",
        description = "A comprehensive Rust blockchain demo API",
        contact(
            name = "Blockchain Developer",
            email = "dev@example.com"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "blockchain_demo=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Blockchain Demo application...");

    // Load configuration
    let config = load_config().await?;
    
    // Initialize application state
    let state = Arc::new(ApiState::new(config).await?);

    // Build the application router
    let app = Router::new()
        .route("/", get(root_handler))
        .nest("/api/v1", api::routes())
        .route("/api-docs/openapi.json", get(|| async { axum::Json(ApiDoc::openapi()) }))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Start the server
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    info!("Server running on http://0.0.0.0:3000");
    info!("Swagger UI available at http://0.0.0.0:3000/swagger-ui");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn root_handler() -> Json<Value> {
    Json(json!({
        "name": "Blockchain Demo API",
        "version": "0.1.0",
        "description": "A comprehensive Rust blockchain demo showcasing professional-level crypto development",
        "endpoints": {
            "health": "/api/v1/health",
            "portfolio": "/api/v1/portfolio",
            "dex": "/api/v1/dex",
            "defi": "/api/v1/defi",
            "swagger": "/swagger-ui"
        }
    }))
}

async fn load_config() -> Result<config::Config> {
    let settings = config::Config::builder()
        .add_source(config::Environment::with_prefix("BLOCKCHAIN_DEMO"))
        .build()?;

    Ok(settings)
}
