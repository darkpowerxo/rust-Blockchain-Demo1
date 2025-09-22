use axum::{extract::State, response::Json, routing::get, Router};
use serde::Serialize;
use std::sync::Arc;
use utoipa::ToSchema;

use crate::api::ApiState;

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: String,
    pub services: ServiceStatus,
}

#[derive(Serialize, ToSchema)]
pub struct ServiceStatus {
    pub chains: Vec<ChainHealth>,
    pub database: bool,
    pub dex_services: bool,
    pub defi_services: bool,
}

#[derive(Serialize, ToSchema)]
pub struct ChainHealth {
    pub chain_id: u64,
    pub name: String,
    pub rpc_healthy: bool,
    pub block_height: Option<u64>,
    pub gas_price: Option<String>,
}

pub fn routes() -> Router<Arc<ApiState>> {
    Router::new().route("/", get(health_check))
}

#[utoipa::path(
    get,
    path = "/api/v1/health",
    responses(
        (status = 200, description = "Health check successful", body = HealthResponse)
    ),
    tag = "health"
)]
pub async fn health_check(State(state): State<Arc<ApiState>>) -> Json<HealthResponse> {
    let chains = state.chain_manager.health_check().await;
    
    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        services: ServiceStatus {
            chains,
            database: true, // TODO: Implement actual DB health check
            dex_services: true, // TODO: Implement actual DEX health check
            defi_services: true, // TODO: Implement actual DeFi health check
        },
    };

    Json(response)
}
