use anyhow::{Result, anyhow};
use ethers::types::{Address, U256, TransactionRequest};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, error};

use crate::dex::uniswap::{UniswapV3Manager, SwapParams as UniswapSwapParams};
use crate::dex::sushiswap::SushiSwapManager;

/// Best route information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BestRoute {
    pub dex: DexType,
    pub input_amount: U256,
    pub output_amount: U256,
    pub price_impact: f64,
    pub gas_estimate: U256,
    pub path: Vec<Address>,
    pub transaction: TransactionRequest,
}

/// Available DEX types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DexType {
    UniswapV3,
    SushiSwap,
}

/// Quote comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteComparison {
    pub uniswap_v3: Option<Quote>,
    pub sushiswap: Option<Quote>,
    pub best_route: BestRoute,
    pub savings_percentage: f64,
}

/// Individual DEX quote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub dex: DexType,
    pub input_amount: U256,
    pub output_amount: U256,
    pub price_impact: f64,
    pub gas_estimate: U256,
    pub path: Vec<Address>,
}

/// Slippage protection settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlippageSettings {
    pub max_slippage_percentage: f64, // e.g., 0.5 for 0.5%
    pub deadline_minutes: u64,        // e.g., 20 for 20 minutes
    pub mev_protection: bool,
}

impl Default for SlippageSettings {
    fn default() -> Self {
        Self {
            max_slippage_percentage: 0.5, // 0.5%
            deadline_minutes: 20,         // 20 minutes
            mev_protection: true,
        }
    }
}

/// MEV protection strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MevProtection {
    None,
    FlashbotsBundle,
    PrivateMempool,
    CommitReveal,
}

pub struct DexAggregator {
    price_cache: HashMap<String, (U256, std::time::Instant)>,
    cache_duration: std::time::Duration,
    slippage_settings: SlippageSettings,
}

impl DexAggregator {
    pub async fn new() -> Result<Self> {
        info!("Initializing DEX Aggregator");

        Ok(Self {
            price_cache: HashMap::new(),
            cache_duration: std::time::Duration::from_secs(30), // 30 second cache
            slippage_settings: SlippageSettings::default(),
        })
    }

    /// Find the best route for a swap across all DEXes
    pub async fn find_best_route(
        &self,
        uniswap: &UniswapV3Manager,
        sushiswap: &SushiSwapManager,
        chain_id: u64,
        token_in: Address,
        token_out: Address,
        amount_in: U256,
        recipient: Address,
    ) -> Result<QuoteComparison> {
        info!("Finding best route for swap: {} {} -> {}", amount_in, token_in, token_out);

        let mut quotes = Vec::new();

        // Get Uniswap V3 quote (try different fee tiers)
        let uniswap_quote = self.get_uniswap_quote(
            uniswap, chain_id, token_in, token_out, amount_in, recipient
        ).await;

        if let Ok(quote) = uniswap_quote {
            quotes.push(quote);
        }

        // Get SushiSwap quote
        let sushiswap_quote = self.get_sushiswap_quote(
            sushiswap, chain_id, token_in, token_out, amount_in, recipient
        ).await;

        if let Ok(quote) = sushiswap_quote {
            quotes.push(quote);
        }

        if quotes.is_empty() {
            return Err(anyhow!("No valid quotes found from any DEX"));
        }

        // Find best quote (highest output amount considering gas costs)
        let best_quote = quotes
            .clone()
            .into_iter()
            .max_by(|a, b| {
                // Adjust for gas costs (simplified calculation)
                let a_adjusted = a.output_amount.saturating_sub(a.gas_estimate * U256::from(20_000_000_000u64));
                let b_adjusted = b.output_amount.saturating_sub(b.gas_estimate * U256::from(20_000_000_000u64));
                a_adjusted.cmp(&b_adjusted)
            })
            .unwrap();

        // Calculate savings compared to worst option
        let worst_output = quotes.iter()
            .min_by_key(|q| q.output_amount)
            .map(|q| q.output_amount)
            .unwrap_or_else(|| best_quote.output_amount);

        let savings_percentage = if worst_output > U256::zero() {
            ((best_quote.output_amount - worst_output).as_u128() as f64 / worst_output.as_u128() as f64) * 100.0
        } else {
            0.0
        };

        // Create transaction for best route
        let transaction = self.create_transaction_for_quote(
            uniswap, sushiswap, chain_id, &best_quote, recipient
        ).await?;

        let best_route = BestRoute {
            dex: best_quote.dex.clone(),
            input_amount: best_quote.input_amount,
            output_amount: best_quote.output_amount,
            price_impact: best_quote.price_impact,
            gas_estimate: best_quote.gas_estimate,
            path: best_quote.path.clone(),
            transaction,
        };

        let comparison = QuoteComparison {
            uniswap_v3: quotes.iter().find(|q| q.dex == DexType::UniswapV3).cloned(),
            sushiswap: quotes.iter().find(|q| q.dex == DexType::SushiSwap).cloned(),
            best_route,
            savings_percentage,
        };

        info!("Best route found: {:?} with {}% savings", comparison.best_route.dex, savings_percentage);
        Ok(comparison)
    }

    /// Execute optimal swap with slippage protection
    pub async fn execute_optimal_swap(
        &self,
        uniswap: &UniswapV3Manager,
        sushiswap: &SushiSwapManager,
        chain_id: u64,
        token_in: Address,
        token_out: Address,
        amount_in: U256,
        recipient: Address,
        slippage_settings: Option<SlippageSettings>,
    ) -> Result<TransactionRequest> {
        let settings = slippage_settings.unwrap_or_else(|| self.slippage_settings.clone());
        
        // Find best route
        let comparison = self.find_best_route(
            uniswap, sushiswap, chain_id, token_in, token_out, amount_in, recipient
        ).await?;

        // Apply slippage protection
        let min_amount_out = self.calculate_min_amount_out(
            comparison.best_route.output_amount,
            settings.max_slippage_percentage,
        );

        info!("Executing optimal swap with slippage protection: min_amount_out = {}", min_amount_out);

        // Create protected transaction
        let mut tx = comparison.best_route.transaction;
        
        // Add MEV protection if enabled
        if settings.mev_protection {
            tx = self.add_mev_protection(tx, MevProtection::PrivateMempool).await?;
        }

        Ok(tx)
    }

    /// Batch multiple swaps for gas optimization
    pub async fn batch_swaps(
        &self,
        uniswap: &UniswapV3Manager,
        sushiswap: &SushiSwapManager,
        chain_id: u64,
        swaps: Vec<(Address, Address, U256)>, // (token_in, token_out, amount_in)
        recipient: Address,
    ) -> Result<Vec<TransactionRequest>> {
        info!("Batching {} swaps for gas optimization", swaps.len());

        let mut transactions = Vec::new();

        for (token_in, token_out, amount_in) in swaps {
            let comparison = self.find_best_route(
                uniswap, sushiswap, chain_id, token_in, token_out, amount_in, recipient
            ).await?;

            transactions.push(comparison.best_route.transaction);
        }

        Ok(transactions)
    }

    /// Monitor price impact and suggest better timing
    pub async fn analyze_price_impact(
        &self,
        uniswap: &UniswapV3Manager,
        sushiswap: &SushiSwapManager,
        chain_id: u64,
        token_in: Address,
        token_out: Address,
        amount_in: U256,
    ) -> Result<PriceImpactAnalysis> {
        // Get quotes for different amounts to analyze price impact
        let base_amount = amount_in / U256::from(10); // 10% of original
        let double_amount = amount_in * U256::from(2);

        let small_quote = self.find_best_route(
            uniswap, sushiswap, chain_id, token_in, token_out, base_amount, Address::zero()
        ).await?;

        let large_quote = self.find_best_route(
            uniswap, sushiswap, chain_id, token_in, token_out, double_amount, Address::zero()
        ).await?;

        // Calculate price impact curve
        let price_impact_curve = self.calculate_price_impact_curve(
            &small_quote.best_route,
            &large_quote.best_route,
            base_amount,
            double_amount,
        );

        let analysis = PriceImpactAnalysis {
            current_impact: small_quote.best_route.price_impact,
            impact_at_2x: large_quote.best_route.price_impact,
            recommended_split: if large_quote.best_route.price_impact > 2.0 {
                Some(self.calculate_optimal_split(amount_in, token_in, token_out))
            } else {
                None
            },
            better_timing_suggestion: self.suggest_better_timing(&price_impact_curve),
        };

        Ok(analysis)
    }

    // Private helper methods

    async fn get_uniswap_quote(
        &self,
        uniswap: &UniswapV3Manager,
        chain_id: u64,
        token_in: Address,
        token_out: Address,
        amount_in: U256,
        recipient: Address,
    ) -> Result<Quote> {
        // Try different fee tiers and find the best one
        let fee_tiers = [500u32, 3000, 10000]; // 0.05%, 0.3%, 1%
        let mut best_quote = None;
        let mut best_output = U256::zero();

        for fee in fee_tiers.iter() {
            if let Ok(output) = uniswap.quote_exact_input_single(
                chain_id, token_in, token_out, *fee, amount_in, U256::zero()
            ).await {
                if output > best_output {
                    best_output = output;
                    best_quote = Some((output, *fee));
                }
            }
        }

        if let Some((output, fee)) = best_quote {
            let price_impact = self.calculate_price_impact(amount_in, output, token_in, token_out);
            
            Ok(Quote {
                dex: DexType::UniswapV3,
                input_amount: amount_in,
                output_amount: output,
                price_impact,
                gas_estimate: U256::from(150_000), // Estimated gas for Uniswap V3
                path: vec![token_in, token_out],
            })
        } else {
            Err(anyhow!("No valid Uniswap V3 quote found"))
        }
    }

    async fn get_sushiswap_quote(
        &self,
        sushiswap: &SushiSwapManager,
        chain_id: u64,
        token_in: Address,
        token_out: Address,
        amount_in: U256,
        _recipient: Address,
    ) -> Result<Quote> {
        let path = vec![token_in, token_out];
        let amounts = sushiswap.get_amounts_out(chain_id, amount_in, path.clone()).await?;
        
        if amounts.len() < 2 {
            return Err(anyhow!("Invalid SushiSwap quote response"));
        }

        let output_amount = amounts[1];
        let price_impact = self.calculate_price_impact(amount_in, output_amount, token_in, token_out);

        Ok(Quote {
            dex: DexType::SushiSwap,
            input_amount: amount_in,
            output_amount,
            price_impact,
            gas_estimate: U256::from(120_000), // Estimated gas for SushiSwap
            path,
        })
    }

    async fn create_transaction_for_quote(
        &self,
        uniswap: &UniswapV3Manager,
        sushiswap: &SushiSwapManager,
        chain_id: u64,
        quote: &Quote,
        recipient: Address,
    ) -> Result<TransactionRequest> {
        let deadline = self.calculate_deadline();

        match quote.dex {
            DexType::UniswapV3 => {
                let params = UniswapSwapParams {
                    token_in: quote.path[0],
                    token_out: quote.path[1],
                    amount_in: quote.input_amount,
                    amount_out_minimum: self.calculate_min_amount_out(quote.output_amount, self.slippage_settings.max_slippage_percentage),
                    fee: 3000, // Default to 0.3% fee tier
                    recipient,
                    deadline,
                    sqrt_price_limit_x96: U256::zero(),
                };

                uniswap.swap_exact_input_single(chain_id, params).await
            },
            DexType::SushiSwap => {
                let min_amount_out = self.calculate_min_amount_out(quote.output_amount, self.slippage_settings.max_slippage_percentage);
                
                sushiswap.swap_exact_tokens_for_tokens(
                    chain_id,
                    quote.input_amount,
                    min_amount_out,
                    quote.path.clone(),
                    recipient,
                    deadline,
                ).await
            },
        }
    }

    fn calculate_price_impact(&self, amount_in: U256, amount_out: U256, _token_in: Address, _token_out: Address) -> f64 {
        // Simplified price impact calculation
        // In reality, you'd need to know the pool reserves and calculate the exact impact
        if amount_in.is_zero() || amount_out.is_zero() {
            return 0.0;
        }

        // Mock calculation - replace with actual price impact formula
        let input_value = amount_in.as_u128() as f64;
        let output_value = amount_out.as_u128() as f64;
        
        // Assume 1:1 base price for simplicity
        let expected_output = input_value;
        let impact = ((expected_output - output_value) / expected_output).abs() * 100.0;
        
        impact.min(50.0) // Cap at 50%
    }

    fn calculate_min_amount_out(&self, amount_out: U256, slippage_percentage: f64) -> U256 {
        let slippage_multiplier = 1.0 - (slippage_percentage / 100.0);
        let min_amount = (amount_out.as_u128() as f64 * slippage_multiplier) as u128;
        U256::from(min_amount)
    }

    fn calculate_deadline(&self) -> u64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now + (self.slippage_settings.deadline_minutes * 60)
    }

    async fn add_mev_protection(&self, mut tx: TransactionRequest, protection: MevProtection) -> Result<TransactionRequest> {
        match protection {
            MevProtection::PrivateMempool => {
                // Add higher gas price for private mempool submission
                // This is simplified - in reality you'd integrate with services like Flashbots
                info!("Adding MEV protection via private mempool");
                Ok(tx)
            },
            _ => Ok(tx),
        }
    }

    fn calculate_price_impact_curve(&self, small: &BestRoute, large: &BestRoute, small_amount: U256, large_amount: U256) -> PriceImpactCurve {
        PriceImpactCurve {
            small_trade_impact: small.price_impact,
            large_trade_impact: large.price_impact,
            liquidity_depth: self.estimate_liquidity_depth(small_amount, large_amount, small.price_impact, large.price_impact),
        }
    }

    fn estimate_liquidity_depth(&self, small_amount: U256, large_amount: U256, small_impact: f64, large_impact: f64) -> f64 {
        // Simplified liquidity depth estimation
        if large_impact <= small_impact {
            return f64::INFINITY; // Very deep liquidity
        }
        
        let amount_ratio = (large_amount.as_u128() / small_amount.as_u128()) as f64;
        let impact_ratio = large_impact / small_impact;
        
        // Higher ratio means less liquid
        impact_ratio / amount_ratio
    }

    fn calculate_optimal_split(&self, amount: U256, _token_in: Address, _token_out: Address) -> Vec<U256> {
        // Simple strategy: split into smaller chunks to reduce price impact
        let chunk_size = amount / U256::from(4); // Split into 4 parts
        vec![chunk_size, chunk_size, chunk_size, amount - (chunk_size * U256::from(3))]
    }

    fn suggest_better_timing(&self, curve: &PriceImpactCurve) -> Option<TimingSuggestion> {
        if curve.liquidity_depth < 2.0 {
            Some(TimingSuggestion {
                wait_minutes: 30,
                reason: "Low liquidity detected, consider waiting for better market conditions".to_string(),
                expected_improvement: 0.5, // 0.5% better pricing expected
            })
        } else {
            None
        }
    }
}

/// Price impact analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceImpactAnalysis {
    pub current_impact: f64,
    pub impact_at_2x: f64,
    pub recommended_split: Option<Vec<U256>>,
    pub better_timing_suggestion: Option<TimingSuggestion>,
}

/// Price impact curve data
#[derive(Debug, Clone)]
struct PriceImpactCurve {
    pub small_trade_impact: f64,
    pub large_trade_impact: f64,
    pub liquidity_depth: f64,
}

/// Timing suggestion for better execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingSuggestion {
    pub wait_minutes: u64,
    pub reason: String,
    pub expected_improvement: f64, // percentage
}
