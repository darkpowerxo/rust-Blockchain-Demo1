use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use ethers::{
    types::{Address, Signature, transaction::eip2718::TypedTransaction},
    utils::hex,
};

use crate::api::ApiState;

/// Wallet connection request
#[derive(Deserialize)]
pub struct WalletConnectionRequest {
    pub wallet_type: String, // "metamask", "walletconnect", "ledger", "local"
    pub chain_id: u64,
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

/// Local wallet creation request
#[derive(Deserialize)]
pub struct LocalWalletRequest {
    pub private_key: Option<String>, // If None, generates random
}

/// Multi-sig wallet creation request
#[derive(Deserialize)]
pub struct MultiSigWalletRequest {
    pub owners: Vec<Address>,
    pub threshold: u8,
    pub chain_id: u64,
}

/// Message signing request
#[derive(Deserialize)]
pub struct SignMessageRequest {
    pub message: String, // Hex-encoded message
}

/// Transaction signing request
#[derive(Deserialize)]
pub struct SignTransactionRequest {
    pub transaction: TypedTransaction,
}

/// Wallet info response
#[derive(Serialize)]
pub struct WalletInfoResponse {
    pub address: Address,
    pub wallet_type: String,
    pub chain_id: u64,
    pub is_connected: bool,
    pub balance: Option<String>, // ETH balance
}

/// Wallet connection response
#[derive(Serialize)]
pub struct WalletConnectionResponse {
    pub address: Address,
    pub wallet_type: String,
    pub chain_id: u64,
    pub message: String,
}

pub fn routes() -> Router<Arc<ApiState>> {
    Router::new()
        .route("/connect/metamask", post(connect_metamask))
        .route("/connect/walletconnect", post(connect_walletconnect))
        .route("/connect/ledger", post(connect_ledger))
        .route("/create/local", post(create_local_wallet))
        .route("/create/multisig", post(create_multisig_wallet))
        .route("/list", get(list_wallets))
        .route("/:address", get(get_wallet_info))
        .route("/:address", delete(disconnect_wallet))
        .route("/:address/sign/message", post(sign_message))
        .route("/:address/sign/transaction", post(sign_transaction))
}

/// Connect MetaMask wallet
async fn connect_metamask(
    State(state): State<Arc<ApiState>>,
    Json(request): Json<WalletConnectionRequest>,
) -> Result<Json<WalletConnectionResponse>, StatusCode> {
    let address = state.wallet_manager.connect_metamask(request.chain_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(WalletConnectionResponse {
        address,
        wallet_type: "metamask".to_string(),
        chain_id: request.chain_id,
        message: "MetaMask wallet connected successfully".to_string(),
    }))
}

/// Connect WalletConnect
async fn connect_walletconnect(
    State(state): State<Arc<ApiState>>,
    Json(request): Json<WalletConnectionRequest>,
) -> Result<Json<WalletConnectionResponse>, StatusCode> {
    let project_id = request.metadata
        .as_ref()
        .and_then(|m| m.get("project_id"))
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let address = state.wallet_manager.connect_walletconnect(project_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(WalletConnectionResponse {
        address,
        wallet_type: "walletconnect".to_string(),
        chain_id: request.chain_id,
        message: "WalletConnect connected successfully".to_string(),
    }))
}

/// Connect Ledger wallet
async fn connect_ledger(
    State(state): State<Arc<ApiState>>,
    Json(request): Json<WalletConnectionRequest>,
) -> Result<Json<WalletConnectionResponse>, StatusCode> {
    let derivation_path = request.metadata
        .as_ref()
        .and_then(|m| m.get("derivation_path"))
        .map(|s| s.as_str())
        .unwrap_or("m/44'/60'/0'/0");
    
    let address = state.wallet_manager.connect_ledger(derivation_path).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(WalletConnectionResponse {
        address,
        wallet_type: "ledger".to_string(),
        chain_id: request.chain_id,
        message: "Ledger wallet connected successfully".to_string(),
    }))
}

/// Create local wallet
async fn create_local_wallet(
    State(state): State<Arc<ApiState>>,
    Json(request): Json<LocalWalletRequest>,
) -> Result<Json<WalletConnectionResponse>, StatusCode> {
    let address = state.wallet_manager.create_local_wallet(request.private_key).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(WalletConnectionResponse {
        address,
        wallet_type: "local".to_string(),
        chain_id: 1, // Default to mainnet
        message: "Local wallet created successfully".to_string(),
    }))
}

/// Create multi-sig wallet
async fn create_multisig_wallet(
    State(state): State<Arc<ApiState>>,
    Json(request): Json<MultiSigWalletRequest>,
) -> Result<Json<WalletConnectionResponse>, StatusCode> {
    let address = state.wallet_manager.create_multisig_wallet(
        request.owners,
        request.threshold,
        request.chain_id,
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(WalletConnectionResponse {
        address,
        wallet_type: "multisig".to_string(),
        chain_id: request.chain_id,
        message: "Multi-sig wallet created successfully".to_string(),
    }))
}

/// List all connected wallets
async fn list_wallets(
    State(state): State<Arc<ApiState>>,
) -> Result<Json<Vec<WalletInfoResponse>>, StatusCode> {
    let wallets = state.wallet_manager.list_wallets().await;
    
    let wallet_responses = wallets.into_iter().map(|info| {
        WalletInfoResponse {
            address: info.address,
            wallet_type: format!("{:?}", info.wallet_type), // Convert enum to string
            chain_id: info.chain_id,
            is_connected: info.is_connected,
            balance: None, // Would fetch balance in real implementation
        }
    }).collect();
    
    Ok(Json(wallet_responses))
}

/// Get wallet information
async fn get_wallet_info(
    State(state): State<Arc<ApiState>>,
    Path(address): Path<Address>,
) -> Result<Json<WalletInfoResponse>, StatusCode> {
    let info = state.wallet_manager.get_wallet_info(address).await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    Ok(Json(WalletInfoResponse {
        address: info.address,
        wallet_type: format!("{:?}", info.wallet_type),
        chain_id: info.chain_id,
        is_connected: info.is_connected,
        balance: None, // Would fetch balance in real implementation
    }))
}

/// Disconnect wallet
async fn disconnect_wallet(
    State(state): State<Arc<ApiState>>,
    Path(address): Path<Address>,
) -> Result<Json<String>, StatusCode> {
    state.wallet_manager.disconnect_wallet(address).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json("Wallet disconnected successfully".to_string()))
}

/// Sign message with wallet
async fn sign_message(
    State(state): State<Arc<ApiState>>,
    Path(address): Path<Address>,
    Json(request): Json<SignMessageRequest>,
) -> Result<Json<Signature>, StatusCode> {
    // Decode hex message
    let message = hex::decode(&request.message.trim_start_matches("0x"))
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let signature = state.wallet_manager.sign_message(address, &message).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(signature))
}

/// Sign transaction with wallet
async fn sign_transaction(
    State(state): State<Arc<ApiState>>,
    Path(address): Path<Address>,
    Json(request): Json<SignTransactionRequest>,
) -> Result<Json<Signature>, StatusCode> {
    let signature = state.wallet_manager.sign_transaction(address, request.transaction).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(signature))
}
