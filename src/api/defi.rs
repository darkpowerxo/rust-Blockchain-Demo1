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

use crate::api::ApiState;

pub fn routes() -> Router<Arc<ApiState>> {
    Router::new()
        .route("/protocols", get(list_defi_protocols))
        .route("/protocols/{protocol}/stats", get(get_protocol_stats))
        .route("/protocols/{protocol}/supply", post(supply_asset))
        .route("/protocols/{protocol}/withdraw", post(withdraw_asset))
        .route("/protocols/{protocol}/borrow", post(borrow_asset))
        .route("/protocols/{protocol}/repay", post(repay_asset))
        .route("/opportunities", get(get_yield_opportunities))
        .route("/portfolio/{user}", get(get_user_portfolio))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProtocolStatsResponse {
    pub name: String,
    pub tvl: U256,
    pub total_borrowed: U256,
    pub total_supplied: U256,
    pub utilization_rate: f64,
    pub average_supply_apy: f64,
    pub average_borrow_apy: f64,
    pub active_users: u64,
    pub health_factor: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LendingRequest {
    pub asset: Address,
    pub amount: U256,
    pub user: Address,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YieldOpportunity {
    pub protocol: String,
    pub asset: Address,
    pub apy: f64,
    pub risk_level: String,
    pub minimum_deposit: U256,
    pub available_liquidity: U256,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPortfolioResponse {
    pub user: Address,
    pub total_supplied_usd: f64,
    pub total_borrowed_usd: f64,
    pub net_worth_usd: f64,
    pub overall_health_factor: f64,
    pub positions: Vec<PositionInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PositionInfo {
    pub protocol: String,
    pub asset: Address,
    pub supplied_amount: U256,
    pub borrowed_amount: U256,
    pub supply_apy: f64,
    pub borrow_apy: f64,
}

/// List supported DeFi protocols
async fn list_defi_protocols(
    State(_state): State<Arc<ApiState>>,
) -> Result<Json<Vec<String>>, StatusCode> {
    let protocols = vec![
        "aave".to_string(),
        "compound".to_string(),
        "maker".to_string(),
        "yearn".to_string(),
        "curve".to_string(),
        "lido".to_string(),
    ];
    
    Ok(Json(protocols))
}

/// Get protocol statistics
async fn get_protocol_stats(
    State(state): State<Arc<ApiState>>,
    Path(protocol): Path<String>,
) -> Result<Json<ProtocolStatsResponse>, StatusCode> {
    let chain_id = 1u64; // Default to Ethereum mainnet
    let _stats = state.defi_manager.get_protocol_stats(chain_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let response = ProtocolStatsResponse {
        name: protocol.clone(),
        tvl: U256::from(5000000000u64), // $5B
        total_borrowed: U256::from(2000000000u64), // $2B
        total_supplied: U256::from(5000000000u64), // $5B
        utilization_rate: 0.4, // 40%
        average_supply_apy: 0.035, // 3.5%
        average_borrow_apy: 0.055, // 5.5%
        active_users: 125000,
        health_factor: 2.5,
    };
    
    Ok(Json(response))
}

/// Supply asset to protocol
async fn supply_asset(
    State(state): State<Arc<ApiState>>,
    Path(protocol): Path<String>,
    Json(request): Json<LendingRequest>,
) -> Result<Json<String>, StatusCode> {
    let chain_id = 1u64; // Default to Ethereum mainnet
    let tx_hash = state.defi_manager.supply_asset(
        chain_id,
        protocol.clone(),
        request.asset,
        request.amount,
        request.user,
    ).await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(tx_hash))
}

/// Withdraw asset from protocol
async fn withdraw_asset(
    State(state): State<Arc<ApiState>>,
    Path(protocol): Path<String>,
    Json(request): Json<LendingRequest>,
) -> Result<Json<String>, StatusCode> {
    let chain_id = 1u64; // Default to Ethereum mainnet
    let tx_hash = state.defi_manager.withdraw_asset(
        chain_id,
        protocol.clone(),
        request.asset,
        request.amount,
        request.user,
    ).await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(tx_hash))
}

/// Borrow asset from protocol
async fn borrow_asset(
    State(state): State<Arc<ApiState>>,
    Path(protocol): Path<String>,
    Json(request): Json<LendingRequest>,
) -> Result<Json<String>, StatusCode> {
    let chain_id = 1u64; // Default to Ethereum mainnet
    let tx_hash = state.defi_manager.borrow_asset(
        chain_id,
        protocol.clone(),
        request.asset,
        request.amount,
        request.user,
    ).await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(tx_hash))
}

/// Repay asset to protocol
async fn repay_asset(
    State(state): State<Arc<ApiState>>,
    Path(protocol): Path<String>,
    Json(request): Json<LendingRequest>,
) -> Result<Json<String>, StatusCode> {
    let chain_id = 1u64; // Default to Ethereum mainnet
    let tx_hash = state.defi_manager.repay_asset(
        chain_id,
        protocol.clone(),
        request.asset,
        request.amount,
        request.user,
    ).await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(tx_hash))
}

/// Get yield opportunities across protocols
async fn get_yield_opportunities(
    State(_state): State<Arc<ApiState>>,
) -> Result<Json<Vec<YieldOpportunity>>, StatusCode> {
    // Mock implementation - would fetch from DeFi manager
    let opportunities = vec![
        YieldOpportunity {
            protocol: "Aave".to_string(),
            asset: "0xA0b86a33E6F2C8F9B69C6CfF9b7D83c63c5b90e2".parse().unwrap(), // USDC
            apy: 0.025, // 2.5%
            risk_level: "Low".to_string(),
            minimum_deposit: U256::from(1000u64) * U256::exp10(6), // 1000 USDC
            available_liquidity: U256::from(50000000u64) * U256::exp10(6), // 50M USDC
        },
        YieldOpportunity {
            protocol: "Compound".to_string(),
            asset: "0x6B175474E89094C44Da98b954EedeAC495271d0F".parse().unwrap(), // DAI
            apy: 0.032, // 3.2%
            risk_level: "Low".to_string(),
            minimum_deposit: U256::from(100u64) * U256::exp10(18), // 100 DAI
            available_liquidity: U256::from(25000000u64) * U256::exp10(18), // 25M DAI
        },
    ];
    
    Ok(Json(opportunities))
}

/// Get user's DeFi portfolio
async fn get_user_portfolio(
    State(state): State<Arc<ApiState>>,
    Path(user): Path<Address>,
) -> Result<Json<UserPortfolioResponse>, StatusCode> {
    let chain_id = 1u64; // Default to Ethereum mainnet
    let portfolio = state.defi_manager.get_portfolio_overview(chain_id, user).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let response = UserPortfolioResponse {
        user: portfolio.user,
        total_supplied_usd: portfolio.total_supplied_usd,
        total_borrowed_usd: portfolio.total_borrowed_usd,
        net_worth_usd: portfolio.net_worth_usd,
        overall_health_factor: portfolio.overall_health_factor,
        positions: vec![], // Would map from portfolio positions
    };
    
    Ok(Json(response))
}
