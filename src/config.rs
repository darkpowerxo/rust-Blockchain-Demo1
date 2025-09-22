use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub chains: HashMap<u64, ChainConfig>,
    pub wallets: WalletConfig,
    pub security: SecurityConfig,
    pub api: ApiConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub name: String,
    pub rpc_url: String,
    pub ws_url: Option<String>,
    pub block_explorer: String,
    pub native_token: String,
    pub is_testnet: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    pub metamask_enabled: bool,
    pub walletconnect_project_id: Option<String>,
    pub ledger_enabled: bool,
    pub local_wallet_enabled: bool,
    pub multisig_enabled: bool,
    pub default_derivation_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub max_transaction_value: String, // Wei as string
    pub max_gas_limit: u64,
    pub enable_reentrancy_protection: bool,
    pub blacklisted_addresses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub cors_enabled: bool,
    pub rate_limiting_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub enable_logging: bool,
}

impl Config {
    pub fn default() -> Self {
        let mut chains = HashMap::new();
        
        // Ethereum Mainnet
        chains.insert(1, ChainConfig {
            name: "Ethereum Mainnet".to_string(),
            rpc_url: "https://eth.llamarpc.com".to_string(),
            ws_url: Some("wss://eth.llamarpc.com".to_string()),
            block_explorer: "https://etherscan.io".to_string(),
            native_token: "ETH".to_string(),
            is_testnet: false,
        });
        
        // Polygon Mainnet  
        chains.insert(137, ChainConfig {
            name: "Polygon Mainnet".to_string(),
            rpc_url: "https://polygon.llamarpc.com".to_string(),
            ws_url: Some("wss://polygon.llamarpc.com".to_string()),
            block_explorer: "https://polygonscan.com".to_string(),
            native_token: "MATIC".to_string(),
            is_testnet: false,
        });
        
        // Arbitrum One
        chains.insert(42161, ChainConfig {
            name: "Arbitrum One".to_string(),
            rpc_url: "https://arbitrum.llamarpc.com".to_string(),
            ws_url: Some("wss://arbitrum.llamarpc.com".to_string()),
            block_explorer: "https://arbiscan.io".to_string(),
            native_token: "ETH".to_string(),
            is_testnet: false,
        });

        // Ethereum Sepolia Testnet
        chains.insert(11155111, ChainConfig {
            name: "Ethereum Sepolia".to_string(),
            rpc_url: "https://eth-sepolia.public.blastapi.io".to_string(),
            ws_url: None,
            block_explorer: "https://sepolia.etherscan.io".to_string(),
            native_token: "ETH".to_string(),
            is_testnet: true,
        });

        Self {
            chains,
            wallets: WalletConfig {
                metamask_enabled: true,
                walletconnect_project_id: None,
                ledger_enabled: true,
                local_wallet_enabled: true,
                multisig_enabled: true,
                default_derivation_path: "m/44'/60'/0'/0/0".to_string(),
            },
            security: SecurityConfig {
                max_transaction_value: "1000000000000000000000".to_string(), // 1000 ETH in wei
                max_gas_limit: 10_000_000,
                enable_reentrancy_protection: true,
                blacklisted_addresses: vec![],
            },
            api: ApiConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
                cors_enabled: true,
                rate_limiting_enabled: false,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost/blockchain_demo".to_string(),
                max_connections: 10,
                enable_logging: false,
            },
        }
    }

    pub fn load_from_env() -> Result<Self> {
        // TODO: Implement loading from environment variables
        // For now, return default config
        Ok(Self::default())
    }
}
