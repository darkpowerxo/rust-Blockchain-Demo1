// Ethereum-specific chain implementations
use anyhow::Result;
use ethers::{
    prelude::*,
    providers::{Http, Provider, Middleware},
    types::{Address, U256},
};
use std::sync::Arc;
use tokio::time::{Duration, timeout};
use tracing::{info, warn};

#[derive(Debug)]
pub struct EthereumChain {
    provider: Arc<Provider<Http>>,
    chain_id: u64,
    rpc_url: String,
    is_testnet: bool,
}

impl EthereumChain {
    pub async fn new(rpc_url: String, is_testnet: bool) -> Result<Self> {
        info!("Initializing Ethereum chain connection to: {}", rpc_url);
        
        // Create provider with timeout
        let provider = Provider::<Http>::try_from(&rpc_url)?;
        let provider = Arc::new(provider);
        
        // Get chain ID to verify connection
        let chain_id = timeout(
            Duration::from_secs(10), 
            provider.get_chainid()
        ).await??;
        
        info!("Connected to Ethereum chain ID: {}", chain_id);
        
        Ok(Self {
            provider,
            chain_id: chain_id.as_u64(),
            rpc_url,
            is_testnet,
        })
    }

    pub async fn get_latest_block_number(&self) -> Result<u64> {
        let block_number = self.provider.get_block_number().await?;
        Ok(block_number.as_u64())
    }

    pub async fn get_balance(&self, address: Address) -> Result<U256> {
        Ok(self.provider.get_balance(address, None).await?)
    }

    pub async fn health_check(&self) -> Result<bool> {
        match timeout(Duration::from_secs(5), self.provider.get_block_number()).await {
            Ok(Ok(_)) => {
                info!("Ethereum health check passed");
                Ok(true)
            }
            Ok(Err(e)) => {
                warn!("Ethereum health check failed: {}", e);
                Ok(false)
            }
            Err(_) => {
                warn!("Ethereum health check timed out");
                Ok(false)
            }
        }
    }
}
