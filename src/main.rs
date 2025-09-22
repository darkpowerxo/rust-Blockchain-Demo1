use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Json, Redirect},
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::{OpenApi, openapi::OpenApiVersion};
use utoipa_swagger_ui::SwaggerUi;

mod api;
mod analytics;
mod app_config;
mod chains;
mod contracts;
mod defi;
mod dex;
mod security;
mod wallets;
// mod websocket; // Temporarily disabled due to compilation issues

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
    ),
    servers(
        (url = "http://localhost:3000", description = "Local server")
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

    // Start real-time updates
    // WebSocket support temporarily disabled
    // websocket::start_real_time_updates(Arc::clone(&state.websocket)).await;

    // Build the application router
    let app = Router::new()
        .route("/", get(root_handler))
        // .route("/ws", get(websocket::websocket_handler)) // WebSocket disabled
        .nest("/api/v1", api::routes())
        .nest("/docs", api::docs::routes())
        .route("/docs/openapi.json", get(openapi_spec_handler))
        .route("/swagger-ui", get(swagger_ui_redirect))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Start the server
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    info!("Server running on http://0.0.0.0:3000");
    info!("Swagger UI available at http://0.0.0.0:3000/swagger-ui");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn openapi_spec_handler() -> Json<Value> {
    let spec = ApiDoc::openapi();
    let mut spec_json = serde_json::to_value(spec).unwrap();
    
    // Explicitly set OpenAPI version to 3.0.3 for better Swagger UI compatibility
    if let Some(obj) = spec_json.as_object_mut() {
        obj.insert("openapi".to_string(), serde_json::Value::String("3.0.3".to_string()));
    }
    
    Json(spec_json)
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

async fn swagger_ui_redirect() -> Redirect {
    Redirect::permanent("/docs/swagger")
}

async fn load_config() -> Result<config::Config> {
    // For demo purposes, create a minimal configuration
    let settings = config::Config::builder()
        .set_default("demo_mode", true)?
        .set_default("server.host", "0.0.0.0")?
        .set_default("server.port", 3000)?
        .set_default("ethereum.rpc_url", "https://mainnet.infura.io/v3/demo")?
        .set_default("polygon.rpc_url", "https://polygon-rpc.com")?
        .set_default("arbitrum.rpc_url", "https://arb1.arbitrum.io/rpc")?
        .add_source(config::Environment::with_prefix("BLOCKCHAIN_DEMO"))
        .build()?;
    
    Ok(settings)
}
