use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use ethers::{
    providers::Middleware,
    types::{Address, Block, Transaction, H256, U256},
};

use crate::api::ApiState;

/// Chain switch request
#[derive(Deserialize)]
pub struct ChainSwitchRequest {
    pub chain_id: u64,
}

/// Block query parameters
#[derive(Deserialize)]
pub struct BlockQuery {
    pub block_number: Option<u64>,
    pub block_hash: Option<H256>,
}

/// Chain info response
#[derive(Serialize)]
pub struct ChainInfoResponse {
    pub chain_id: u64,
    pub name: String,
    pub rpc_url: String,
    pub block_explorer: String,
    pub native_currency: CurrencyInfo,
    pub current_block: u64,
    pub gas_price: U256,
    pub is_connected: bool,
}

/// Currency information
#[derive(Serialize)]
pub struct CurrencyInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

/// Gas price response
#[derive(Serialize)]
pub struct GasPriceResponse {
    pub chain_id: u64,
    pub gas_price: U256,
    pub fast_gas_price: U256,
    pub slow_gas_price: U256,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Network stats response
#[derive(Serialize)]
pub struct NetworkStatsResponse {
    pub chain_id: u64,
    pub block_number: u64,
    pub block_time: f64, // Average block time in seconds
    pub transaction_count: u64,
    pub pending_transactions: u64,
    pub network_hashrate: Option<String>,
    pub difficulty: Option<U256>,
}

pub fn routes() -> Router<Arc<ApiState>> {
    Router::new()
        .route("/", get(list_supported_chains))
        .route("/switch", post(switch_chain))
        .route("/{chain_id}", get(get_chain_info))
        .route("/{chain_id}/gas", get(get_gas_price))
        .route("/{chain_id}/stats", get(get_network_stats))
        .route("/{chain_id}/block", get(get_block))
        .route("/{chain_id}/transaction/{tx_hash}", get(get_transaction))
        .route("/{chain_id}/balance/{address}", get(get_balance))
}

/// List all supported chains
async fn list_supported_chains(
    State(_state): State<Arc<ApiState>>,
) -> Result<Json<Vec<ChainInfoResponse>>, StatusCode> {
    // Return hardcoded supported chains info
    let chains = vec![
        ChainInfoResponse {
            chain_id: 1,
            name: "Ethereum Mainnet".to_string(),
            rpc_url: "https://eth-mainnet.alchemyapi.io/v2/demo".to_string(),
            block_explorer: "https://etherscan.io".to_string(),
            native_currency: CurrencyInfo {
                name: "Ethereum".to_string(),
                symbol: "ETH".to_string(),
                decimals: 18,
            },
            current_block: 18500000, // Would be fetched dynamically
            gas_price: U256::from(20_000_000_000u64), // 20 Gwei
            is_connected: true,
        },
        ChainInfoResponse {
            chain_id: 137,
            name: "Polygon Mainnet".to_string(),
            rpc_url: "https://polygon-rpc.com".to_string(),
            block_explorer: "https://polygonscan.com".to_string(),
            native_currency: CurrencyInfo {
                name: "Polygon".to_string(),
                symbol: "MATIC".to_string(),
                decimals: 18,
            },
            current_block: 50000000, // Would be fetched dynamically
            gas_price: U256::from(30_000_000_000u64), // 30 Gwei
            is_connected: true,
        },
        ChainInfoResponse {
            chain_id: 42161,
            name: "Arbitrum One".to_string(),
            rpc_url: "https://arb1.arbitrum.io/rpc".to_string(),
            block_explorer: "https://arbiscan.io".to_string(),
            native_currency: CurrencyInfo {
                name: "Ethereum".to_string(),
                symbol: "ETH".to_string(),
                decimals: 18,
            },
            current_block: 140000000, // Would be fetched dynamically
            gas_price: U256::from(100_000_000u64), // 0.1 Gwei
            is_connected: true,
        },
    ];
    
    Ok(Json(chains))
}

/// Switch to different chain
async fn switch_chain(
    State(state): State<Arc<ApiState>>,
    Json(request): Json<ChainSwitchRequest>,
) -> Result<Json<String>, StatusCode> {
    // In real implementation, would switch the chain
    // For now, just verify the chain exists
    let _provider_info = state.chain_manager.get_provider(request.chain_id).await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    Ok(Json(format!("Switched to chain {}", request.chain_id)))
}

/// Get chain information
async fn get_chain_info(
    State(state): State<Arc<ApiState>>,
    Path(chain_id): Path<u64>,
) -> Result<Json<ChainInfoResponse>, StatusCode> {
    // Get provider for the chain
    let provider_info = state.chain_manager.get_provider(chain_id).await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    // Get current block number
    let block_number = provider_info.provider
        .get_block_number()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Get gas price
    let gas_price = provider_info.provider
        .get_gas_price()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let chain_info = match chain_id {
        1 => ChainInfoResponse {
            chain_id: 1,
            name: "Ethereum Mainnet".to_string(),
            rpc_url: "https://eth-mainnet.alchemyapi.io/v2/demo".to_string(),
            block_explorer: "https://etherscan.io".to_string(),
            native_currency: CurrencyInfo {
                name: "Ethereum".to_string(),
                symbol: "ETH".to_string(),
                decimals: 18,
            },
            current_block: block_number.as_u64(),
            gas_price,
            is_connected: true,
        },
        137 => ChainInfoResponse {
            chain_id: 137,
            name: "Polygon Mainnet".to_string(),
            rpc_url: "https://polygon-rpc.com".to_string(),
            block_explorer: "https://polygonscan.com".to_string(),
            native_currency: CurrencyInfo {
                name: "Polygon".to_string(),
                symbol: "MATIC".to_string(),
                decimals: 18,
            },
            current_block: block_number.as_u64(),
            gas_price,
            is_connected: true,
        },
        42161 => ChainInfoResponse {
            chain_id: 42161,
            name: "Arbitrum One".to_string(),
            rpc_url: "https://arb1.arbitrum.io/rpc".to_string(),
            block_explorer: "https://arbiscan.io".to_string(),
            native_currency: CurrencyInfo {
                name: "Ethereum".to_string(),
                symbol: "ETH".to_string(),
                decimals: 18,
            },
            current_block: block_number.as_u64(),
            gas_price,
            is_connected: true,
        },
        _ => return Err(StatusCode::NOT_FOUND),
    };
    
    Ok(Json(chain_info))
}

/// Get gas price information
async fn get_gas_price(
    State(state): State<Arc<ApiState>>,
    Path(chain_id): Path<u64>,
) -> Result<Json<GasPriceResponse>, StatusCode> {
    let provider_info = state.chain_manager.get_provider(chain_id).await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    let gas_price = provider_info.provider
        .get_gas_price()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Simulate fast and slow gas prices (would use gas station APIs in real implementation)
    let fast_gas_price = gas_price * 120 / 100; // 20% higher
    let slow_gas_price = gas_price * 80 / 100;  // 20% lower
    
    Ok(Json(GasPriceResponse {
        chain_id,
        gas_price,
        fast_gas_price,
        slow_gas_price,
        last_updated: chrono::Utc::now(),
    }))
}

/// Get network statistics
async fn get_network_stats(
    State(state): State<Arc<ApiState>>,
    Path(chain_id): Path<u64>,
) -> Result<Json<NetworkStatsResponse>, StatusCode> {
    let provider_info = state.chain_manager.get_provider(chain_id).await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    let block_number = provider_info.provider
        .get_block_number()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Get latest block to count transactions
    let latest_block = provider_info.provider
        .get_block(ethers::types::BlockId::Number(ethers::types::BlockNumber::Latest))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    Ok(Json(NetworkStatsResponse {
        chain_id,
        block_number: block_number.as_u64(),
        block_time: 12.0, // Would calculate from recent blocks
        transaction_count: latest_block.transactions.len() as u64,
        pending_transactions: 0, // Would get from mempool
        network_hashrate: None, // Would get from network stats
        difficulty: Some(latest_block.difficulty),
    }))
}

/// Get block information
async fn get_block(
    State(state): State<Arc<ApiState>>,
    Path(chain_id): Path<u64>,
    Query(query): Query<BlockQuery>,
) -> Result<Json<Block<H256>>, StatusCode> {
    let provider_info = state.chain_manager.get_provider(chain_id).await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    let block_id = if let Some(block_number) = query.block_number {
        ethers::types::BlockId::Number(ethers::types::BlockNumber::Number(block_number.into()))
    } else if let Some(block_hash) = query.block_hash {
        ethers::types::BlockId::Hash(block_hash)
    } else {
        ethers::types::BlockId::Number(ethers::types::BlockNumber::Latest)
    };
    
    let block = provider_info.provider
        .get_block(block_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    Ok(Json(block))
}

/// Get transaction information
async fn get_transaction(
    State(state): State<Arc<ApiState>>,
    Path((chain_id, tx_hash)): Path<(u64, H256)>,
) -> Result<Json<Transaction>, StatusCode> {
    let provider_info = state.chain_manager.get_provider(chain_id).await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    let transaction = provider_info.provider
        .get_transaction(tx_hash)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    Ok(Json(transaction))
}

/// Get address balance
async fn get_balance(
    State(state): State<Arc<ApiState>>,
    Path((chain_id, address)): Path<(u64, Address)>,
) -> Result<Json<U256>, StatusCode> {
    let provider_info = state.chain_manager.get_provider(chain_id).await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    let balance = provider_info.provider
        .get_balance(address, None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(balance))
}
