use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
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
    pub id: Uuid,
    pub address: String,
    pub total_value_usd: Decimal,
    pub assets: Vec<Asset>,
    pub defi_positions: Vec<DefiPosition>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Asset {
    pub token_address: String,
    pub symbol: String,
    pub name: String,
    pub balance: Decimal,
    pub price_usd: Decimal,
    pub value_usd: Decimal,
    pub chain_id: u64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct DefiPosition {
    pub protocol: String,
    pub position_type: String, // lending, staking, liquidity_pool
    pub token_address: String,
    pub amount: Decimal,
    pub value_usd: Decimal,
    pub apy: Option<Decimal>,
    pub rewards: Vec<Reward>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Reward {
    pub token_address: String,
    pub amount: Decimal,
    pub value_usd: Decimal,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SwapQuote {
    pub from_token: String,
    pub to_token: String,
    pub from_amount: Decimal,
    pub to_amount: Decimal,
    pub price_impact: Decimal,
    pub gas_estimate: u64,
    pub dex: String,
    pub route: Vec<String>,
    pub slippage_tolerance: Decimal,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SwapRequest {
    pub from_token: String,
    pub to_token: String,
    pub amount: Decimal,
    pub slippage_tolerance: Option<Decimal>,
    pub chain_id: u64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct YieldOpportunity {
    pub protocol: String,
    pub pool_address: String,
    pub tokens: Vec<String>,
    pub apy: Decimal,
    pub tvl: Decimal,
    pub risk_score: u8,
    pub strategy_type: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ArbitrageOpportunity {
    pub token_pair: TokenPair,
    pub dex_a: DexInfo,
    pub dex_b: DexInfo,
    pub profit_potential: Decimal,
    pub gas_cost: Decimal,
    pub net_profit: Decimal,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct TokenPair {
    pub token_a: String,
    pub token_b: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct DexInfo {
    pub name: String,
    pub price: Decimal,
    pub liquidity: Decimal,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct GasOptimization {
    pub current_gas_price: u64,
    pub recommended_gas_price: u64,
    pub estimated_confirmation_time: u32, // in seconds
    pub potential_savings: Decimal,
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
    pub supplied: Decimal,
    pub borrowed: Decimal,
    pub supply_apy: Decimal,
    pub borrow_apy: Decimal,
    pub health_factor: Decimal,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct FlashLoanOpportunity {
    pub protocol: String,
    pub token: String,
    pub available_amount: Decimal,
    pub fee: Decimal,
    pub arbitrage_profit: Decimal,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct ErrorResponse {
    pub code: u16,
    pub message: String,
    pub details: Option<String>,
}
