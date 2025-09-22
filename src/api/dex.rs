use axum::{extract::State, response::Json, routing::{get, post}, Router};
use std::sync::Arc;

use crate::api::{models::SwapQuote, ApiState};

pub fn routes() -> Router<Arc<ApiState>> {
    Router::new()
        .route("/quote", get(get_swap_quote))
        .route("/swap", post(execute_swap))
}

#[utoipa::path(
    get,
    path = "/api/v1/dex/quote",
    responses(
        (status = 200, description = "Swap quote retrieved successfully", body = SwapQuote)
    ),
    tag = "dex"
)]
pub async fn get_swap_quote(State(_state): State<Arc<ApiState>>) -> Json<SwapQuote> {
    // Mock implementation
    let quote = SwapQuote {
        from_token: "0xA0b86a33E6441c8e8C3aB8C37C0b14E1FEd0E8C6".to_string(),
        to_token: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
        from_amount: rust_decimal::Decimal::from(1),
        to_amount: rust_decimal::Decimal::new(1800, 0),
        price_impact: rust_decimal::Decimal::new(5, 3), // 0.5%
        gas_estimate: 150000,
        dex: "Uniswap V3".to_string(),
        route: vec![
            "0xA0b86a33E6441c8e8C3aB8C37C0b14E1FEd0E8C6".to_string(),
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
        ],
        slippage_tolerance: rust_decimal::Decimal::new(1, 2), // 1%
    };

    Json(quote)
}

pub async fn execute_swap(
    State(_state): State<Arc<ApiState>>,
    Json(_request): Json<crate::api::models::SwapRequest>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "success",
        "tx_hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
    }))
}
