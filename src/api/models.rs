use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

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

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Portfolio {
    pub id: String,
    pub address: String,
    pub total_value_usd: f64,
    pub assets: Vec<Asset>,
    pub defi_positions: Vec<DefiPosition>,
    pub last_updated: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Asset {
    pub token_address: String,
    pub symbol: String,
    pub name: String,
    pub balance: f64,
    pub price_usd: f64,
    pub value_usd: f64,
    pub chain_id: u64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct DefiPosition {
    pub protocol: String,
    pub position_type: String, // lending, staking, liquidity_pool
    pub token_address: String,
    pub amount: f64,
    pub value_usd: f64,
    pub apy: Option<f64>,
    pub rewards: Vec<Reward>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Reward {
    pub token_address: String,
    pub amount: f64,
    pub value_usd: f64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SwapQuote {
    pub from_token: String,
    pub to_token: String,
    pub from_amount: f64,
    pub to_amount: f64,
    pub price_impact: f64,
    pub gas_estimate: u64,
    pub dex: String,
    pub route: Vec<String>,
    pub slippage_tolerance: f64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SwapRequest {
    pub from_token: String,
    pub to_token: String,
    pub amount: f64,
    pub slippage_tolerance: Option<f64>,
    pub chain_id: u64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct YieldOpportunity {
    pub protocol: String,
    pub pool_address: String,
    pub tokens: Vec<String>,
    pub apy: f64,
    pub tvl: f64,
    pub risk_score: u8,
    pub strategy_type: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ArbitrageOpportunity {
    pub token_pair: TokenPair,
    pub dex_a: DexInfo,
    pub dex_b: DexInfo,
    pub profit_potential: f64,
    pub gas_cost: f64,
    pub net_profit: f64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct TokenPair {
    pub token_a: String,
    pub token_b: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct DexInfo {
    pub name: String,
    pub price: f64,
    pub liquidity: f64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct GasOptimization {
    pub current_gas_price: u64,
    pub recommended_gas_price: u64,
    pub estimated_confirmation_time: u32, // in seconds
    pub potential_savings: f64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct TransactionSimulation {
    pub success: bool,
    pub gas_used: u64,
    pub gas_limit: u64,
    pub error_message: Option<String>,
    pub state_changes: Vec<StateChange>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct StateChange {
    pub address: String,
    pub slot: String,
    pub old_value: String,
    pub new_value: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct LendingPosition {
    pub protocol: String,
    pub token: String,
    pub supplied: f64,
    pub borrowed: f64,
    pub supply_apy: f64,
    pub borrow_apy: f64,
    pub health_factor: f64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct FlashLoanOpportunity {
    pub protocol: String,
    pub token: String,
    pub available_amount: f64,
    pub fee: f64,
    pub arbitrage_profit: f64,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct ErrorResponse {
    pub code: u16,
    pub message: String,
    pub details: Option<String>,
}
