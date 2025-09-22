use axum::{extract::State, response::Json, routing::get, Router};
use std::sync::Arc;

use crate::api::ApiState;

pub fn routes() -> Router<Arc<ApiState>> {
    Router::new()
        .route("/yield", get(get_yield_opportunities))
        .route("/lending", get(get_lending_positions))
}

pub async fn get_yield_opportunities(State(_state): State<Arc<ApiState>>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "opportunities": []
    }))
}

pub async fn get_lending_positions(State(_state): State<Arc<ApiState>>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "positions": []
    }))
}
