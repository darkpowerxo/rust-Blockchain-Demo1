use anyhow::Result;
use ethers::{prelude::*, types::transaction::eip2718::TypedTransaction};
use tracing::warn;

pub struct TransactionValidator {
    max_gas_price: U256,
    min_gas_limit: u64,
    max_gas_limit: u64,
}

impl TransactionValidator {
    pub fn new() -> Self {
        Self {
            max_gas_price: U256::from(500_000_000_000u64), // 500 gwei max
            min_gas_limit: 21_000, // Minimum for ETH transfer
            max_gas_limit: 10_000_000, // Maximum reasonable gas limit
        }
    }

    pub async fn validate_transaction(&self, tx: &TypedTransaction) -> Result<()> {
        // Validate gas price
        if let Some(gas_price) = tx.gas_price() {
            if gas_price > self.max_gas_price {
                warn!("Gas price {} exceeds maximum {}", gas_price, self.max_gas_price);
                return Err(anyhow::anyhow!("Gas price too high"));
            }
        }

        // Validate gas limit
        if let Some(gas_limit) = tx.gas() {
            let gas_limit_u64 = gas_limit.as_u64();
            if gas_limit_u64 < self.min_gas_limit {
                return Err(anyhow::anyhow!("Gas limit too low"));
            }
            if gas_limit_u64 > self.max_gas_limit {
                return Err(anyhow::anyhow!("Gas limit too high"));
            }
        }

        // Validate nonce (basic check)
        if let Some(nonce) = tx.nonce() {
            if nonce.is_zero() {
                warn!("Transaction nonce is zero, which may cause issues");
            }
        }

        Ok(())
    }
}
