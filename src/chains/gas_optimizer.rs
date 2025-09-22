use anyhow::Result;
use ethers::types::U256;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::info;

pub struct GasOptimizer {
    chain_configs: HashMap<u64, ChainGasConfig>,
    recent_prices: RwLock<HashMap<u64, Vec<GasPricePoint>>>,
}

#[derive(Clone)]
struct ChainGasConfig {
    pub base_fee_multiplier: f64,
    pub priority_fee_multiplier: f64,
    pub max_fee_multiplier: f64,
    pub confirmation_target_blocks: u64,
}

#[derive(Clone)]
struct GasPricePoint {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub base_fee: U256,
    pub priority_fee: U256,
    pub gas_used: u64,
}

impl GasOptimizer {
    pub fn new() -> Self {
        let mut chain_configs = HashMap::new();

        // Ethereum mainnet configuration
        chain_configs.insert(1, ChainGasConfig {
            base_fee_multiplier: 1.125, // EIP-1559 recommended
            priority_fee_multiplier: 1.1,
            max_fee_multiplier: 2.0,
            confirmation_target_blocks: 3,
        });

        // Polygon configuration
        chain_configs.insert(137, ChainGasConfig {
            base_fee_multiplier: 1.1,
            priority_fee_multiplier: 1.05,
            max_fee_multiplier: 1.5,
            confirmation_target_blocks: 2,
        });

        // Arbitrum configuration
        chain_configs.insert(42161, ChainGasConfig {
            base_fee_multiplier: 1.05,
            priority_fee_multiplier: 1.02,
            max_fee_multiplier: 1.2,
            confirmation_target_blocks: 1,
        });

        Self {
            chain_configs,
            recent_prices: RwLock::new(HashMap::new()),
        }
    }

    pub async fn estimate_optimal_gas(&self, chain_id: u64, _tx_data: &[u8]) -> Result<(U256, U256)> {
        let config = self.chain_configs
            .get(&chain_id)
            .ok_or_else(|| anyhow::anyhow!("No gas configuration for chain {}", chain_id))?;

        // For now, return basic estimates
        // In production, this would analyze recent blocks, mempool, and transaction complexity
        let base_gas_price = self.get_current_base_fee(chain_id).await?;
        let priority_fee = self.get_optimal_priority_fee(chain_id).await?;

        let max_fee_per_gas = U256::from((base_gas_price.as_u64() as f64 * config.max_fee_multiplier) as u64) + priority_fee;
        let max_priority_fee_per_gas = priority_fee;

        info!(
            "Optimized gas for chain {}: max_fee={}, priority_fee={}",
            chain_id,
            max_fee_per_gas,
            max_priority_fee_per_gas
        );

        Ok((max_fee_per_gas, max_priority_fee_per_gas))
    }

    async fn get_current_base_fee(&self, chain_id: u64) -> Result<U256> {
        // In production, this would fetch from the actual chain
        // For demo purposes, return chain-specific default values
        let base_fee = match chain_id {
            1 => U256::from(20_000_000_000u64), // 20 gwei for Ethereum
            137 => U256::from(30_000_000_000u64), // 30 gwei for Polygon
            42161 => U256::from(100_000_000u64), // 0.1 gwei for Arbitrum
            _ => U256::from(20_000_000_000u64),
        };

        Ok(base_fee)
    }

    async fn get_optimal_priority_fee(&self, chain_id: u64) -> Result<U256> {
        // In production, this would analyze recent blocks and mempool
        let priority_fee = match chain_id {
            1 => U256::from(2_000_000_000u64), // 2 gwei for Ethereum
            137 => U256::from(30_000_000_000u64), // 30 gwei for Polygon (higher due to validator requirements)
            42161 => U256::from(10_000_000u64), // 0.01 gwei for Arbitrum
            _ => U256::from(1_000_000_000u64),
        };

        Ok(priority_fee)
    }

    pub async fn predict_confirmation_time(&self, chain_id: u64, gas_price: U256) -> Result<u64> {
        let config = self.chain_configs
            .get(&chain_id)
            .ok_or_else(|| anyhow::anyhow!("No gas configuration for chain {}", chain_id))?;

        // Simple prediction based on gas price relative to base fee
        let base_fee = self.get_current_base_fee(chain_id).await?;
        let price_ratio = gas_price.as_u64() as f64 / base_fee.as_u64() as f64;

        let estimated_blocks = if price_ratio >= 2.0 {
            1 // Very fast
        } else if price_ratio >= 1.5 {
            config.confirmation_target_blocks
        } else if price_ratio >= 1.1 {
            config.confirmation_target_blocks * 2
        } else {
            config.confirmation_target_blocks * 4
        };

        // Convert blocks to seconds (chain-specific block times)
        let block_time = match chain_id {
            1 => 12, // Ethereum: ~12 seconds
            137 => 2, // Polygon: ~2 seconds
            42161 => 1, // Arbitrum: ~1 second (L2)
            _ => 12,
        };

        Ok(estimated_blocks * block_time)
    }

    pub async fn calculate_gas_savings(&self, chain_id: u64, current_price: U256, optimized_price: U256, gas_limit: u64) -> Result<f64> {
        if current_price <= optimized_price {
            return Ok(0.0);
        }

        let savings_per_gas = current_price - optimized_price;
        let total_savings_wei = savings_per_gas * U256::from(gas_limit);
        
        // Convert to USD (simplified - in production would use real price feeds)
        let eth_price_usd = match chain_id {
            1 | 42161 => 2000.0, // ETH price
            137 => 0.8, // MATIC price
            _ => 2000.0,
        };

        let savings_eth = total_savings_wei.as_u64() as f64 / 1e18;
        let savings_usd = savings_eth * eth_price_usd;

        Ok(savings_usd)
    }
}
