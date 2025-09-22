use axum::{extract::State, response::Json, routing::get, Router};
use std::sync::Arc;
use utoipa::ToSchema;

use crate::api::{models::Portfolio, ApiState};

pub fn routes() -> Router<Arc<ApiState>> {
    Router::new()
        .route("/", get(get_portfolio))
        .route("/{address}", get(get_portfolio_by_address))
}

#[utoipa::path(
    get,
    path = "/api/v1/portfolio",
    responses(
        (status = 200, description = "Portfolio retrieved successfully", body = Portfolio)
    ),
    tag = "portfolio"
)]
pub async fn get_portfolio(State(_state): State<Arc<ApiState>>) -> Json<Portfolio> {
    // Mock implementation
    let portfolio = Portfolio {
        id: uuid::Uuid::new_v4().to_string(),
        address: "0x1234567890123456789012345678901234567890".to_string(),
        total_value_usd: 10000.0,
        assets: vec![],
        defi_positions: vec![],
        last_updated: chrono::Utc::now().to_rfc3339(),
    };

    Json(portfolio)
}

pub async fn get_portfolio_by_address(
    State(_state): State<Arc<ApiState>>,
    axum::extract::Path(_address): axum::extract::Path<String>,
) -> Json<Portfolio> {
    get_portfolio(State(_state)).await
}
