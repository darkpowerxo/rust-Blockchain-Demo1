use anyhow::{Result, anyhow};
use ethers::{
    abi::{Abi, Token},
    contract::Contract,
    providers::{Provider, Http},
    types::{Address, U256, TransactionRequest},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;
use tracing::{info, warn, error};

use crate::chains::ChainManager;

/// SushiSwap pair information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairInfo {
    pub address: Address,
    pub token0: Address,
    pub token1: Address,
    pub reserves: (U256, U256, u32), // reserve0, reserve1, blockTimestampLast
    pub price0_cumulative_last: U256,
    pub price1_cumulative_last: U256,
    pub k_last: U256,
}

/// Farming pool information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FarmInfo {
    pub pid: u64,
    pub lp_token: Address,
    pub alloc_point: U256,
    pub last_reward_block: u64,
    pub acc_sushi_per_share: U256,
    pub reward_per_block: U256,
    pub total_staked: U256,
    pub apy: f64,
}

/// User farming position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPosition {
    pub pid: u64,
    pub amount: U256,
    pub reward_debt: U256,
    pub pending_rewards: U256,
}

/// SushiSwap contract addresses for different chains
#[derive(Debug, Clone)]
pub struct SushiSwapContracts {
    pub factory: Address,
    pub router: Address,
    pub master_chef: Address,
    pub sushi_token: Address,
}

impl SushiSwapContracts {
    pub fn for_chain(chain_id: u64) -> Self {
        match chain_id {
            1 => Self::ethereum_mainnet(),
            137 => Self::polygon(),
            42161 => Self::arbitrum(),
            _ => Self::ethereum_mainnet(),
        }
    }

    fn ethereum_mainnet() -> Self {
        Self {
            factory: "0xC0AEe478e3658e2610c5F7A4A2E1777cE9e4f2Ac".parse().unwrap(),
            router: "0xd9e1cE17f2641f24aE83637ab66a2cca9C378B9F".parse().unwrap(),
            master_chef: "0xc2EdaD668740f1aA35E4D8f227fB8E17dcA888Cd".parse().unwrap(),
            sushi_token: "0x6B3595068778DD592e39A122f4f5a5cF09C90fE2".parse().unwrap(),
        }
    }

    fn polygon() -> Self {
        Self {
            factory: "0xc35DADB65012eC5796536bD9864eD8773aBc74C4".parse().unwrap(),
            router: "0x1b02dA8Cb0d097eB8D57A175b88c7D8b47997506".parse().unwrap(),
            master_chef: "0x0769fd68dFb93167989C6f7254cd0D766Fb2841F".parse().unwrap(),
            sushi_token: "0x0b3F868E0BE5597D5DB7fEB59E1CADBb0fdDa50a".parse().unwrap(),
        }
    }

    fn arbitrum() -> Self {
        Self {
            factory: "0xc35DADB65012eC5796536bD9864eD8773aBc74C4".parse().unwrap(),
            router: "0x1b02dA8Cb0d097eB8D57A175b88c7D8b47997506".parse().unwrap(),
            master_chef: "0xF4d73326C13a4Fc5FD7A064217e12780e9Bd62c3".parse().unwrap(),
            sushi_token: "0xd4d42F0b6DEF4CE0383636770eF773390d85c61A".parse().unwrap(),
        }
    }
}

pub struct SushiSwapManager {
    chain_manager: Arc<ChainManager>,
    contracts: HashMap<u64, SushiSwapContracts>,
    pairs_cache: Arc<tokio::sync::RwLock<HashMap<Address, PairInfo>>>,
    farms_cache: Arc<tokio::sync::RwLock<HashMap<u64, FarmInfo>>>,
}

impl SushiSwapManager {
    pub async fn new(chain_manager: Arc<ChainManager>) -> Result<Self> {
        info!("Initializing SushiSwap Manager");

        let mut contracts = HashMap::new();
        contracts.insert(1, SushiSwapContracts::for_chain(1));
        contracts.insert(137, SushiSwapContracts::for_chain(137));
        contracts.insert(42161, SushiSwapContracts::for_chain(42161));

        Ok(Self {
            chain_manager,
            contracts,
            pairs_cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            farms_cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        })
    }

    pub async fn new_demo() -> Result<Self> {
        info!("Creating SushiSwapManager in demo mode");
        
        let chain_manager = Arc::new(ChainManager::new_demo().await?);
        let contracts = HashMap::new(); // Empty contracts for demo
        
        Ok(Self {
            chain_manager,
            contracts,
            pairs_cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            farms_cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        })
    }

    /// Get pair information
    pub async fn get_pair_info(&self, chain_id: u64, token0: Address, token1: Address) -> Result<PairInfo> {
        info!("Getting pair info for tokens {:?}/{:?} on chain {}", token0, token1, chain_id);

        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Chain {} not supported", chain_id))?;

        let chain_provider = self.chain_manager.get_provider(chain_id).await?;
        let provider = Arc::new(chain_provider.provider.clone());

        // Get factory contract
        let factory_abi = Self::get_factory_abi()?;
        let factory = Contract::new(contracts.factory, factory_abi, provider.clone());

        // Get pair address
        let pair_address: Address = factory
            .method::<_, Address>("getPair", (token0, token1))?
            .call()
            .await?;

        if pair_address == Address::zero() {
            return Err(anyhow!("Pair does not exist"));
        }

        // Get pair contract
        let pair_abi = Self::get_pair_abi()?;
        let pair_contract = Contract::new(pair_address, pair_abi, provider);

        // Get reserves
        let reserves: (U256, U256, u32) = pair_contract
            .method::<_, (U256, U256, u32)>("getReserves", ())?
            .call()
            .await?;

        let price0_cumulative_last: U256 = pair_contract
            .method::<_, U256>("price0CumulativeLast", ())?
            .call()
            .await?;

        let price1_cumulative_last: U256 = pair_contract
            .method::<_, U256>("price1CumulativeLast", ())?
            .call()
            .await?;

        let k_last: U256 = pair_contract
            .method::<_, U256>("kLast", ())?
            .call()
            .await?;

        let pair_info = PairInfo {
            address: pair_address,
            token0,
            token1,
            reserves,
            price0_cumulative_last,
            price1_cumulative_last,
            k_last,
        };

        // Cache the pair info
        self.pairs_cache.write().await.insert(pair_address, pair_info.clone());

        Ok(pair_info)
    }

    /// Swap exact tokens for tokens
    pub async fn swap_exact_tokens_for_tokens(
        &self,
        chain_id: u64,
        amount_in: U256,
        amount_out_min: U256,
        path: Vec<Address>,
        to: Address,
        deadline: u64,
    ) -> Result<TransactionRequest> {
        info!("Creating swap transaction for {} tokens", amount_in);

        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Chain {} not supported", chain_id))?;

        let chain_provider = self.chain_manager.get_provider(chain_id).await?;
        let provider = Arc::new(chain_provider.provider.clone());

        let router_abi = Self::get_router_abi()?;
        let router = Contract::new(contracts.router, router_abi, provider);

        let call = router.method::<_, Vec<U256>>(
            "swapExactTokensForTokens",
            (amount_in, amount_out_min, path, to, deadline),
        )?;

        let tx = TransactionRequest::new()
            .to(contracts.router)
            .data(call.calldata().unwrap_or_default());

        Ok(tx)
    }

    /// Add liquidity to a pair
    pub async fn add_liquidity(
        &self,
        chain_id: u64,
        token_a: Address,
        token_b: Address,
        amount_a_desired: U256,
        amount_b_desired: U256,
        amount_a_min: U256,
        amount_b_min: U256,
        to: Address,
        deadline: u64,
    ) -> Result<TransactionRequest> {
        info!("Creating add liquidity transaction for {}/{}", token_a, token_b);

        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Chain {} not supported", chain_id))?;

        let chain_provider = self.chain_manager.get_provider(chain_id).await?;
        let provider = Arc::new(chain_provider.provider.clone());

        let router_abi = Self::get_router_abi()?;
        let router = Contract::new(contracts.router, router_abi, provider);

        let call = router.method::<_, (U256, U256, U256)>(
            "addLiquidity",
            (
                token_a,
                token_b,
                amount_a_desired,
                amount_b_desired,
                amount_a_min,
                amount_b_min,
                to,
                deadline,
            ),
        )?;

        let tx = TransactionRequest::new()
            .to(contracts.router)
            .data(call.calldata().unwrap_or_default());

        Ok(tx)
    }

    /// Remove liquidity from a pair
    pub async fn remove_liquidity(
        &self,
        chain_id: u64,
        token_a: Address,
        token_b: Address,
        liquidity: U256,
        amount_a_min: U256,
        amount_b_min: U256,
        to: Address,
        deadline: u64,
    ) -> Result<TransactionRequest> {
        info!("Creating remove liquidity transaction");

        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Chain {} not supported", chain_id))?;

        let chain_provider = self.chain_manager.get_provider(chain_id).await?;
        let provider = Arc::new(chain_provider.provider.clone());

        let router_abi = Self::get_router_abi()?;
        let router = Contract::new(contracts.router, router_abi, provider);

        let call = router.method::<_, (U256, U256)>(
            "removeLiquidity",
            (
                token_a,
                token_b,
                liquidity,
                amount_a_min,
                amount_b_min,
                to,
                deadline,
            ),
        )?;

        let tx = TransactionRequest::new()
            .to(contracts.router)
            .data(call.calldata().unwrap_or_default());

        Ok(tx)
    }

    /// Get farm information
    pub async fn get_farm_info(&self, chain_id: u64, pid: u64) -> Result<FarmInfo> {
        info!("Getting farm info for pool {}", pid);

        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Chain {} not supported", chain_id))?;

        let chain_provider = self.chain_manager.get_provider(chain_id).await?;
        let provider = Arc::new(chain_provider.provider.clone());

        let master_chef_abi = Self::get_master_chef_abi()?;
        let master_chef = Contract::new(contracts.master_chef, master_chef_abi, provider);

        // Get pool info
        let pool_info: (Address, U256, u64, U256) = master_chef
            .method::<_, (Address, U256, u64, U256)>("poolInfo", pid)?
            .call()
            .await?;

        let reward_per_block: U256 = master_chef
            .method::<_, U256>("sushiPerBlock", ())?
            .call()
            .await
            .unwrap_or_default();

        let total_alloc_point: U256 = master_chef
            .method::<_, U256>("totalAllocPoint", ())?
            .call()
            .await
            .unwrap_or_default();

        // Calculate APY (simplified)
        let apy = if total_alloc_point > U256::zero() {
            let pool_reward_per_block = reward_per_block * pool_info.1 / total_alloc_point;
            // This is a simplified APY calculation - in reality you'd need token prices
            pool_reward_per_block.as_u64() as f64 * 0.1 // Mock calculation
        } else {
            0.0
        };

        let farm_info = FarmInfo {
            pid,
            lp_token: pool_info.0,
            alloc_point: pool_info.1,
            last_reward_block: pool_info.2,
            acc_sushi_per_share: pool_info.3,
            reward_per_block,
            total_staked: U256::zero(), // Would need additional call
            apy,
        };

        self.farms_cache.write().await.insert(pid, farm_info.clone());
        Ok(farm_info)
    }

    /// Stake LP tokens in farm
    pub async fn stake_in_farm(
        &self,
        chain_id: u64,
        pid: u64,
        amount: U256,
    ) -> Result<TransactionRequest> {
        info!("Creating stake transaction for pool {} amount {}", pid, amount);

        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Chain {} not supported", chain_id))?;

        let chain_provider = self.chain_manager.get_provider(chain_id).await?;
        let provider = Arc::new(chain_provider.provider.clone());

        let master_chef_abi = Self::get_master_chef_abi()?;
        let master_chef = Contract::new(contracts.master_chef, master_chef_abi, provider);

        let call = master_chef.method::<_, ()>("deposit", (pid, amount))?;

        let tx = TransactionRequest::new()
            .to(contracts.master_chef)
            .data(call.calldata().unwrap_or_default());

        Ok(tx)
    }

    /// Unstake LP tokens from farm
    pub async fn unstake_from_farm(
        &self,
        chain_id: u64,
        pid: u64,
        amount: U256,
    ) -> Result<TransactionRequest> {
        info!("Creating unstake transaction for pool {} amount {}", pid, amount);

        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Chain {} not supported", chain_id))?;

        let chain_provider = self.chain_manager.get_provider(chain_id).await?;
        let provider = Arc::new(chain_provider.provider.clone());

        let master_chef_abi = Self::get_master_chef_abi()?;
        let master_chef = Contract::new(contracts.master_chef, master_chef_abi, provider);

        let call = master_chef.method::<_, ()>("withdraw", (pid, amount))?;

        let tx = TransactionRequest::new()
            .to(contracts.master_chef)
            .data(call.calldata().unwrap_or_default());

        Ok(tx)
    }

    /// Get user farming position
    pub async fn get_user_position(&self, chain_id: u64, pid: u64, user: Address) -> Result<UserPosition> {
        info!("Getting user position for pool {} user {:?}", pid, user);

        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Chain {} not supported", chain_id))?;

        let chain_provider = self.chain_manager.get_provider(chain_id).await?;
        let provider = Arc::new(chain_provider.provider.clone());

        let master_chef_abi = Self::get_master_chef_abi()?;
        let master_chef = Contract::new(contracts.master_chef, master_chef_abi, provider);

        let user_info: (U256, U256) = master_chef
            .method::<_, (U256, U256)>("userInfo", (pid, user))?
            .call()
            .await?;

        let pending_rewards: U256 = master_chef
            .method::<_, U256>("pendingSushi", (pid, user))?
            .call()
            .await?;

        Ok(UserPosition {
            pid,
            amount: user_info.0,
            reward_debt: user_info.1,
            pending_rewards,
        })
    }

    /// Get amounts out for a swap
    pub async fn get_amounts_out(&self, chain_id: u64, amount_in: U256, path: Vec<Address>) -> Result<Vec<U256>> {
        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Chain {} not supported", chain_id))?;

        let chain_provider = self.chain_manager.get_provider(chain_id).await?;
        let provider = Arc::new(chain_provider.provider.clone());

        let router_abi = Self::get_router_abi()?;
        let router = Contract::new(contracts.router, router_abi, provider);

        let amounts: Vec<U256> = router
            .method::<_, Vec<U256>>("getAmountsOut", (amount_in, path))?
            .call()
            .await?;

        Ok(amounts)
    }

    /// Get all available farms
    pub async fn get_all_farms(&self, chain_id: u64) -> Result<Vec<FarmInfo>> {
        info!("Getting all farms for chain {}", chain_id);
        
        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Chain {} not supported", chain_id))?;

        let _provider = self.chain_manager.get_provider(chain_id).await?;
        // Note: Contract interaction would be implemented here in production

        // Get the number of pools (mock implementation for now)
        let pool_length = 10u64; // In reality, this would be fetched from the contract

        let mut farms = Vec::new();
        
        for i in 0..pool_length.min(10) { // Limit to first 10 for demo
            // Create mock farm info - in reality this would be fetched from contract
            let farm = FarmInfo {
                pid: i,
                lp_token: Address::from_low_u64_be(0x1000 + i), // Mock address
                alloc_point: U256::from(100),
                last_reward_block: 1000000 + i,
                acc_sushi_per_share: U256::zero(),
                reward_per_block: U256::from(1000),
                total_staked: U256::from(10000000), // Mock 10M tokens staked
                apy: 15.5, // Mock 15.5% APY
            };
            farms.push(farm);
        }

        Ok(farms)
    }

    // ABI helper methods
    fn get_factory_abi() -> Result<Abi> {
        let abi_json = r#"[
            {
                "inputs": [
                    {"internalType": "address", "name": "tokenA", "type": "address"},
                    {"internalType": "address", "name": "tokenB", "type": "address"}
                ],
                "name": "getPair",
                "outputs": [{"internalType": "address", "name": "pair", "type": "address"}],
                "stateMutability": "view",
                "type": "function"
            }
        ]"#;
        
        Ok(serde_json::from_str(abi_json)?)
    }

    fn get_pair_abi() -> Result<Abi> {
        let abi_json = r#"[
            {
                "inputs": [],
                "name": "getReserves",
                "outputs": [
                    {"internalType": "uint112", "name": "reserve0", "type": "uint112"},
                    {"internalType": "uint112", "name": "reserve1", "type": "uint112"},
                    {"internalType": "uint32", "name": "blockTimestampLast", "type": "uint32"}
                ],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [],
                "name": "price0CumulativeLast",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [],
                "name": "price1CumulativeLast",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [],
                "name": "kLast",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "view",
                "type": "function"
            }
        ]"#;
        
        Ok(serde_json::from_str(abi_json)?)
    }

    fn get_router_abi() -> Result<Abi> {
        let abi_json = r#"[
            {
                "inputs": [
                    {"internalType": "uint256", "name": "amountIn", "type": "uint256"},
                    {"internalType": "uint256", "name": "amountOutMin", "type": "uint256"},
                    {"internalType": "address[]", "name": "path", "type": "address[]"},
                    {"internalType": "address", "name": "to", "type": "address"},
                    {"internalType": "uint256", "name": "deadline", "type": "uint256"}
                ],
                "name": "swapExactTokensForTokens",
                "outputs": [{"internalType": "uint256[]", "name": "amounts", "type": "uint256[]"}],
                "stateMutability": "nonpayable",
                "type": "function"
            },
            {
                "inputs": [
                    {"internalType": "address", "name": "tokenA", "type": "address"},
                    {"internalType": "address", "name": "tokenB", "type": "address"},
                    {"internalType": "uint256", "name": "amountADesired", "type": "uint256"},
                    {"internalType": "uint256", "name": "amountBDesired", "type": "uint256"},
                    {"internalType": "uint256", "name": "amountAMin", "type": "uint256"},
                    {"internalType": "uint256", "name": "amountBMin", "type": "uint256"},
                    {"internalType": "address", "name": "to", "type": "address"},
                    {"internalType": "uint256", "name": "deadline", "type": "uint256"}
                ],
                "name": "addLiquidity",
                "outputs": [
                    {"internalType": "uint256", "name": "amountA", "type": "uint256"},
                    {"internalType": "uint256", "name": "amountB", "type": "uint256"},
                    {"internalType": "uint256", "name": "liquidity", "type": "uint256"}
                ],
                "stateMutability": "nonpayable",
                "type": "function"
            },
            {
                "inputs": [
                    {"internalType": "address", "name": "tokenA", "type": "address"},
                    {"internalType": "address", "name": "tokenB", "type": "address"},
                    {"internalType": "uint256", "name": "liquidity", "type": "uint256"},
                    {"internalType": "uint256", "name": "amountAMin", "type": "uint256"},
                    {"internalType": "uint256", "name": "amountBMin", "type": "uint256"},
                    {"internalType": "address", "name": "to", "type": "address"},
                    {"internalType": "uint256", "name": "deadline", "type": "uint256"}
                ],
                "name": "removeLiquidity",
                "outputs": [
                    {"internalType": "uint256", "name": "amountA", "type": "uint256"},
                    {"internalType": "uint256", "name": "amountB", "type": "uint256"}
                ],
                "stateMutability": "nonpayable",
                "type": "function"
            },
            {
                "inputs": [
                    {"internalType": "uint256", "name": "amountIn", "type": "uint256"},
                    {"internalType": "address[]", "name": "path", "type": "address[]"}
                ],
                "name": "getAmountsOut",
                "outputs": [{"internalType": "uint256[]", "name": "amounts", "type": "uint256[]"}],
                "stateMutability": "view",
                "type": "function"
            }
        ]"#;
        
        Ok(serde_json::from_str(abi_json)?)
    }

    fn get_master_chef_abi() -> Result<Abi> {
        let abi_json = r#"[
            {
                "inputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "name": "poolInfo",
                "outputs": [
                    {"internalType": "address", "name": "lpToken", "type": "address"},
                    {"internalType": "uint256", "name": "allocPoint", "type": "uint256"},
                    {"internalType": "uint256", "name": "lastRewardBlock", "type": "uint256"},
                    {"internalType": "uint256", "name": "accSushiPerShare", "type": "uint256"}
                ],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [
                    {"internalType": "uint256", "name": "", "type": "uint256"},
                    {"internalType": "address", "name": "", "type": "address"}
                ],
                "name": "userInfo",
                "outputs": [
                    {"internalType": "uint256", "name": "amount", "type": "uint256"},
                    {"internalType": "uint256", "name": "rewardDebt", "type": "uint256"}
                ],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [
                    {"internalType": "uint256", "name": "pid", "type": "uint256"},
                    {"internalType": "uint256", "name": "amount", "type": "uint256"}
                ],
                "name": "deposit",
                "outputs": [],
                "stateMutability": "nonpayable",
                "type": "function"
            },
            {
                "inputs": [
                    {"internalType": "uint256", "name": "pid", "type": "uint256"},
                    {"internalType": "uint256", "name": "amount", "type": "uint256"}
                ],
                "name": "withdraw",
                "outputs": [],
                "stateMutability": "nonpayable",
                "type": "function"
            },
            {
                "inputs": [
                    {"internalType": "uint256", "name": "pid", "type": "uint256"},
                    {"internalType": "address", "name": "user", "type": "address"}
                ],
                "name": "pendingSushi",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [],
                "name": "sushiPerBlock",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [],
                "name": "totalAllocPoint",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "view",
                "type": "function"
            }
        ]"#;
        
        Ok(serde_json::from_str(abi_json)?)
    }
}
