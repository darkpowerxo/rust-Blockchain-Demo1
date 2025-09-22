// Polygon (Matic) chain implementations
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
pub struct PolygonChain {
    provider: Arc<Provider<Http>>,
    chain_id: u64,
    rpc_url: String,
    is_testnet: bool,
}

impl PolygonChain {
    pub async fn new(rpc_url: String, is_testnet: bool) -> Result<Self> {
        info!("Initializing Polygon chain connection to: {}", rpc_url);
        
        let provider = Provider::<Http>::try_from(&rpc_url)?;
        let provider = Arc::new(provider);
        
        // Verify connection and get chain ID
        let chain_id = timeout(
            Duration::from_secs(10), 
            provider.get_chainid()
        ).await??;
        
        info!("Connected to Polygon chain ID: {}", chain_id);
        
        // Validate it's actually Polygon network
        let expected_chain_id = if is_testnet { 80001 } else { 137 }; // Mumbai testnet or Polygon mainnet
        if chain_id.as_u64() != expected_chain_id {
            warn!("Expected Polygon chain ID {} but got {}", expected_chain_id, chain_id);
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

    pub async fn get_matic_balance(&self, address: Address) -> Result<U256> {
        // MATIC is the native token on Polygon
        self.get_balance(address).await
    }

    pub async fn health_check(&self) -> Result<bool> {
        match timeout(Duration::from_secs(5), self.provider.get_block_number()).await {
            Ok(Ok(_)) => {
                info!("Polygon health check passed");
                Ok(true)
            }
            Ok(Err(e)) => {
                warn!("Polygon health check failed: {}", e);
                Ok(false)
            }
            Err(_) => {
                warn!("Polygon health check timed out");
                Ok(false)
            }
        }
    }
}
