use anyhow::{Result, anyhow};
use ethers::{
    abi::{Abi, Token},
    contract::Contract,
    providers::{Provider, Http},
    types::{Address, U256, TransactionRequest, Bytes, H256},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;
use tracing::{info, warn, error};

use crate::chains::ChainManager;
use crate::contracts::erc20::ERC20Contract;

/// Uniswap V3 pool information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolInfo {
    pub address: Address,
    pub token0: Address,
    pub token1: Address,
    pub fee: u32,
    pub tick_spacing: i32,
    pub liquidity: U256,
    pub sqrt_price_x96: U256,
    pub tick: i32,
    pub fee_growth_global0_x128: U256,
    pub fee_growth_global1_x128: U256,
}

/// Token swap parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapParams {
    pub token_in: Address,
    pub token_out: Address,
    pub amount_in: U256,
    pub amount_out_minimum: U256,
    pub fee: u32,
    pub recipient: Address,
    pub deadline: u64,
    pub sqrt_price_limit_x96: U256,
}

/// Liquidity position information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPosition {
    pub token_id: U256,
    pub pool: Address,
    pub token0: Address,
    pub token1: Address,
    pub fee: u32,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub liquidity: U256,
    pub fee_growth_inside0_last_x128: U256,
    pub fee_growth_inside1_last_x128: U256,
    pub tokens_owed0: U256,
    pub tokens_owed1: U256,
}

/// Price and liquidity data for a pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolData {
    pub pool_info: PoolInfo,
    pub token0_price: f64,
    pub token1_price: f64,
    pub volume_24h: U256,
    pub tvl: U256,
    pub fee_apr: f64,
}

/// Uniswap V3 contract addresses for different chains
#[derive(Debug, Clone)]
pub struct UniswapContracts {
    pub factory: Address,
    pub router: Address,
    pub position_manager: Address,
    pub quoter: Address,
}

impl UniswapContracts {
    /// Get contract addresses for a specific chain
    pub fn for_chain(chain_id: u64) -> Self {
        match chain_id {
            1 => Self::ethereum_mainnet(),
            137 => Self::polygon(),
            42161 => Self::arbitrum(),
            _ => Self::ethereum_mainnet(), // Default to mainnet
        }
    }

    fn ethereum_mainnet() -> Self {
        Self {
            factory: "0x1F98431c8aD98523631AE4a59f267346ea31F984".parse().unwrap(),
            router: "0xE592427A0AEce92De3Edee1F18E0157C05861564".parse().unwrap(),
            position_manager: "0xC36442b4a4522E871399CD717aBDD847Ab11FE88".parse().unwrap(),
            quoter: "0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6".parse().unwrap(),
        }
    }

    fn polygon() -> Self {
        Self {
            factory: "0x1F98431c8aD98523631AE4a59f267346ea31F984".parse().unwrap(),
            router: "0xE592427A0AEce92De3Edee1F18E0157C05861564".parse().unwrap(),
            position_manager: "0xC36442b4a4522E871399CD717aBDD847Ab11FE88".parse().unwrap(),
            quoter: "0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6".parse().unwrap(),
        }
    }

    fn arbitrum() -> Self {
        Self {
            factory: "0x1F98431c8aD98523631AE4a59f267346ea31F984".parse().unwrap(),
            router: "0xE592427A0AEce92De3Edee1F18E0157C05861564".parse().unwrap(),
            position_manager: "0xC36442b4a4522E871399CD717aBDD847Ab11FE88".parse().unwrap(),
            quoter: "0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6".parse().unwrap(),
        }
    }
}

pub struct UniswapV3Manager {
    chain_manager: Arc<ChainManager>,
    contracts: HashMap<u64, UniswapContracts>,
    pools_cache: Arc<tokio::sync::RwLock<HashMap<Address, PoolData>>>,
}

impl UniswapV3Manager {
    pub async fn new(chain_manager: Arc<ChainManager>) -> Result<Self> {
        info!("Initializing Uniswap V3 Manager");

        let mut contracts = HashMap::new();
        
        // Initialize contracts for supported chains
        contracts.insert(1, UniswapContracts::for_chain(1));     // Ethereum
        contracts.insert(137, UniswapContracts::for_chain(137)); // Polygon
        contracts.insert(42161, UniswapContracts::for_chain(42161)); // Arbitrum

        Ok(Self {
            chain_manager,
            contracts,
            pools_cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        })
    }

    pub async fn new_demo() -> Result<Self> {
        info!("Creating UniswapV3Manager in demo mode");
        
        let chain_manager = Arc::new(ChainManager::new_demo().await?);
        let contracts = HashMap::new(); // Empty contracts for demo
        
        Ok(Self {
            chain_manager,
            contracts,
            pools_cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        })
    }

    /// Get pool information for a trading pair
    pub async fn get_pool_info(&self, chain_id: u64, token0: Address, token1: Address, fee: u32) -> Result<PoolInfo> {
        info!("Getting pool info for tokens {:?}/{:?} on chain {}", token0, token1, chain_id);

        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Chain {} not supported", chain_id))?;

        let chain_provider = self.chain_manager.get_provider(chain_id).await?;
        let provider = Arc::new(chain_provider.provider.clone());

        // Get factory contract
        let factory_abi = Self::get_factory_abi()?;
        let factory = Contract::new(contracts.factory, factory_abi, provider.clone());

        // Get pool address
        let pool_address: Address = factory
            .method::<_, Address>("getPool", (token0, token1, fee))?
            .call()
            .await?;

        if pool_address == Address::zero() {
            return Err(anyhow!("Pool does not exist for this pair and fee tier"));
        }

        // Get pool contract
        let pool_abi = Self::get_pool_abi()?;
        let pool_contract = Contract::new(pool_address, pool_abi, provider);

        // Get pool state
        let slot0: (U256, i32, u16, u16, u16, u8, bool) = pool_contract
            .method::<_, (U256, i32, u16, u16, u16, u8, bool)>("slot0", ())?
            .call()
            .await?;

        let liquidity: U256 = pool_contract
            .method::<_, U256>("liquidity", ())?
            .call()
            .await?;

        let tick_spacing: i32 = pool_contract
            .method::<_, i32>("tickSpacing", ())?
            .call()
            .await?;

        let fee_growth_global0_x128: U256 = pool_contract
            .method::<_, U256>("feeGrowthGlobal0X128", ())?
            .call()
            .await?;

        let fee_growth_global1_x128: U256 = pool_contract
            .method::<_, U256>("feeGrowthGlobal1X128", ())?
            .call()
            .await?;

        let pool_info = PoolInfo {
            address: pool_address,
            token0,
            token1,
            fee,
            tick_spacing,
            liquidity,
            sqrt_price_x96: slot0.0,
            tick: slot0.1,
            fee_growth_global0_x128,
            fee_growth_global1_x128,
        };

        info!("Retrieved pool info: {:?}", pool_info);
        Ok(pool_info)
    }

    /// Execute a token swap
    pub async fn swap_exact_input_single(
        &self,
        chain_id: u64,
        params: SwapParams,
    ) -> Result<TransactionRequest> {
        info!("Creating swap transaction for {} -> {} on chain {}", 
              params.token_in, params.token_out, chain_id);

        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Chain {} not supported", chain_id))?;

        let chain_provider = self.chain_manager.get_provider(chain_id).await?;
        let provider = Arc::new(chain_provider.provider.clone());

        // Get router contract
        let router_abi = Self::get_router_abi()?;
        let router = Contract::new(contracts.router, router_abi, provider);

        // Prepare swap parameters
        let exact_input_single_params = (
            params.token_in,
            params.token_out,
            params.fee,
            params.recipient,
            params.deadline,
            params.amount_in,
            params.amount_out_minimum,
            params.sqrt_price_limit_x96,
        );

        let call = router
            .method::<_, U256>("exactInputSingle", exact_input_single_params)?;

        let tx = TransactionRequest::new()
            .to(contracts.router)
            .data(call.calldata().unwrap_or_default());

        Ok(tx)
    }

    /// Get quote for a swap
    pub async fn quote_exact_input_single(
        &self,
        chain_id: u64,
        token_in: Address,
        token_out: Address,
        fee: u32,
        amount_in: U256,
        sqrt_price_limit_x96: U256,
    ) -> Result<U256> {
        info!("Getting quote for {} tokens {} -> {}", amount_in, token_in, token_out);

        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Chain {} not supported", chain_id))?;

        let chain_provider = self.chain_manager.get_provider(chain_id).await?;
        let provider = Arc::new(chain_provider.provider.clone());

        let quoter_abi = Self::get_quoter_abi()?;
        let quoter = Contract::new(contracts.quoter, quoter_abi, provider);

        let quote: U256 = quoter
            .method::<_, U256>("quoteExactInputSingle", (
                token_in,
                token_out,
                fee,
                amount_in,
                sqrt_price_limit_x96,
            ))?
            .call()
            .await?;

        info!("Quote result: {} output tokens", quote);
        Ok(quote)
    }

    /// Add liquidity to a pool
    pub async fn add_liquidity(
        &self,
        chain_id: u64,
        token0: Address,
        token1: Address,
        fee: u32,
        tick_lower: i32,
        tick_upper: i32,
        amount0_desired: U256,
        amount1_desired: U256,
        amount0_min: U256,
        amount1_min: U256,
        recipient: Address,
        deadline: u64,
    ) -> Result<TransactionRequest> {
        info!("Creating add liquidity transaction for pool {}/{}", token0, token1);

        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Chain {} not supported", chain_id))?;

        let chain_provider = self.chain_manager.get_provider(chain_id).await?;
        let provider = Arc::new(chain_provider.provider.clone());

        let position_manager_abi = Self::get_position_manager_abi()?;
        let position_manager = Contract::new(contracts.position_manager, position_manager_abi, provider);

        let mint_params = (
            token0,
            token1,
            fee,
            tick_lower,
            tick_upper,
            amount0_desired,
            amount1_desired,
            amount0_min,
            amount1_min,
            recipient,
            deadline,
        );

        let call = position_manager
            .method::<_, (U256, U256, U256, U256)>("mint", mint_params)?;

        let tx = TransactionRequest::new()
            .to(contracts.position_manager)
            .data(call.calldata().unwrap_or_default());

        Ok(tx)
    }

    /// Remove liquidity from a position
    pub async fn remove_liquidity(
        &self,
        chain_id: u64,
        token_id: U256,
        liquidity: U256,
        amount0_min: U256,
        amount1_min: U256,
        deadline: u64,
    ) -> Result<TransactionRequest> {
        info!("Creating remove liquidity transaction for position {}", token_id);

        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Chain {} not supported", chain_id))?;

        let chain_provider = self.chain_manager.get_provider(chain_id).await?;
        let provider = Arc::new(chain_provider.provider.clone());

        let position_manager_abi = Self::get_position_manager_abi()?;
        let position_manager = Contract::new(contracts.position_manager, position_manager_abi, provider);

        let decrease_params = (
            token_id,
            liquidity,
            amount0_min,
            amount1_min,
            deadline,
        );

        let call = position_manager
            .method::<_, (U256, U256)>("decreaseLiquidity", decrease_params)?;

        let tx = TransactionRequest::new()
            .to(contracts.position_manager)
            .data(call.calldata().unwrap_or_default());

        Ok(tx)
    }

    /// Get all liquidity positions for an address
    pub async fn get_positions(&self, chain_id: u64, owner: Address) -> Result<Vec<LiquidityPosition>> {
        info!("Getting liquidity positions for address {:?}", owner);

        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Chain {} not supported", chain_id))?;

        let chain_provider = self.chain_manager.get_provider(chain_id).await?;
        let provider = Arc::new(chain_provider.provider.clone());

        let position_manager_abi = Self::get_position_manager_abi()?;
        let position_manager = Contract::new(contracts.position_manager, position_manager_abi, provider);

        // Get balance (number of NFT positions)
        let balance: U256 = position_manager
            .method::<_, U256>("balanceOf", owner)?
            .call()
            .await?;

        let mut positions = Vec::new();

        for i in 0..balance.as_u64() {
            let token_id: U256 = position_manager
                .method::<_, U256>("tokenOfOwnerByIndex", (owner, U256::from(i)))?
                .call()
                .await?;

            // Get position details
            let position_data: (
                U256, Address, Address, Address, u32, i32, i32, u128, U256, U256, u128, u128
            ) = position_manager
                .method("positions", token_id)?
                .call()
                .await?;

            // Find pool address
            let pool_address = self.get_pool_address(chain_id, position_data.2, position_data.3, position_data.4).await?;

            let position = LiquidityPosition {
                token_id,
                pool: pool_address,
                token0: position_data.2,
                token1: position_data.3,
                fee: position_data.4,
                tick_lower: position_data.5,
                tick_upper: position_data.6,
                liquidity: U256::from(position_data.7),
                fee_growth_inside0_last_x128: position_data.9,
                fee_growth_inside1_last_x128: position_data.10.into(),
                tokens_owed0: U256::from(position_data.11),
                tokens_owed1: U256::from(position_data.11), // Note: This should be different for token1
            };

            positions.push(position);
        }

        info!("Found {} positions for owner", positions.len());
        Ok(positions)
    }

    /// Calculate optimal tick range for liquidity provision
    pub async fn calculate_optimal_range(
        &self,
        chain_id: u64,
        token0: Address,
        token1: Address,
        fee: u32,
        range_factor: f64,
    ) -> Result<(i32, i32)> {
        let pool_info = self.get_pool_info(chain_id, token0, token1, fee).await?;
        
        // Calculate range based on current price and volatility
        let current_tick = pool_info.tick;
        let tick_spacing = pool_info.tick_spacing;
        
        // Range calculation based on fee tier and range factor
        let range_ticks = match fee {
            500 => (range_factor * 200.0) as i32,   // 0.05% fee - tight range
            3000 => (range_factor * 600.0) as i32,  // 0.3% fee - medium range  
            10000 => (range_factor * 2000.0) as i32, // 1% fee - wide range
            _ => (range_factor * 600.0) as i32,
        };

        // Align to tick spacing
        let tick_lower = ((current_tick - range_ticks) / tick_spacing) * tick_spacing;
        let tick_upper = ((current_tick + range_ticks) / tick_spacing) * tick_spacing;

        info!("Calculated optimal range: {} to {} (current: {})", tick_lower, tick_upper, current_tick);
        Ok((tick_lower, tick_upper))
    }

    // Helper methods for getting pool address
    async fn get_pool_address(&self, chain_id: u64, token0: Address, token1: Address, fee: u32) -> Result<Address> {
        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Chain {} not supported", chain_id))?;

        let chain_provider = self.chain_manager.get_provider(chain_id).await?;
        let provider = Arc::new(chain_provider.provider.clone());

        let factory_abi = Self::get_factory_abi()?;
        let factory = Contract::new(contracts.factory, factory_abi, provider);

        let pool_address: Address = factory
            .method::<_, Address>("getPool", (token0, token1, fee))?
            .call()
            .await?;

        Ok(pool_address)
    }

    // ABI helper methods
    fn get_factory_abi() -> Result<Abi> {
        let abi_json = r#"[
            {
                "inputs": [
                    {"internalType": "address", "name": "tokenA", "type": "address"},
                    {"internalType": "address", "name": "tokenB", "type": "address"},
                    {"internalType": "uint24", "name": "fee", "type": "uint24"}
                ],
                "name": "getPool",
                "outputs": [{"internalType": "address", "name": "pool", "type": "address"}],
                "stateMutability": "view",
                "type": "function"
            }
        ]"#;
        
        Ok(serde_json::from_str(abi_json)?)
    }

    fn get_pool_abi() -> Result<Abi> {
        let abi_json = r#"[
            {
                "inputs": [],
                "name": "slot0",
                "outputs": [
                    {"internalType": "uint160", "name": "sqrtPriceX96", "type": "uint160"},
                    {"internalType": "int24", "name": "tick", "type": "int24"},
                    {"internalType": "uint16", "name": "observationIndex", "type": "uint16"},
                    {"internalType": "uint16", "name": "observationCardinality", "type": "uint16"},
                    {"internalType": "uint16", "name": "observationCardinalityNext", "type": "uint16"},
                    {"internalType": "uint8", "name": "feeProtocol", "type": "uint8"},
                    {"internalType": "bool", "name": "unlocked", "type": "bool"}
                ],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [],
                "name": "liquidity",
                "outputs": [{"internalType": "uint128", "name": "", "type": "uint128"}],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [],
                "name": "tickSpacing",
                "outputs": [{"internalType": "int24", "name": "", "type": "int24"}],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [],
                "name": "feeGrowthGlobal0X128",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [],
                "name": "feeGrowthGlobal1X128",
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
                    {
                        "components": [
                            {"internalType": "address", "name": "tokenIn", "type": "address"},
                            {"internalType": "address", "name": "tokenOut", "type": "address"},
                            {"internalType": "uint24", "name": "fee", "type": "uint24"},
                            {"internalType": "address", "name": "recipient", "type": "address"},
                            {"internalType": "uint256", "name": "deadline", "type": "uint256"},
                            {"internalType": "uint256", "name": "amountIn", "type": "uint256"},
                            {"internalType": "uint256", "name": "amountOutMinimum", "type": "uint256"},
                            {"internalType": "uint160", "name": "sqrtPriceLimitX96", "type": "uint160"}
                        ],
                        "internalType": "struct ISwapRouter.ExactInputSingleParams",
                        "name": "params",
                        "type": "tuple"
                    }
                ],
                "name": "exactInputSingle",
                "outputs": [{"internalType": "uint256", "name": "amountOut", "type": "uint256"}],
                "stateMutability": "payable",
                "type": "function"
            }
        ]"#;
        
        Ok(serde_json::from_str(abi_json)?)
    }

    fn get_quoter_abi() -> Result<Abi> {
        let abi_json = r#"[
            {
                "inputs": [
                    {"internalType": "address", "name": "tokenIn", "type": "address"},
                    {"internalType": "address", "name": "tokenOut", "type": "address"},
                    {"internalType": "uint24", "name": "fee", "type": "uint24"},
                    {"internalType": "uint256", "name": "amountIn", "type": "uint256"},
                    {"internalType": "uint160", "name": "sqrtPriceLimitX96", "type": "uint160"}
                ],
                "name": "quoteExactInputSingle",
                "outputs": [{"internalType": "uint256", "name": "amountOut", "type": "uint256"}],
                "stateMutability": "nonpayable",
                "type": "function"
            }
        ]"#;
        
        Ok(serde_json::from_str(abi_json)?)
    }

    fn get_position_manager_abi() -> Result<Abi> {
        let abi_json = r#"[
            {
                "inputs": [{"internalType": "address", "name": "owner", "type": "address"}],
                "name": "balanceOf",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [
                    {"internalType": "address", "name": "owner", "type": "address"},
                    {"internalType": "uint256", "name": "index", "type": "uint256"}
                ],
                "name": "tokenOfOwnerByIndex",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [{"internalType": "uint256", "name": "tokenId", "type": "uint256"}],
                "name": "positions",
                "outputs": [
                    {"internalType": "uint96", "name": "nonce", "type": "uint96"},
                    {"internalType": "address", "name": "operator", "type": "address"},
                    {"internalType": "address", "name": "token0", "type": "address"},
                    {"internalType": "address", "name": "token1", "type": "address"},
                    {"internalType": "uint24", "name": "fee", "type": "uint24"},
                    {"internalType": "int24", "name": "tickLower", "type": "int24"},
                    {"internalType": "int24", "name": "tickUpper", "type": "int24"},
                    {"internalType": "uint128", "name": "liquidity", "type": "uint128"},
                    {"internalType": "uint256", "name": "feeGrowthInside0LastX128", "type": "uint256"},
                    {"internalType": "uint256", "name": "feeGrowthInside1LastX128", "type": "uint256"},
                    {"internalType": "uint128", "name": "tokensOwed0", "type": "uint128"},
                    {"internalType": "uint128", "name": "tokensOwed1", "type": "uint128"}
                ],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [
                    {
                        "components": [
                            {"internalType": "address", "name": "token0", "type": "address"},
                            {"internalType": "address", "name": "token1", "type": "address"},
                            {"internalType": "uint24", "name": "fee", "type": "uint24"},
                            {"internalType": "int24", "name": "tickLower", "type": "int24"},
                            {"internalType": "int24", "name": "tickUpper", "type": "int24"},
                            {"internalType": "uint256", "name": "amount0Desired", "type": "uint256"},
                            {"internalType": "uint256", "name": "amount1Desired", "type": "uint256"},
                            {"internalType": "uint256", "name": "amount0Min", "type": "uint256"},
                            {"internalType": "uint256", "name": "amount1Min", "type": "uint256"},
                            {"internalType": "address", "name": "recipient", "type": "address"},
                            {"internalType": "uint256", "name": "deadline", "type": "uint256"}
                        ],
                        "internalType": "struct INonfungiblePositionManager.MintParams",
                        "name": "params",
                        "type": "tuple"
                    }
                ],
                "name": "mint",
                "outputs": [
                    {"internalType": "uint256", "name": "tokenId", "type": "uint256"},
                    {"internalType": "uint128", "name": "liquidity", "type": "uint128"},
                    {"internalType": "uint256", "name": "amount0", "type": "uint256"},
                    {"internalType": "uint256", "name": "amount1", "type": "uint256"}
                ],
                "stateMutability": "payable",
                "type": "function"
            },
            {
                "inputs": [
                    {
                        "components": [
                            {"internalType": "uint256", "name": "tokenId", "type": "uint256"},
                            {"internalType": "uint128", "name": "liquidity", "type": "uint128"},
                            {"internalType": "uint256", "name": "amount0Min", "type": "uint256"},
                            {"internalType": "uint256", "name": "amount1Min", "type": "uint256"},
                            {"internalType": "uint256", "name": "deadline", "type": "uint256"}
                        ],
                        "internalType": "struct INonfungiblePositionManager.DecreaseLiquidityParams",
                        "name": "params",
                        "type": "tuple"
                    }
                ],
                "name": "decreaseLiquidity",
                "outputs": [
                    {"internalType": "uint256", "name": "amount0", "type": "uint256"},
                    {"internalType": "uint256", "name": "amount1", "type": "uint256"}
                ],
                "stateMutability": "payable",
                "type": "function"
            }
        ]"#;
        
        Ok(serde_json::from_str(abi_json)?)
    }
}
