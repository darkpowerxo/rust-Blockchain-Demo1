use std::sync::Arc;
use crate::chains::ChainManager;
use crate::dex::DexManager;
use anyhow::Result;
use ethers::types::{Address, U256, TransactionRequest};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

pub mod aave;
pub mod compound;
pub mod flash_loans;

use aave::{AaveManager, LendingPosition as AaveLendingPosition, YieldStrategy as AaveYieldStrategy};
use compound::{CompoundManager, UserCompoundData, CompoundYieldStrategy, LiquidationOpportunity, CompArbitrageOpportunity};
use flash_loans::{FlashLoanManager, FlashLoanStrategy, ArbitrageStrategy};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefiPortfolio {
    pub user: Address,
    pub total_supplied_usd: f64,
    pub total_borrowed_usd: f64,
    pub net_worth_usd: f64,
    pub overall_health_factor: f64,
    pub aave_positions: Vec<AaveLendingPosition>,
    pub compound_positions: Vec<compound::UserCTokenPosition>,
    pub active_strategies: Vec<ActiveStrategy>,
    pub yield_earned_24h: f64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveStrategy {
    pub strategy_id: String,
    pub protocol: String, // "aave", "compound", "cross-protocol"
    pub strategy_type: String,
    pub invested_amount: U256,
    pub current_value: U256,
    pub apy: f64,
    pub risk_level: String,
    pub start_date: DateTime<Utc>,
    pub profit_loss: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimalYieldOpportunity {
    pub strategy_type: String,
    pub protocol: String,
    pub estimated_apy: f64,
    pub risk_level: String,
    pub min_deposit: U256,
    pub max_deposit: U256,
    pub liquidity_risk: f64,
    pub impermanent_loss_risk: f64,
    pub smart_contract_risk: f64,
    pub description: String,
    pub steps: Vec<YieldOpportunityStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum YieldOpportunityStep {
    Supply { protocol: String, asset: Address, amount: U256 },
    Borrow { protocol: String, asset: Address, amount: U256 },
    Swap { dex: String, token_in: Address, token_out: Address, amount: U256 },
    Farm { protocol: String, pool: Address, amount: U256 },
    Stake { protocol: String, token: Address, amount: U256 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossProtocolArbitrage {
    pub arbitrage_type: String,
    pub profit_estimate: U256,
    pub required_capital: U256,
    pub success_probability: f64,
    pub gas_cost_estimate: U256,
    pub net_profit_estimate: U256,
    pub execution_time_minutes: u32,
    pub protocols_involved: Vec<String>,
    pub operations: Vec<ArbitrageOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArbitrageOperation {
    FlashLoan { protocol: String, asset: Address, amount: U256 },
    Supply { protocol: String, asset: Address, amount: U256 },
    Borrow { protocol: String, asset: Address, amount: U256 },
    Swap { dex: String, token_in: Address, token_out: Address, amount_in: U256 },
    Liquidate { protocol: String, borrower: Address, asset: Address, amount: U256 },
    Repay { protocol: String, asset: Address, amount: U256 },
}

pub struct DefiManager {
    chain_manager: Arc<ChainManager>,
    dex_manager: Arc<DexManager>,
    aave: aave::AaveManager,
    compound: compound::CompoundManager,
    flash_loans: flash_loans::FlashLoanManager,
}

impl DefiManager {
    pub async fn new(chain_manager: Arc<ChainManager>, dex_manager: Arc<DexManager>) -> Result<Self> {
        let aave = AaveManager::new(chain_manager.clone(), dex_manager.clone()).await?;
        let compound = CompoundManager::new(chain_manager.clone(), dex_manager.clone()).await?;
        let flash_loans = FlashLoanManager::new(chain_manager.clone(), dex_manager.clone()).await?;

        Ok(Self {
            chain_manager,
            dex_manager,
            aave,
            compound,
            flash_loans,
        })
    }

    /// Get comprehensive DeFi portfolio overview for a user
    pub async fn get_portfolio_overview(&self, chain_id: u64, user: Address) -> Result<DefiPortfolio> {
        // Get Aave positions
        let aave_positions = self.aave.get_lending_position(chain_id, user).await?;
        
        // Get Compound positions
        let compound_data = self.compound.get_user_compound_data(chain_id, user).await?;
        
        // Calculate totals
        let mut total_supplied_usd = 0.0;
        let mut total_borrowed_usd = 0.0;
        
        for position in &aave_positions {
            total_supplied_usd += (position.supplied_amount.as_u128() as f64) / 1e18;
            total_borrowed_usd += (position.borrowed_amount_variable.as_u128() as f64) / 1e18;
        }
        
        for position in &compound_data.positions {
            total_supplied_usd += (position.supply_balance.as_u128() as f64) / 1e18;
            total_borrowed_usd += (position.borrow_balance.as_u128() as f64) / 1e18;
        }

        let net_worth_usd = total_supplied_usd - total_borrowed_usd;
        
        // Calculate overall health factor (weighted average)
        let aave_health = if !aave_positions.is_empty() {
            aave_positions[0].health_factor.as_u128() as f64 / 1e18
        } else { f64::INFINITY };
        
        let compound_health = compound_data.health_factor;
        let overall_health_factor = (aave_health + compound_health) / 2.0;

        Ok(DefiPortfolio {
            user,
            total_supplied_usd,
            total_borrowed_usd,
            net_worth_usd,
            overall_health_factor,
            aave_positions,
            compound_positions: compound_data.positions,
            active_strategies: Vec::new(), // Would be populated from strategy tracking
            yield_earned_24h: 150.75, // Mock value
            last_updated: chrono::Utc::now(),
        })
    }

    /// Find optimal yield opportunities across all protocols
    pub async fn find_optimal_yield_opportunities(&self, chain_id: u64, asset: Address, amount: U256) -> Result<Vec<OptimalYieldOpportunity>> {
        let mut opportunities = Vec::new();

        // Get Aave strategies
        let aave_strategies = self.aave.get_yield_strategies(chain_id, asset, amount).await?;
        for strategy in aave_strategies {
            opportunities.push(OptimalYieldOpportunity {
                strategy_type: strategy.name.clone(),
                protocol: "Aave".to_string(),
                estimated_apy: strategy.estimated_apy,
                risk_level: format!("{:?}", strategy.risk_level),
                min_deposit: strategy.min_deposit,
                max_deposit: amount * U256::from(10), // 10x leverage max
                liquidity_risk: match strategy.risk_level {
                    aave::RiskLevel::Low => 0.1,
                    aave::RiskLevel::Medium => 0.3,
                    aave::RiskLevel::High => 0.6,
                    aave::RiskLevel::VeryHigh => 0.9,
                },
                impermanent_loss_risk: 0.0, // No IL risk for lending
                smart_contract_risk: 0.15, // Aave has good security record
                description: strategy.description,
                steps: strategy.steps.into_iter().map(|step| match step {
                    aave::YieldStep::Supply { asset, .. } => YieldOpportunityStep::Supply { 
                        protocol: "Aave".to_string(), 
                        asset, 
                        amount 
                    },
                    aave::YieldStep::Borrow { asset, .. } => YieldOpportunityStep::Borrow { 
                        protocol: "Aave".to_string(), 
                        asset, 
                        amount 
                    },
                    aave::YieldStep::Swap { token_in, token_out, .. } => YieldOpportunityStep::Swap { 
                        dex: "Uniswap".to_string(), 
                        token_in, 
                        token_out, 
                        amount 
                    },
                    aave::YieldStep::Farm { pool_address, .. } => YieldOpportunityStep::Farm { 
                        protocol: "SushiSwap".to_string(), 
                        pool: pool_address, 
                        amount 
                    },
                }).collect(),
            });
        }

        // Get Compound strategies
        let compound_strategies = self.compound.get_yield_strategies(chain_id, asset, amount).await?;
        for strategy in compound_strategies {
            opportunities.push(OptimalYieldOpportunity {
                strategy_type: strategy.name.clone(),
                protocol: "Compound".to_string(),
                estimated_apy: strategy.estimated_apy,
                risk_level: format!("{:?}", strategy.risk_level),
                min_deposit: strategy.min_deposit,
                max_deposit: amount * U256::from(5), // 5x leverage max for Compound
                liquidity_risk: match strategy.risk_level {
                    compound::RiskLevel::Low => 0.05,
                    compound::RiskLevel::Medium => 0.25,
                    compound::RiskLevel::High => 0.55,
                    compound::RiskLevel::VeryHigh => 0.85,
                },
                impermanent_loss_risk: 0.0,
                smart_contract_risk: 0.1, // Compound also has good security
                description: strategy.description,
                steps: Vec::new(), // Would convert from compound steps
            });
        }

        // Add cross-protocol strategies
        opportunities.push(self.create_cross_protocol_strategy(chain_id, asset, amount).await?);

        // Sort by estimated APY descending
        opportunities.sort_by(|a, b| b.estimated_apy.partial_cmp(&a.estimated_apy).unwrap());

        Ok(opportunities)
    }

    /// Execute optimal yield strategy automatically
    pub async fn execute_optimal_yield_strategy(&self, chain_id: u64, strategy: OptimalYieldOpportunity, user: Address) -> Result<Vec<TransactionRequest>> {
        let mut transactions = Vec::new();

        for step in &strategy.steps {
            match step {
                YieldOpportunityStep::Supply { protocol, asset, amount } => {
                    let tx = match protocol.as_str() {
                        "Aave" => self.aave.supply(chain_id, *asset, *amount, user, 0).await?,
                        "Compound" => {
                            // Find appropriate cToken for asset
                            let ctoken = self.find_ctoken_for_asset(chain_id, *asset).await?;
                            self.compound.supply(chain_id, ctoken, *amount).await?
                        },
                        _ => return Err(anyhow::anyhow!("Unsupported protocol: {}", protocol)),
                    };
                    transactions.push(tx);
                },
                YieldOpportunityStep::Borrow { protocol, asset, amount } => {
                    let tx = match protocol.as_str() {
                        "Aave" => self.aave.borrow(chain_id, *asset, *amount, 2, 0, user).await?,
                        "Compound" => {
                            let ctoken = self.find_ctoken_for_asset(chain_id, *asset).await?;
                            self.compound.borrow(chain_id, ctoken, *amount).await?
                        },
                        _ => return Err(anyhow::anyhow!("Unsupported protocol: {}", protocol)),
                    };
                    transactions.push(tx);
                },
                YieldOpportunityStep::Swap { token_in, token_out, amount, .. } => {
                    // Use DEX manager for optimal swapping
                    let swap_result = self.dex_manager.execute_optimal_swap(
                        chain_id,
                        *token_in,
                        *token_out,
                        *amount,
                        Address::zero(), // Default recipient (will be set by DEX manager)
                        None, // Use default slippage settings
                    ).await?;
                    transactions.push(swap_result.transaction);
                },
                YieldOpportunityStep::Farm { protocol, pool, amount } => {
                    // Add liquidity to farming pool
                    if protocol == "SushiSwap" {
                        // Would integrate with SushiSwap farming
                        println!("Adding {} to SushiSwap farm at pool {}", amount, pool);
                    }
                },
                YieldOpportunityStep::Stake { protocol, token, amount } => {
                    // Handle staking operations
                    println!("Staking {} of token {} on {}", amount, token, protocol);
                },
            }
        }

        Ok(transactions)
    }

    /// Find cross-protocol arbitrage opportunities
    pub async fn find_cross_protocol_arbitrage(&self, chain_id: u64) -> Result<Vec<CrossProtocolArbitrage>> {
        let mut opportunities = Vec::new();

        // Rate arbitrage between Aave and Compound
        let aave_rates = self.get_aave_rates(chain_id).await?;
        let compound_rates = self.compound.get_all_borrow_rates(chain_id).await?;

        for (aave_asset, aave_supply_rate) in aave_rates {
            for (compound_ctoken, compound_borrow_rate) in &compound_rates {
                if aave_supply_rate > *compound_borrow_rate {
                    let profit_rate = aave_supply_rate - compound_borrow_rate;
                    let required_capital = U256::from(100000u64); // $100k
                    let annual_profit = required_capital * profit_rate / U256::from(1e18 as u64);
                    
                    opportunities.push(CrossProtocolArbitrage {
                        arbitrage_type: "Rate Arbitrage".to_string(),
                        profit_estimate: annual_profit / U256::from(365), // Daily profit
                        required_capital,
                        success_probability: 0.9,
                        gas_cost_estimate: U256::from(500000u64),
                        net_profit_estimate: (annual_profit / U256::from(365)) - U256::from(500000u64),
                        execution_time_minutes: 15,
                        protocols_involved: vec!["Compound".to_string(), "Aave".to_string()],
                        operations: vec![
                            ArbitrageOperation::Borrow { 
                                protocol: "Compound".to_string(), 
                                asset: aave_asset, 
                                amount: required_capital 
                            },
                            ArbitrageOperation::Supply { 
                                protocol: "Aave".to_string(), 
                                asset: aave_asset, 
                                amount: required_capital 
                            },
                        ],
                    });
                }
            }
        }

        // Liquidation arbitrage opportunities
        let compound_liquidations = self.compound.find_liquidation_opportunities(chain_id).await?;
        for liq in compound_liquidations {
            opportunities.push(CrossProtocolArbitrage {
                arbitrage_type: "Liquidation Arbitrage".to_string(),
                profit_estimate: liq.profit_estimate,
                required_capital: liq.repay_amount,
                success_probability: 0.95,
                gas_cost_estimate: U256::from(300000u64),
                net_profit_estimate: liq.profit_estimate - U256::from(300000u64),
                execution_time_minutes: 5,
                protocols_involved: vec!["Compound".to_string()],
                operations: vec![
                    ArbitrageOperation::FlashLoan { 
                        protocol: "Aave".to_string(), 
                        asset: liq.ctoken_borrowed, 
                        amount: liq.repay_amount 
                    },
                    ArbitrageOperation::Liquidate { 
                        protocol: "Compound".to_string(), 
                        borrower: liq.account, 
                        asset: liq.ctoken_borrowed, 
                        amount: liq.repay_amount 
                    },
                ],
            });
        }

        // Sort by profit potential
        opportunities.sort_by(|a, b| b.net_profit_estimate.cmp(&a.net_profit_estimate));

        Ok(opportunities)
    }

    /// Execute flash loan strategy across protocols
    pub async fn execute_flash_loan_arbitrage(&self, chain_id: u64, arbitrage: CrossProtocolArbitrage) -> Result<Vec<TransactionRequest>> {
        let mut transactions = Vec::new();

        // Create flash loan strategy from arbitrage operations
        let flash_loan_strategy = FlashLoanStrategy {
            strategy_name: arbitrage.arbitrage_type.clone(),
            description: format!("Cross-protocol arbitrage involving: {:?}", arbitrage.protocols_involved),
            target_profit: arbitrage.profit_estimate,
            max_gas_fee: arbitrage.gas_cost_estimate,
            operations: arbitrage.operations.into_iter().map(|op| match op {
                ArbitrageOperation::Supply { asset, amount, .. } => 
                    flash_loans::FlashLoanOperation::Supply { 
                        protocol: "aave".to_string(), 
                        asset, 
                        amount 
                    },
                ArbitrageOperation::Borrow { asset, amount, .. } => 
                    flash_loans::FlashLoanOperation::Borrow { 
                        protocol: "compound".to_string(), 
                        asset, 
                        amount, 
                        interest_rate_mode: 2 
                    },
                ArbitrageOperation::Swap { token_in, token_out, amount_in, .. } => 
                    flash_loans::FlashLoanOperation::Swap { 
                        dex: "uniswap".to_string(), 
                        token_in, 
                        token_out, 
                        amount_in, 
                        min_amount_out: amount_in * U256::from(95) / U256::from(100) 
                    },
                _ => flash_loans::FlashLoanOperation::Supply { 
                    protocol: "aave".to_string(), 
                    asset: Address::zero(), 
                    amount: U256::zero() 
                },
            }).collect(),
        };

        // Execute flash loan strategy
        let flash_loan_txs = self.flash_loans.execute_flash_loan_strategy(chain_id, flash_loan_strategy).await?;
        transactions.extend(flash_loan_txs);

        Ok(transactions)
    }

    /// Rebalance portfolio to optimize yield
    pub async fn rebalance_portfolio(&self, chain_id: u64, user: Address, target_allocation: std::collections::HashMap<String, f64>) -> Result<Vec<TransactionRequest>> {
        let mut transactions = Vec::new();
        
        let portfolio = self.get_portfolio_overview(chain_id, user).await?;
        
        // Calculate current allocation
        let total_value = portfolio.total_supplied_usd;
        
        for (protocol, target_percentage) in target_allocation {
            let target_value = total_value * target_percentage;
            let current_value = match protocol.as_str() {
                "aave" => portfolio.aave_positions.iter().map(|p| (p.supplied_amount.as_u128() as f64) / 1e18).sum::<f64>(),
                "compound" => portfolio.compound_positions.iter().map(|p| (p.supply_balance.as_u128() as f64) / 1e18).sum::<f64>(),
                _ => 0.0,
            };
            
            let difference = target_value - current_value;
            
            if difference.abs() > total_value * 0.05 { // 5% threshold
                if difference > 0.0 {
                    // Need to allocate more to this protocol
                    let amount = U256::from((difference * 1e18) as u64);
                    let asset = Address::zero(); // Would determine based on strategy
                    
                    match protocol.as_str() {
                        "aave" => {
                            let tx = self.aave.supply(chain_id, asset, amount, user, 0).await?;
                            transactions.push(tx);
                        },
                        "compound" => {
                            let ctoken = self.find_ctoken_for_asset(chain_id, asset).await?;
                            let tx = self.compound.supply(chain_id, ctoken, amount).await?;
                            transactions.push(tx);
                        },
                        _ => {}
                    }
                } else {
                    // Need to withdraw from this protocol
                    let amount = U256::from((difference.abs() * 1e18) as u64);
                    let asset = Address::zero(); // Would determine based on strategy
                    
                    match protocol.as_str() {
                        "aave" => {
                            let tx = self.aave.withdraw(chain_id, asset, amount, user).await?;
                            transactions.push(tx);
                        },
                        "compound" => {
                            let ctoken = self.find_ctoken_for_asset(chain_id, asset).await?;
                            let tx = self.compound.redeem_underlying(chain_id, ctoken, amount).await?;
                            transactions.push(tx);
                        },
                        _ => {}
                    }
                }
            }
        }
        
        Ok(transactions)
    }

    /// Monitor and alert for liquidation risks
    pub async fn monitor_liquidation_risks(&self, chain_id: u64, user: Address) -> Result<Vec<String>> {
        let mut alerts = Vec::new();
        
        let portfolio = self.get_portfolio_overview(chain_id, user).await?;
        
        // Check Aave health factors
        for position in &portfolio.aave_positions {
            let health_factor = (position.health_factor.as_u128() as f64) / 1e18;
            if health_factor < 1.5 {
                alerts.push(format!(
                    "⚠️ Aave position for {} at risk! Health factor: {:.2}",
                    format!("{:?}", position.asset)[2..8].to_uppercase(),
                    health_factor
                ));
            }
        }
        
        // Check Compound health factor
        if portfolio.overall_health_factor < 1.3 {
            alerts.push(format!(
                "⚠️ Compound positions at risk! Health factor: {:.2}",
                portfolio.overall_health_factor
            ));
        }
        
        // Check for high borrowing ratios
        if portfolio.total_borrowed_usd / portfolio.total_supplied_usd > 0.8 {
            alerts.push("⚠️ High borrowing ratio detected! Consider reducing leverage.".to_string());
        }
        
        Ok(alerts)
    }

    // Helper methods
    async fn create_cross_protocol_strategy(&self, chain_id: u64, asset: Address, amount: U256) -> Result<OptimalYieldOpportunity> {
        Ok(OptimalYieldOpportunity {
            strategy_type: "Cross-Protocol Yield Maximization".to_string(),
            protocol: "Aave + Compound".to_string(),
            estimated_apy: 18.5,
            risk_level: "High".to_string(),
            min_deposit: U256::from(50000u64),
            max_deposit: amount * U256::from(3),
            liquidity_risk: 0.4,
            impermanent_loss_risk: 0.0,
            smart_contract_risk: 0.25,
            description: "Supply on Aave, borrow stablecoin, supply on Compound for rate arbitrage".to_string(),
            steps: vec![
                YieldOpportunityStep::Supply { protocol: "Aave".to_string(), asset, amount },
                YieldOpportunityStep::Borrow { 
                    protocol: "Aave".to_string(), 
                    asset: "0xA0b86a33E6441E5A3D3CdeC19A4F6BbBc2A906b4".parse()?, // USDC
                    amount: amount * U256::from(75) / U256::from(100) 
                },
                YieldOpportunityStep::Supply { 
                    protocol: "Compound".to_string(), 
                    asset: "0xA0b86a33E6441E5A3D3CdeC19A4F6BbBc2A906b4".parse()?, 
                    amount: amount * U256::from(75) / U256::from(100) 
                },
            ],
        })
    }

    async fn get_aave_rates(&self, chain_id: u64) -> Result<Vec<(Address, U256)>> {
        // Mock implementation - would get actual rates from Aave
        Ok(vec![
            ("0xA0b86a33E6441E5A3D3CdeC19A4F6BbBc2A906b4".parse()?, U256::from(35000000000000000u64)), // 3.5%
            ("0x2170Ed0880ac9A755fd29B2688956BD959F933F8".parse()?, U256::from(25000000000000000u64)), // 2.5%
        ])
    }

    async fn find_ctoken_for_asset(&self, chain_id: u64, asset: Address) -> Result<Address> {
        // Mock implementation - would have proper asset to cToken mapping
        Ok("0x5d3a536E4D6DbD6114cc1Ead35777bAB948E3643".parse()?) // cDAI
    }

    pub fn aave(&self) -> &AaveManager {
        &self.aave
    }

    pub fn compound(&self) -> &CompoundManager {
        &self.compound
    }

    pub fn flash_loans(&self) -> &FlashLoanManager {
        &self.flash_loans
    }

    pub fn dex_manager(&self) -> &Arc<DexManager> {
        &self.dex_manager
    }
}
