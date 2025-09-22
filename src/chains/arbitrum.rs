// Arbitrum chain implementations
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
pub struct ArbitrumChain {
    provider: Arc<Provider<Http>>,
    chain_id: u64,
    rpc_url: String,
    is_testnet: bool,
}

impl ArbitrumChain {
    pub async fn new(rpc_url: String, is_testnet: bool) -> Result<Self> {
        info!("Initializing Arbitrum chain connection to: {}", rpc_url);
        
        let provider = Provider::<Http>::try_from(&rpc_url)?;
        let provider = Arc::new(provider);
        
        // Verify connection and get chain ID
        let chain_id = timeout(
            Duration::from_secs(10), 
            provider.get_chainid()
        ).await??;
        
        info!("Connected to Arbitrum chain ID: {}", chain_id);
        
        // Validate it's actually Arbitrum network
        let expected_chain_id = if is_testnet { 421614 } else { 42161 }; // Arbitrum Sepolia or Arbitrum One
        if chain_id.as_u64() != expected_chain_id {
            warn!("Expected Arbitrum chain ID {} but got {}", expected_chain_id, chain_id);
        }
        
        Ok(Self {
            provider,
            chain_id: chain_id.as_u64(),
            rpc_url,
            is_testnet,
        })
    }

    pub async fn get_balance(&self, address: Address) -> Result<U256> {
        Ok(self.provider.get_balance(address, None).await?)
    }

    pub async fn get_eth_balance(&self, address: Address) -> Result<U256> {
        // ETH is the native token on Arbitrum (bridged from Ethereum)
        self.get_balance(address).await
    }

    pub async fn health_check(&self) -> Result<bool> {
        match timeout(Duration::from_secs(5), self.provider.get_block_number()).await {
            Ok(Ok(_)) => {
                info!("Arbitrum health check passed");
                Ok(true)
            }
            Ok(Err(e)) => {
                warn!("Arbitrum health check failed: {}", e);
                Ok(false)
            }
            Err(_) => {
                warn!("Arbitrum health check timed out");
                Ok(false)
            }
        }
    }
}
