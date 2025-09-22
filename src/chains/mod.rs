use anyhow::Result;
use ethers::{
    providers::{Http, Middleware, Provider},
    types::{Address, U256},
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

pub mod ethereum;
pub mod polygon;
pub mod arbitrum;
pub mod gas_optimizer;

use crate::api::health::ChainHealth;
use ethereum::EthereumChain;
use polygon::PolygonChain;
use arbitrum::ArbitrumChain;
use gas_optimizer::GasOptimizer;

#[derive(Debug, Clone)]
pub struct ChainConfig {
    pub chain_id: u64,
    pub name: String,
    pub rpc_url: String,
    pub ws_url: Option<String>,
    pub block_explorer: String,
    pub native_token: String,
    pub is_testnet: bool,
}

#[derive(Debug)]
pub enum ChainImplementation {
    Ethereum(EthereumChain),
    Polygon(PolygonChain),
    Arbitrum(ArbitrumChain),
}

pub struct ChainManager {
    chains: HashMap<u64, Arc<ChainProvider>>,
    gas_optimizer: GasOptimizer,
}

pub struct ChainProvider {
    pub config: ChainConfig,
    pub provider: Provider<Http>,
    pub chain_impl: Arc<ChainImplementation>,
    pub connection_pool: Arc<RwLock<ConnectionPool>>,
}

#[derive(Debug)]
struct ConnectionPool {
    active_connections: u32,
    max_connections: u32,
    retry_count: HashMap<String, u32>,
}

impl ChainManager {
    pub async fn new(config: &config::Config) -> Result<Self> {
        let mut chains = HashMap::new();

        // Initialize Ethereum mainnet
        let eth_config = ChainConfig {
            chain_id: 1,
            name: "Ethereum".to_string(),
            rpc_url: config
                .get_string("ethereum_rpc_url")
                .unwrap_or_else(|_| "https://mainnet.infura.io/v3/YOUR_PROJECT_ID".to_string()),
            ws_url: Some(config
                .get_string("ethereum_ws_url")
                .unwrap_or_else(|_| "wss://mainnet.infura.io/ws/v3/YOUR_PROJECT_ID".to_string())),
            block_explorer: "https://etherscan.io".to_string(),
            native_token: "ETH".to_string(),
            is_testnet: false,
        };

        let eth_provider = ChainProvider::new(eth_config).await?;
        chains.insert(1, Arc::new(eth_provider));

        // Initialize Polygon
        let polygon_config = ChainConfig {
            chain_id: 137,
            name: "Polygon".to_string(),
            rpc_url: config
                .get_string("polygon_rpc_url")
                .unwrap_or_else(|_| "https://polygon-rpc.com".to_string()),
            ws_url: Some(config
                .get_string("polygon_ws_url")
                .unwrap_or_else(|_| "wss://polygon-rpc.com".to_string())),
            block_explorer: "https://polygonscan.com".to_string(),
            native_token: "MATIC".to_string(),
            is_testnet: false,
        };

        let polygon_provider = ChainProvider::new(polygon_config).await?;
        chains.insert(137, Arc::new(polygon_provider));

        // Initialize Arbitrum
        let arbitrum_config = ChainConfig {
            chain_id: 42161,
            name: "Arbitrum One".to_string(),
            rpc_url: config
                .get_string("arbitrum_rpc_url")
                .unwrap_or_else(|_| "https://arb1.arbitrum.io/rpc".to_string()),
            ws_url: Some(config
                .get_string("arbitrum_ws_url")
                .unwrap_or_else(|_| "wss://arb1.arbitrum.io/ws".to_string())),
            block_explorer: "https://arbiscan.io".to_string(),
            native_token: "ETH".to_string(),
            is_testnet: false,
        };

        let arbitrum_provider = ChainProvider::new(arbitrum_config).await?;
        chains.insert(42161, Arc::new(arbitrum_provider));

        let gas_optimizer = gas_optimizer::GasOptimizer::new();

        info!("Initialized ChainManager with {} chains", chains.len());

        Ok(Self {
            chains,
            gas_optimizer,
        })
    }

    pub async fn new_demo() -> Result<Self> {
        info!("Creating ChainManager in demo mode");
        let chains = HashMap::new(); // Empty chains for demo
        let gas_optimizer = gas_optimizer::GasOptimizer::new();

        Ok(Self {
            chains,
            gas_optimizer,
        })
    }

    pub async fn get_provider(&self, chain_id: u64) -> Result<Arc<ChainProvider>> {
        self.chains
            .get(&chain_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Chain {} not supported", chain_id))
    }

    pub async fn get_block_number(&self, chain_id: u64) -> Result<u64> {
        let provider = self.get_provider(chain_id).await?;
        let block_number = provider.provider.get_block_number().await?;
        Ok(block_number.as_u64())
    }

    pub async fn get_gas_price(&self, chain_id: u64) -> Result<U256> {
        let provider = self.get_provider(chain_id).await?;
        let gas_price = provider.provider.get_gas_price().await?;
        Ok(gas_price)
    }

    pub async fn get_balance(&self, chain_id: u64, address: Address) -> Result<U256> {
        let provider = self.get_provider(chain_id).await?;
        let balance = provider.provider.get_balance(address, None).await?;
        Ok(balance)
    }

    pub async fn estimate_gas_optimized(&self, chain_id: u64, tx_data: &[u8]) -> Result<(U256, U256)> {
        self.gas_optimizer.estimate_optimal_gas(chain_id, tx_data).await
    }

    pub async fn health_check(&self) -> Vec<ChainHealth> {
        let mut health_results = Vec::new();

        for (chain_id, provider) in &self.chains {
            let health = self.check_chain_health(*chain_id, provider).await;
            health_results.push(health);
        }

        health_results
    }

    async fn check_chain_health(&self, chain_id: u64, provider: &Arc<ChainProvider>) -> ChainHealth {
        let mut health = ChainHealth {
            chain_id,
            name: provider.config.name.clone(),
            rpc_healthy: false,
            block_height: None,
            gas_price: None,
        };

        // Test RPC connectivity and get block height
        match provider.provider.get_block_number().await {
            Ok(block_number) => {
                health.rpc_healthy = true;
                health.block_height = Some(block_number.as_u64());
            }
            Err(e) => {
                warn!("Chain {} RPC unhealthy: {}", chain_id, e);
            }
        }

        // Get current gas price
        match provider.provider.get_gas_price().await {
            Ok(gas_price) => {
                health.gas_price = Some(gas_price.to_string());
            }
            Err(e) => {
                warn!("Failed to get gas price for chain {}: {}", chain_id, e);
            }
        }

        health
    }

    pub fn get_supported_chains(&self) -> Vec<&ChainConfig> {
        self.chains.values().map(|provider| &provider.config).collect()
    }
}

impl ChainProvider {
    pub async fn new(config: ChainConfig) -> Result<Self> {
        let provider = Provider::<Http>::try_from(&config.rpc_url)?;
        
        // Test the connection
        match provider.get_chainid().await {
            Ok(chain_id) => {
                if chain_id.as_u64() != config.chain_id {
                    warn!(
                        "Chain ID mismatch for {}: expected {}, got {}",
                        config.name,
                        config.chain_id,
                        chain_id.as_u64()
                    );
                }
                info!("Successfully connected to {} (chain_id: {})", config.name, chain_id);
            }
            Err(e) => {
                error!("Failed to connect to {}: {}", config.name, e);
                return Err(e.into());
            }
        }

        // Create chain-specific implementation
        let chain_impl = match config.chain_id {
            1 | 11155111 => { // Ethereum mainnet or Sepolia
                let eth_chain = EthereumChain::new(config.rpc_url.clone(), config.is_testnet).await?;
                Arc::new(ChainImplementation::Ethereum(eth_chain))
            },
            137 | 80001 => { // Polygon mainnet or Mumbai
                let polygon_chain = PolygonChain::new(config.rpc_url.clone(), config.is_testnet).await?;
                Arc::new(ChainImplementation::Polygon(polygon_chain))
            },
            42161 | 421614 => { // Arbitrum One or Sepolia
                let arbitrum_chain = ArbitrumChain::new(config.rpc_url.clone(), config.is_testnet).await?;
                Arc::new(ChainImplementation::Arbitrum(arbitrum_chain))
            },
            _ => {
                // Fallback to generic Ethereum implementation for unknown chains
                warn!("Unknown chain ID {}, using generic Ethereum implementation", config.chain_id);
                let eth_chain = EthereumChain::new(config.rpc_url.clone(), config.is_testnet).await?;
                Arc::new(ChainImplementation::Ethereum(eth_chain))
            }
        };

        let connection_pool = Arc::new(RwLock::new(ConnectionPool {
            active_connections: 0,
            max_connections: 10,
            retry_count: HashMap::new(),
        }));

        Ok(Self {
            config,
            provider,
            chain_impl,
            connection_pool,
        })
    }

    pub async fn with_retry<T, F, Fut>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut attempts = 0;
        let max_attempts = 3;
        let mut last_error = None;

        while attempts < max_attempts {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempts += 1;
                    last_error = Some(e);
                    
                    if attempts < max_attempts {
                        let delay = std::time::Duration::from_millis(1000 * attempts);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    // Chain-specific method access
    pub async fn get_chain_specific_balance(&self, address: Address) -> Result<U256> {
        match self.chain_impl.as_ref() {
            ChainImplementation::Ethereum(eth) => eth.get_balance(address).await,
            ChainImplementation::Polygon(poly) => poly.get_matic_balance(address).await,
            ChainImplementation::Arbitrum(arb) => arb.get_eth_balance(address).await,
        }
    }

    pub async fn chain_health_check(&self) -> Result<bool> {
        match self.chain_impl.as_ref() {
            ChainImplementation::Ethereum(eth) => eth.health_check().await,
            ChainImplementation::Polygon(poly) => poly.health_check().await,
            ChainImplementation::Arbitrum(arb) => arb.health_check().await,
        }
    }

    pub fn get_chain_name(&self) -> &str {
        match self.chain_impl.as_ref() {
            ChainImplementation::Ethereum(_) => {
                if self.config.is_testnet { "Ethereum Sepolia" } else { "Ethereum Mainnet" }
            },
            ChainImplementation::Polygon(_) => {
                if self.config.is_testnet { "Polygon Mumbai" } else { "Polygon Mainnet" }
            },
            ChainImplementation::Arbitrum(_) => {
                if self.config.is_testnet { "Arbitrum Sepolia" } else { "Arbitrum One" }
            },
        }
    }
}
