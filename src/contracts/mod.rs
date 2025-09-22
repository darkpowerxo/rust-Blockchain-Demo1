// Smart Contract Interaction Layer
use anyhow::{Result, anyhow};
use ethers::{
    prelude::*,
    abi::{Abi, Contract},
    types::{Address, U256, H256, Bytes, Transaction, Log},
    providers::{Provider, Http},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;
use tracing::{info, warn, error};
use tokio::sync::RwLock;

pub mod erc20;
pub mod erc721;
pub mod defi_contracts;
pub mod proxy;

use crate::chains::ChainManager;
use erc20::ERC20Contract;
use erc721::ERC721Contract;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractInfo {
    pub address: Address,
    pub contract_type: ContractType,
    pub name: String,
    pub chain_id: u64,
    pub abi_hash: String,
    pub is_verified: bool,
    pub deployment_block: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContractType {
    ERC20,
    ERC721,
    ERC1155,
    UniswapV2,
    UniswapV3,
    Aave,
    Compound,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum ContractInstance {
    ERC20(ERC20Contract),
    ERC721(ERC721Contract),
    // Add other contract types as needed
}

pub struct ContractManager {
    chain_manager: Arc<ChainManager>,
    contracts: Arc<RwLock<HashMap<Address, ContractInstance>>>,
    contract_registry: Arc<RwLock<HashMap<Address, ContractInfo>>>,
    abi_cache: Arc<RwLock<HashMap<String, Abi>>>,
}

impl ContractManager {
    pub async fn new(chain_manager: Arc<ChainManager>) -> Result<Self> {
        info!("Initializing ContractManager");

        Ok(Self {
            chain_manager,
            contracts: Arc::new(RwLock::new(HashMap::new())),
            contract_registry: Arc::new(RwLock::new(HashMap::new())),
            abi_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn register_erc20_contract(
        &self,
        address: Address,
        chain_id: u64,
    ) -> Result<()> {
        info!("Registering ERC-20 contract {:?} on chain {}", address, chain_id);

        let chain_provider = self.chain_manager.get_provider(chain_id).await?;
        let provider = Arc::new(chain_provider.provider.clone());
        let mut contract = ERC20Contract::new(address, provider, chain_id).await?;
        contract.load_token_info().await?;

        let name = contract.get_token_info()
            .map(|info| info.name.clone())
            .unwrap_or_else(|| "Unknown Token".to_string());

        let contract_info = ContractInfo {
            address,
            contract_type: ContractType::ERC20,
            name,
            chain_id,
            abi_hash: "erc20_standard".to_string(),
            is_verified: true,
            deployment_block: 0,
        };

        let mut contracts = self.contracts.write().await;
        let mut registry = self.contract_registry.write().await;
        
        contracts.insert(address, ContractInstance::ERC20(contract));
        registry.insert(address, contract_info);

        info!("ERC-20 contract registered successfully");
        Ok(())
    }

    pub async fn register_erc721_contract(
        &self,
        address: Address,
        chain_id: u64,
    ) -> Result<()> {
        info!("Registering ERC-721 contract {:?} on chain {}", address, chain_id);

        let chain_provider = self.chain_manager.get_provider(chain_id).await?;
        let provider = Arc::new(chain_provider.provider.clone());
        let contract = ERC721Contract::new(address, provider)?;

        let collection_info = contract.load_collection_info().await;
        let name = collection_info
            .map(|info| info.name)
            .unwrap_or_else(|_| "Unknown Collection".to_string());

        let contract_info = ContractInfo {
            address,
            contract_type: ContractType::ERC721,
            name,
            chain_id,
            abi_hash: "erc721_standard".to_string(),
            is_verified: true,
            deployment_block: 0,
        };

        let mut contracts = self.contracts.write().await;
        let mut registry = self.contract_registry.write().await;
        
        contracts.insert(address, ContractInstance::ERC721(contract));
        registry.insert(address, contract_info);

        info!("ERC-721 contract registered successfully");
        Ok(())
    }

    pub async fn get_contract_info(&self, address: Address) -> Result<ContractInfo> {
        let registry = self.contract_registry.read().await;
        registry.get(&address)
            .cloned()
            .ok_or_else(|| anyhow!("Contract not registered: {:?}", address))
    }

    pub async fn get_registered_contracts(&self) -> HashMap<Address, ContractInfo> {
        self.contract_registry.read().await.clone()
    }
}
