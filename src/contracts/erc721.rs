use ethers::{
    abi::{Abi, Token, Tokenize, Detokenize},
    contract::{Contract, ContractError},
    providers::{Provider, Http},
    types::{Address, U256, H256, TransactionRequest},
};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// NFT Collection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFTCollection {
    pub address: Address,
    pub name: String,
    pub symbol: String,
    pub total_supply: U256,
    pub owner: Address,
    pub base_uri: String,
}

/// NFT Metadata structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFTMetadata {
    pub token_id: U256,
    pub name: String,
    pub description: String,
    pub image: String,
    pub attributes: Vec<NFTAttribute>,
    pub owner: Address,
}

/// NFT Attribute structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFTAttribute {
    pub trait_type: String,
    pub value: String,
    pub display_type: Option<String>,
}

/// Collection marketplace statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionStats {
    pub floor_price: U256,
    pub volume_24h: U256,
    pub sales_24h: u64,
    pub average_price: U256,
}

/// ERC721 contract interface
#[derive(Debug, Clone)]
pub struct ERC721Contract {
    contract: Contract<Provider<Http>>,
    address: Address,
    provider: Arc<Provider<Http>>,
}

impl ERC721Contract {
    /// Create a new ERC721 contract instance
    pub fn new(
        address: Address,
        provider: Arc<Provider<Http>>,
    ) -> Result<Self> {
        let abi = Self::get_erc721_abi()?;
        let contract = Contract::new(address, abi, provider.clone());
        
        Ok(Self {
            contract,
            address,
            provider,
        })
    }

    /// Get ERC721 ABI
    fn get_erc721_abi() -> Result<Abi> {
        let abi_json = r#"[
            {
                "inputs": [{"internalType": "uint256", "name": "tokenId", "type": "uint256"}],
                "name": "ownerOf",
                "outputs": [{"internalType": "address", "name": "", "type": "address"}],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [],
                "name": "name",
                "outputs": [{"internalType": "string", "name": "", "type": "string"}],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [],
                "name": "symbol",
                "outputs": [{"internalType": "string", "name": "", "type": "string"}],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [],
                "name": "totalSupply",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [{"internalType": "uint256", "name": "tokenId", "type": "uint256"}],
                "name": "tokenURI",
                "outputs": [{"internalType": "string", "name": "", "type": "string"}],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [{"internalType": "address", "name": "owner", "type": "address"}],
                "name": "balanceOf",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [
                    {"internalType": "address", "name": "from", "type": "address"},
                    {"internalType": "address", "name": "to", "type": "address"},
                    {"internalType": "uint256", "name": "tokenId", "type": "uint256"}
                ],
                "name": "transferFrom",
                "outputs": [],
                "stateMutability": "nonpayable",
                "type": "function"
            },
            {
                "inputs": [
                    {"internalType": "address", "name": "to", "type": "address"},
                    {"internalType": "uint256", "name": "tokenId", "type": "uint256"}
                ],
                "name": "approve",
                "outputs": [],
                "stateMutability": "nonpayable",
                "type": "function"
            }
        ]"#;
        
        let abi: Abi = serde_json::from_str(abi_json)?;
        Ok(abi)
    }

    /// Load collection information
    pub async fn load_collection_info(&self) -> Result<NFTCollection> {
        let name: String = self.contract
            .method::<_, String>("name", ())?
            .call()
            .await?;
            
        let symbol: String = self.contract
            .method::<_, String>("symbol", ())?
            .call()
            .await?;
            
        let total_supply: U256 = self.contract
            .method::<_, U256>("totalSupply", ())?
            .call()
            .await
            .unwrap_or_else(|_| U256::zero());

        Ok(NFTCollection {
            address: self.address,
            name,
            symbol,
            total_supply,
            owner: Address::zero(), // Would need owner() function in contract
            base_uri: String::new(), // Would need baseURI() function in contract
        })
    }

    /// Get contract address
    pub fn address(&self) -> Address {
        self.address
    }
}
