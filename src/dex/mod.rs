use anyhow::Result;
use ethers::types::{Address, U256, TransactionRequest};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, error};

use crate::chains::ChainManager;

pub mod uniswap;
pub mod sushiswap;
pub mod aggregator;

use self::aggregator::{DexAggregator, QuoteComparison, SlippageSettings, PriceImpactAnalysis};

/// Comprehensive DEX management system
pub struct DexManager {
    chain_manager: Arc<ChainManager>,
    uniswap: uniswap::UniswapV3Manager,
    sushiswap: sushiswap::SushiSwapManager,
    aggregator: DexAggregator,
}

/// DEX operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexOperationResult {
    pub transaction: TransactionRequest,
    pub expected_output: U256,
    pub price_impact: f64,
    pub gas_estimate: U256,
    pub dex_used: String,
    pub savings_percentage: f64,
}

/// Liquidity provision result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityResult {
    pub add_transaction: Option<TransactionRequest>,
    pub remove_transaction: Option<TransactionRequest>,
    pub pool_address: Address,
    pub liquidity_amount: U256,
    pub token_amounts: (U256, U256),
}

/// DEX statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexStats {
    pub total_swaps: u64,
    pub total_volume: U256,
    pub average_savings: f64,
    pub best_dex_performance: String,
    pub price_impact_distribution: Vec<f64>,
}

/// Farming opportunity information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FarmingOpportunity {
    pub dex: String,
    pub pool_address: Address,
    pub token_a: Address,
    pub token_b: Address,
    pub apy: f64,
    pub total_liquidity: U256,
    pub reward_token: Address,
    pub user_staked: U256,
}

/// Trading pair information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingPair {
    pub token_a: Address,
    pub token_b: Address,
    pub dex: String,
    pub pool_address: Address,
    pub liquidity: U256,
    pub volume_24h: U256,
    pub fee_tier: u32,
}

impl DexManager {
    pub async fn new(chain_manager: Arc<ChainManager>) -> Result<Self> {
        info!("Initializing comprehensive DEX manager");

        let uniswap = uniswap::UniswapV3Manager::new(chain_manager.clone()).await?;
        let sushiswap = sushiswap::SushiSwapManager::new(chain_manager.clone()).await?;
        let aggregator = aggregator::DexAggregator::new().await?;

        Ok(Self {
            chain_manager,
            uniswap,
            sushiswap,
            aggregator,
        })
    }

    /// Execute optimal swap with automatic DEX selection
    pub async fn execute_optimal_swap(
        &self,
        chain_id: u64,
        token_in: Address,
        token_out: Address,
        amount_in: U256,
        recipient: Address,
        slippage_settings: Option<SlippageSettings>,
    ) -> Result<DexOperationResult> {
        info!("Executing optimal swap: {} {} -> {} on chain {}", 
               amount_in, token_in, token_out, chain_id);

        // Find best route across all DEXes
        let comparison = self.aggregator.find_best_route(
            &self.uniswap,
            &self.sushiswap,
            chain_id,
            token_in,
            token_out,
            amount_in,
            recipient,
        ).await?;

        // Execute with slippage protection
        let transaction = self.aggregator.execute_optimal_swap(
            &self.uniswap,
            &self.sushiswap,
            chain_id,
            token_in,
            token_out,
            amount_in,
            recipient,
            slippage_settings,
        ).await?;

        let result = DexOperationResult {
            transaction,
            expected_output: comparison.best_route.output_amount,
            price_impact: comparison.best_route.price_impact,
            gas_estimate: comparison.best_route.gas_estimate,
            dex_used: format!("{:?}", comparison.best_route.dex),
            savings_percentage: comparison.savings_percentage,
        };

        info!("Optimal swap prepared using {:?} with {}% savings", 
               comparison.best_route.dex, comparison.savings_percentage);

        Ok(result)
    }

    /// Get comprehensive quotes from all DEXes
    pub async fn get_comprehensive_quotes(
        &self,
        chain_id: u64,
        token_in: Address,
        token_out: Address,
        amount_in: U256,
        recipient: Address,
    ) -> Result<QuoteComparison> {
        info!("Getting comprehensive quotes for {} {} -> {} on chain {}",
               amount_in, token_in, token_out, chain_id);

        self.aggregator.find_best_route(
            &self.uniswap,
            &self.sushiswap,
            chain_id,
            token_in,
            token_out,
            amount_in,
            recipient,
        ).await
    }

    /// Analyze price impact and provide trading recommendations
    pub async fn analyze_trade_impact(
        &self,
        chain_id: u64,
        token_in: Address,
        token_out: Address,
        amount_in: U256,
    ) -> Result<PriceImpactAnalysis> {
        info!("Analyzing price impact for trade: {} {} -> {} on chain {}",
               amount_in, token_in, token_out, chain_id);

        self.aggregator.analyze_price_impact(
            &self.uniswap,
            &self.sushiswap,
            chain_id,
            token_in,
            token_out,
            amount_in,
        ).await
    }

    /// Batch multiple swaps for gas optimization
    pub async fn batch_optimal_swaps(
        &self,
        chain_id: u64,
        swaps: Vec<(Address, Address, U256)>, // (token_in, token_out, amount_in)
        recipient: Address,
    ) -> Result<Vec<DexOperationResult>> {
        info!("Batching {} swaps for gas optimization on chain {}", swaps.len(), chain_id);

        let transactions = self.aggregator.batch_swaps(
            &self.uniswap,
            &self.sushiswap,
            chain_id,
            swaps.clone(),
            recipient,
        ).await?;

        let mut results = Vec::new();
        for (i, tx) in transactions.into_iter().enumerate() {
            let (token_in, token_out, amount_in) = &swaps[i];
            
            // Get quote for this specific swap to get the details
            let comparison = self.get_comprehensive_quotes(
                chain_id, *token_in, *token_out, *amount_in, recipient
            ).await?;

            results.push(DexOperationResult {
                transaction: tx,
                expected_output: comparison.best_route.output_amount,
                price_impact: comparison.best_route.price_impact,
                gas_estimate: comparison.best_route.gas_estimate,
                dex_used: format!("{:?}", comparison.best_route.dex),
                savings_percentage: comparison.savings_percentage,
            });
        }

        Ok(results)
    }

    /// Add liquidity to the best available pool
    pub async fn add_optimal_liquidity(
        &self,
        chain_id: u64,
        token_a: Address,
        token_b: Address,
        amount_a: U256,
        amount_b: U256,
        recipient: Address,
    ) -> Result<LiquidityResult> {
        info!("Adding optimal liquidity: {} {} + {} {} on chain {}",
               amount_a, token_a, amount_b, token_b, chain_id);

        // Try Uniswap V3 first (generally better for concentrated liquidity)
        match self.uniswap.add_liquidity(
            chain_id, token_a, token_b, 3000, -887220, 887220, // Full range
            amount_a, amount_b, U256::zero(), U256::zero(), recipient, 
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() + 1800
        ).await {
            Ok(uniswap_tx) => {
                let pool_info = self.uniswap.get_pool_info(chain_id, token_a, token_b, 3000).await?;
                
                Ok(LiquidityResult {
                    add_transaction: Some(uniswap_tx),
                    remove_transaction: None,
                    pool_address: pool_info.address,
                    liquidity_amount: U256::zero(), // Would be calculated based on pool response
                    token_amounts: (amount_a, amount_b),
                })
            },
            Err(_) => {
                // Fall back to SushiSwap
                info!("Falling back to SushiSwap for liquidity provision");
                
                let sushiswap_tx = self.sushiswap.add_liquidity(
                    chain_id, token_a, token_b, amount_a, amount_b, U256::zero(), U256::zero(), recipient,
                    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() + 1800
                ).await?;

                let pair_info = self.sushiswap.get_pair_info(chain_id, token_a, token_b).await?;

                Ok(LiquidityResult {
                    add_transaction: Some(sushiswap_tx),
                    remove_transaction: None,
                    pool_address: pair_info.address,
                    liquidity_amount: U256::zero(),
                    token_amounts: (amount_a, amount_b),
                })
            }
        }
    }

    /// Remove liquidity from pools
    pub async fn remove_optimal_liquidity(
        &self,
        chain_id: u64,
        token_a: Address,
        token_b: Address,
        liquidity_amount: U256,
        recipient: Address,
    ) -> Result<LiquidityResult> {
        info!("Removing optimal liquidity: {} from {}/{} pool on chain {}",
               liquidity_amount, token_a, token_b, chain_id);

        // Try to determine which DEX has the position
        // Try Uniswap V3 first
        match self.uniswap.remove_liquidity(
            chain_id, U256::from(1), liquidity_amount, U256::zero(), U256::zero(), // token_id would need to be tracked
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() + 1800
        ).await {
            Ok(uniswap_tx) => {
                let pool_info = self.uniswap.get_pool_info(chain_id, token_a, token_b, 3000).await?;
                
                Ok(LiquidityResult {
                    add_transaction: None,
                    remove_transaction: Some(uniswap_tx),
                    pool_address: pool_info.address,
                    liquidity_amount,
                    token_amounts: (U256::zero(), U256::zero()),
                })
            },
            Err(_) => {
                // Try SushiSwap
                let sushiswap_tx = self.sushiswap.remove_liquidity(
                    chain_id, token_a, token_b, liquidity_amount, U256::zero(), U256::zero(), recipient,
                    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() + 1800
                ).await?;

                let pair_info = self.sushiswap.get_pair_info(chain_id, token_a, token_b).await?;

                Ok(LiquidityResult {
                    add_transaction: None,
                    remove_transaction: Some(sushiswap_tx),
                    pool_address: pair_info.address,
                    liquidity_amount,
                    token_amounts: (U256::zero(), U256::zero()),
                })
            }
        }
    }

    /// Get farming opportunities across all DEXes
    pub async fn get_farming_opportunities(
        &self,
        chain_id: u64,
        user_address: Address,
    ) -> Result<Vec<FarmingOpportunity>> {
        info!("Getting farming opportunities for user {} on chain {}", user_address, chain_id);

        let mut opportunities = Vec::new();

        // Get SushiSwap farming opportunities
        match self.sushiswap.get_all_farms(chain_id).await {
            Ok(farms) => {
                for farm in farms {
                    opportunities.push(FarmingOpportunity {
                        dex: "SushiSwap".to_string(),
                        pool_address: farm.lp_token,
                        token_a: Address::zero(), // Would be extracted from pair info
                        token_b: Address::zero(),
                        apy: farm.apy,
                        total_liquidity: farm.total_staked,
                        reward_token: Address::zero(), // SUSHI token address - would be fetched
                        user_staked: U256::zero(), // Would be queried
                    });
                }
            },
            Err(e) => {
                error!("Failed to get SushiSwap farms: {}", e);
            }
        }

        Ok(opportunities)
    }

    /// Get comprehensive DEX statistics
    pub async fn get_dex_statistics(&self, chain_id: u64) -> Result<DexStats> {
        info!("Getting DEX statistics for chain {}", chain_id);

        // This would be implemented with actual data tracking
        Ok(DexStats {
            total_swaps: 1000,
            total_volume: U256::from(1_000_000),
            average_savings: 0.75, // 0.75% average savings
            best_dex_performance: "UniswapV3".to_string(),
            price_impact_distribution: vec![0.1, 0.2, 0.5, 0.8, 1.2],
        })
    }

    /// Get all available trading pairs across DEXes
    pub async fn get_available_pairs(&self, chain_id: u64) -> Result<Vec<TradingPair>> {
        info!("Getting available trading pairs for chain {}", chain_id);

        let mut pairs = Vec::new();

        // This would query each DEX for their available pairs
        // Implementation would depend on the specific DEX APIs/subgraphs

        Ok(pairs)
    }

    // Utility methods for direct DEX access
    pub fn uniswap(&self) -> &uniswap::UniswapV3Manager {
        &self.uniswap
    }

    pub fn sushiswap(&self) -> &sushiswap::SushiSwapManager {
        &self.sushiswap
    }

    pub fn aggregator(&self) -> &DexAggregator {
        &self.aggregator
    }

    pub fn chain_manager(&self) -> &Arc<ChainManager> {
        &self.chain_manager
    }
}
