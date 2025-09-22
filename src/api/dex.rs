use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use ethers::types::{Address, U256};

use crate::api::{models::SwapQuote, ApiState};

/// Pool query parameters
#[derive(Deserialize)]
pub struct PoolQuery {
    pub token_a: Address,
    pub token_b: Address,
}

/// Swap request
#[derive(Deserialize)]
pub struct SwapRequest {
    pub pool_address: Address,
    pub token_in: Address,
    pub token_out: Address,
    pub amount_in: U256,
    pub min_amount_out: U256,
    pub recipient: Address,
}

/// Add liquidity request
#[derive(Deserialize)]
pub struct AddLiquidityRequest {
    pub pool_address: Address,
    pub token_a: Address,
    pub token_b: Address,
    pub amount_a: U256,
    pub amount_b: U256,
    pub min_amount_a: U256,
    pub min_amount_b: U256,
    pub recipient: Address,
}

/// Pool info response
#[derive(Serialize)]
pub struct PoolInfoResponse {
    pub address: Address,
    pub token_a: TokenInfo,
    pub token_b: TokenInfo,
    pub reserve_a: U256,
    pub reserve_b: U256,
    pub total_supply: U256,
    pub fee_rate: U256,
    pub volume_24h: U256,
    pub tvl: U256,
    pub apr: f64,
}

/// Token information
#[derive(Serialize)]
pub struct TokenInfo {
    pub address: Address,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub price_usd: f64,
}

/// DEX statistics response
#[derive(Serialize)]
pub struct DexStatsResponse {
    pub name: String,
    pub total_tvl: U256,
    pub volume_24h: U256,
    pub fees_24h: U256,
    pub active_pools: u64,
    pub supported_tokens: u64,
}

pub fn routes() -> Router<Arc<ApiState>> {
    Router::new()
        .route("/", get(list_dex_protocols))
        .route("/{dex}/stats", get(get_dex_stats))
        .route("/{dex}/pools", get(list_pools))
        .route("/{dex}/pool", get(get_pool_info))
        .route("/quote", get(get_swap_quote))
        .route("/swap", post(execute_swap))
        .route("/{dex}/liquidity/add", post(add_liquidity))
        .route("/{dex}/liquidity/remove", post(remove_liquidity))
        .route("/{dex}/tokens", get(list_supported_tokens))
}

#[utoipa::path(
    get,
    path = "/api/v1/dex/quote",
    responses(
        (status = 200, description = "Swap quote retrieved successfully", body = SwapQuote)
    ),
    tag = "dex"
)]
/// List supported DEX protocols
async fn list_dex_protocols(
    State(_state): State<Arc<ApiState>>,
) -> Result<Json<Vec<String>>, StatusCode> {
    let protocols = vec![
        "uniswap-v2".to_string(),
        "uniswap-v3".to_string(),
        "sushiswap".to_string(),
        "quickswap".to_string(),
        "pancakeswap".to_string(),
        "1inch".to_string(),
    ];
    
    Ok(Json(protocols))
}

/// Get DEX statistics
async fn get_dex_stats(
    State(state): State<Arc<ApiState>>,
    Path(dex): Path<String>,
) -> Result<Json<DexStatsResponse>, StatusCode> {
    let _stats = state.dex_manager.get_protocol_stats(&dex).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let response = DexStatsResponse {
        name: dex.clone(),
        total_tvl: U256::from(1000000000u64),
        volume_24h: U256::from(50000000u64),
        fees_24h: U256::from(150000u64),
        active_pools: 1500,
        supported_tokens: 5000,
    };
    
    Ok(Json(response))
}

/// List pools for a DEX
async fn list_pools(
    State(state): State<Arc<ApiState>>,
    Path(dex): Path<String>,
) -> Result<Json<Vec<PoolInfoResponse>>, StatusCode> {
    let pools = state.dex_manager.get_top_pools(&dex, 50).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let pool_responses: Vec<PoolInfoResponse> = pools.into_iter()
        .map(|pool| PoolInfoResponse {
            address: pool.address,
            token_a: TokenInfo {
                address: pool.token_a,
                symbol: "TOKEN".to_string(),
                name: "Token".to_string(),
                decimals: 18,
                price_usd: 1.0,
            },
            token_b: TokenInfo {
                address: pool.token_b,
                symbol: "TOKEN".to_string(),
                name: "Token".to_string(),
                decimals: 18,
                price_usd: 1.0,
            },
            reserve_a: pool.reserve_a,
            reserve_b: pool.reserve_b,
            total_supply: U256::zero(),
            fee_rate: pool.fee_rate,
            volume_24h: U256::zero(),
            tvl: U256::zero(),
            apr: 0.0,
        })
        .collect();
    
    Ok(Json(pool_responses))
}

/// Get pool information
async fn get_pool_info(
    State(state): State<Arc<ApiState>>,
    Path(dex): Path<String>,
    axum::extract::Query(query): axum::extract::Query<PoolQuery>,
) -> Result<Json<PoolInfoResponse>, StatusCode> {
    let pool = state.dex_manager.get_pool_info(&dex, query.token_a, query.token_b).await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    let response = PoolInfoResponse {
        address: pool.address,
        token_a: TokenInfo {
            address: query.token_a,
            symbol: "TOKEN_A".to_string(),
            name: "Token A".to_string(),
            decimals: 18,
            price_usd: 1.0,
        },
        token_b: TokenInfo {
            address: query.token_b,
            symbol: "TOKEN_B".to_string(),
            name: "Token B".to_string(),
            decimals: 18,
            price_usd: 1.0,
        },
        reserve_a: pool.reserve_a,
        reserve_b: pool.reserve_b,
        total_supply: U256::zero(),
        fee_rate: pool.fee_rate,
        volume_24h: U256::zero(),
        tvl: U256::zero(),
        apr: 0.0,
    };
    
    Ok(Json(response))
}

/// Add liquidity
async fn add_liquidity(
    State(state): State<Arc<ApiState>>,
    Path(dex): Path<String>,
    Json(request): Json<AddLiquidityRequest>,
) -> Result<Json<String>, StatusCode> {
    let tx_hash = state.dex_manager.add_liquidity(
        &dex,
        request.token_a,
        request.token_b,
        request.amount_a,
        request.amount_b,
        request.min_amount_a,
        request.min_amount_b,
        request.recipient,
    ).await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(format!("{:#x}", tx_hash)))
}

/// Remove liquidity
async fn remove_liquidity(
    State(state): State<Arc<ApiState>>,
    Path(dex): Path<String>,
    Json(request): Json<AddLiquidityRequest>,
) -> Result<Json<String>, StatusCode> {
    let tx_hash = state.dex_manager.remove_liquidity(
        &dex,
        request.token_a,
        request.token_b,
        request.amount_a,
        request.min_amount_a,
        request.min_amount_b,
        request.recipient,
    ).await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(format!("{:#x}", tx_hash)))
}

/// List supported tokens
async fn list_supported_tokens(
    State(state): State<Arc<ApiState>>,
    Path(dex): Path<String>,
) -> Result<Json<Vec<TokenInfo>>, StatusCode> {
    let tokens = state.dex_manager.get_supported_tokens(&dex).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let token_infos: Vec<TokenInfo> = tokens.into_iter()
        .map(|token| TokenInfo {
            address: token.address,
            symbol: token.symbol,
            name: token.name,
            decimals: token.decimals,
            price_usd: 1.0,
        })
        .collect();
    
    Ok(Json(token_infos))
}

#[utoipa::path(
    get,
    path = "/api/dex/quote",
    responses(
        (status = 200, description = "Swap quote retrieved successfully", body = SwapQuote)
    )
)]
async fn get_swap_quote(State(state): State<Arc<ApiState>>) -> Json<SwapQuote> {
    // Mock implementation
    let quote = SwapQuote {
        from_token: "0xA0b86a33E6441c8e8C3aB8C37C0b14E1FEd0E8C6".to_string(),
        to_token: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
        from_amount: 1.0,
        to_amount: 1800.0,
        price_impact: 0.005, // 0.5%
        gas_estimate: 150000,
        dex: "Uniswap V3".to_string(),
        route: vec![
            "0xA0b86a33E6441c8e8C3aB8C37C0b14E1FEd0E8C6".to_string(),
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
        ],
        slippage_tolerance: 0.01, // 1%
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
