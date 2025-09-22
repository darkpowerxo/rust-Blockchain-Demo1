use std::{sync::Arc, collections::HashMap};
use ethers::types::{Address, U256, H256, TransactionRequest};
use ethers::abi::{Abi, Token, ParamType, AbiEncode};
use ethers::contract::Contract;
use crate::chains::ChainManager;
use crate::dex::DexManager;
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompoundContracts {
    pub comptroller: Address,
    pub price_oracle: Address,
    pub comp_token: Address,
    pub ceth: Address,
    pub cdai: Address,
    pub cusdc: Address,
    pub cwbtc: Address,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CTokenInfo {
    pub symbol: String,
    pub underlying_address: Address,
    pub ctoken_address: Address,
    pub decimals: u8,
    pub exchange_rate: U256,
    pub supply_rate_per_block: U256,
    pub borrow_rate_per_block: U256,
    pub total_supply: U256,
    pub total_borrows: U256,
    pub total_reserves: U256,
    pub cash: U256,
    pub collateral_factor: U256,
    pub liquidation_incentive: U256,
    pub reserve_factor: U256,
    pub comp_speed_supply: U256,
    pub comp_speed_borrow: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCompoundData {
    pub account: Address,
    pub total_supply_value: U256,
    pub total_borrow_value: U256,
    pub account_liquidity: U256,
    pub shortfall: U256,
    pub health_factor: f64,
    pub positions: Vec<UserCTokenPosition>,
    pub comp_accrued: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCTokenPosition {
    pub ctoken: Address,
    pub underlying_symbol: String,
    pub supply_balance: U256,
    pub borrow_balance: U256,
    pub supply_apy: f64,
    pub borrow_apy: f64,
    pub comp_apy_supply: f64,
    pub comp_apy_borrow: f64,
    pub collateral_factor: U256,
    pub is_collateral: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompoundYieldStrategy {
    pub strategy_id: String,
    pub name: String,
    pub description: String,
    pub estimated_apy: f64,
    pub risk_level: RiskLevel,
    pub min_deposit: U256,
    pub assets_involved: Vec<Address>,
    pub steps: Vec<CompoundStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompoundStep {
    Supply { ctoken: Address, amount_ratio: f64 },
    Borrow { ctoken: Address, amount_ratio: f64 },
    EnterMarkets { ctokens: Vec<Address> },
    ClaimComp { account: Address },
    SwapCompForAsset { asset: Address },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    VeryHigh,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidationOpportunity {
    pub account: Address,
    pub ctoken_borrowed: Address,
    pub ctoken_collateral: Address,
    pub repay_amount: U256,
    pub seize_amount: U256,
    pub profit_estimate: U256,
    pub health_factor: f64,
    pub liquidation_incentive: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompArbitrageOpportunity {
    pub strategy_type: String,
    pub profit_estimate: U256,
    pub gas_estimate: U256,
    pub net_profit: U256,
    pub required_capital: U256,
    pub success_probability: f64,
    pub operations: Vec<ArbitrageOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArbitrageOperation {
    SupplyCompound { ctoken: Address, amount: U256 },
    BorrowCompound { ctoken: Address, amount: U256 },
    SwapDex { token_in: Address, token_out: Address, amount: U256 },
    RepayCompound { ctoken: Address, amount: U256 },
    WithdrawCompound { ctoken: Address, amount: U256 },
}

pub struct CompoundManager {
    chain_manager: Arc<ChainManager>,
    dex_manager: Arc<DexManager>,
    contracts: HashMap<u64, CompoundContracts>,
    ctoken_cache: Arc<tokio::sync::RwLock<HashMap<(u64, Address), CTokenInfo>>>,
    user_data_cache: Arc<tokio::sync::RwLock<HashMap<(u64, Address), UserCompoundData>>>,
    oracle_prices_cache: Arc<tokio::sync::RwLock<HashMap<Address, (U256, std::time::Instant)>>>,
}

impl CompoundManager {
    pub async fn new(chain_manager: Arc<ChainManager>, dex_manager: Arc<DexManager>) -> Result<Self> {
        let mut contracts = HashMap::new();
        
        // Ethereum mainnet contracts
        contracts.insert(1, CompoundContracts {
            comptroller: "0x3d9819210A31b4961b30EF54bE2aeD79B9c9Cd3B".parse()?,
            price_oracle: "0x922018674c12a7F0D394ebEEf9B58F186CdE13c1".parse()?,
            comp_token: "0xc00e94Cb662C3520282E6f5717214004A7f26888".parse()?,
            ceth: "0x4Ddc2D193948926D02f9B1fE9e1daa0718270ED5".parse()?,
            cdai: "0x5d3a536E4D6DbD6114cc1Ead35777bAB948E3643".parse()?,
            cusdc: "0x39AA39c021dfbaE8faC545936693aC917d5E7563".parse()?,
            cwbtc: "0xC11b1268C1A384e55C48c2391d8d480264A3A7F4".parse()?,
        });

        Ok(Self {
            chain_manager,
            dex_manager,
            contracts,
            ctoken_cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            user_data_cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            oracle_prices_cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        })
    }

    pub async fn get_ctoken_info(&self, chain_id: u64, ctoken: Address) -> Result<CTokenInfo> {
        // Check cache first
        {
            let cache = self.ctoken_cache.read().await;
            if let Some(info) = cache.get(&(chain_id, ctoken)) {
                return Ok(info.clone());
            }
        }

        let provider = self.chain_manager.get_provider(chain_id).await?;
        let ctoken_contract = Contract::new(
            ctoken,
            Self::get_ctoken_abi()?,
            Arc::new(provider.provider.clone()),
        );

        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Unsupported chain: {}", chain_id))?;

        let comptroller_contract = Contract::new(
            contracts.comptroller,
            Self::get_comptroller_abi()?,
            Arc::new(provider.provider.clone()),
        );

        // Get basic cToken data
        let symbol: String = ctoken_contract.method("symbol", ())?.call().await?;
        let decimals: u8 = ctoken_contract.method("decimals", ())?.call().await?;
        let exchange_rate: U256 = ctoken_contract.method("exchangeRateStored", ())?.call().await?;
        let supply_rate: U256 = ctoken_contract.method("supplyRatePerBlock", ())?.call().await?;
        let borrow_rate: U256 = ctoken_contract.method("borrowRatePerBlock", ())?.call().await?;
        let total_supply: U256 = ctoken_contract.method("totalSupply", ())?.call().await?;
        let total_borrows: U256 = ctoken_contract.method("totalBorrows", ())?.call().await?;
        let total_reserves: U256 = ctoken_contract.method("totalReserves", ())?.call().await?;
        let cash: U256 = ctoken_contract.method("getCash", ())?.call().await?;
        let reserve_factor: U256 = ctoken_contract.method("reserveFactorMantissa", ())?.call().await?;

        // Get underlying token address (or use ETH address for cETH)
        let underlying_address = if ctoken == contracts.ceth {
            "0x0000000000000000000000000000000000000000".parse()?
        } else {
            ctoken_contract.method("underlying", ())?.call().await?
        };

        // Get market data from comptroller
        let market_data: (bool, U256, bool) = comptroller_contract
            .method("markets", ctoken)?
            .call()
            .await?;

        let collateral_factor = market_data.1;

        // Get COMP speeds
        let comp_speed_supply: U256 = comptroller_contract
            .method("compSupplySpeeds", ctoken)?
            .call()
            .await
            .unwrap_or(U256::zero());

        let comp_speed_borrow: U256 = comptroller_contract
            .method("compBorrowSpeeds", ctoken)?
            .call()
            .await
            .unwrap_or(U256::zero());

        let liquidation_incentive: U256 = comptroller_contract
            .method("liquidationIncentiveMantissa", ())?
            .call()
            .await?;

        let ctoken_info = CTokenInfo {
            symbol,
            underlying_address,
            ctoken_address: ctoken,
            decimals,
            exchange_rate,
            supply_rate_per_block: supply_rate,
            borrow_rate_per_block: borrow_rate,
            total_supply,
            total_borrows,
            total_reserves,
            cash,
            collateral_factor,
            liquidation_incentive,
            reserve_factor,
            comp_speed_supply,
            comp_speed_borrow,
        };

        // Cache the result
        {
            let mut cache = self.ctoken_cache.write().await;
            cache.insert((chain_id, ctoken), ctoken_info.clone());
        }

        Ok(ctoken_info)
    }

    pub async fn get_user_compound_data(&self, chain_id: u64, account: Address) -> Result<UserCompoundData> {
        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Unsupported chain: {}", chain_id))?;

        let provider = self.chain_manager.get_provider(chain_id).await?;
        let comptroller_contract = Contract::new(
            contracts.comptroller,
            Self::get_comptroller_abi()?,
            Arc::new(provider.provider.clone()),
        );

        // Get account liquidity
        let account_liquidity: (U256, U256, U256) = comptroller_contract
            .method("getAccountLiquidity", account)?
            .call()
            .await?;

        // Get COMP accrued
        let comp_accrued: U256 = comptroller_contract
            .method("compAccrued", account)?
            .call()
            .await
            .unwrap_or(U256::zero());

        // Get all markets the user has entered
        let entered_markets: Vec<Address> = comptroller_contract
            .method("getAssetsIn", account)?
            .call()
            .await?;

        let mut positions = Vec::new();
        let mut total_supply_value = U256::zero();
        let mut total_borrow_value = U256::zero();

        // Get user positions in each market
        for ctoken in entered_markets {
            let position = self.get_user_ctoken_position(chain_id, ctoken, account).await?;
            
            // Calculate USD values (simplified - would use oracle in production)
            let supply_value = position.supply_balance; // Mock calculation
            let borrow_value = position.borrow_balance; // Mock calculation
            
            total_supply_value += supply_value;
            total_borrow_value += borrow_value;
            
            positions.push(position);
        }

        // Calculate health factor
        let health_factor = if total_borrow_value.is_zero() {
            f64::INFINITY
        } else {
            (account_liquidity.1.as_u128() as f64) / (total_borrow_value.as_u128() as f64)
        };

        Ok(UserCompoundData {
            account,
            total_supply_value,
            total_borrow_value,
            account_liquidity: account_liquidity.1,
            shortfall: account_liquidity.2,
            health_factor,
            positions,
            comp_accrued,
        })
    }

    pub async fn get_user_ctoken_position(&self, chain_id: u64, ctoken: Address, account: Address) -> Result<UserCTokenPosition> {
        let provider = self.chain_manager.get_provider(chain_id).await?;
        let ctoken_contract = Contract::new(
            ctoken,
            Self::get_ctoken_abi()?,
            Arc::new(provider.provider.clone()),
        );

        let ctoken_info = self.get_ctoken_info(chain_id, ctoken).await?;

        // Get user balances
        let supply_balance: U256 = ctoken_contract.method("balanceOf", account)?.call().await?;
        let borrow_balance: U256 = ctoken_contract.method("borrowBalanceStored", account)?.call().await?;

        // Calculate APYs (simplified calculation)
        let blocks_per_year = 2102400u64; // Approximate blocks per year
        let supply_apy = (ctoken_info.supply_rate_per_block.as_u128() as f64) * (blocks_per_year as f64) / 1e18 * 100.0;
        let borrow_apy = (ctoken_info.borrow_rate_per_block.as_u128() as f64) * (blocks_per_year as f64) / 1e18 * 100.0;

        // Mock COMP APY calculation
        let comp_apy_supply = 2.5; // Mock value
        let comp_apy_borrow = 1.8; // Mock value

        // Check if asset is used as collateral
        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Unsupported chain: {}", chain_id))?;

        let comptroller_contract = Contract::new(
            contracts.comptroller,
            Self::get_comptroller_abi()?,
            Arc::new(provider.provider.clone()),
        );

        let entered_markets: Vec<Address> = comptroller_contract
            .method("getAssetsIn", account)?
            .call()
            .await?;

        let is_collateral = entered_markets.contains(&ctoken);

        Ok(UserCTokenPosition {
            ctoken,
            underlying_symbol: ctoken_info.symbol.replace("c", "").to_uppercase(),
            supply_balance,
            borrow_balance,
            supply_apy,
            borrow_apy,
            comp_apy_supply,
            comp_apy_borrow,
            collateral_factor: ctoken_info.collateral_factor,
            is_collateral,
        })
    }

    pub async fn supply(&self, chain_id: u64, ctoken: Address, amount: U256) -> Result<TransactionRequest> {
        let provider = self.chain_manager.get_provider(chain_id).await?;
        let ctoken_contract = Contract::new(
            ctoken,
            Self::get_ctoken_abi()?,
            Arc::new(provider.provider.clone()),
        );

        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Unsupported chain: {}", chain_id))?;

        let tx = if ctoken == contracts.ceth {
            // For ETH, use mint() with value
            let mut mint_tx = ctoken_contract.method::<_, H256>("mint", ())?;
            mint_tx.tx.set_value(amount);
            mint_tx.tx
        } else {
            // For ERC20 tokens, use mint(amount)
            ctoken_contract.method::<_, H256>("mint", amount)?.tx
        };

        Ok(tx.into())
    }

    pub async fn borrow(&self, chain_id: u64, ctoken: Address, amount: U256) -> Result<TransactionRequest> {
        let provider = self.chain_manager.get_provider(chain_id).await?;
        let ctoken_contract = Contract::new(
            ctoken,
            Self::get_ctoken_abi()?,
            Arc::new(provider.provider.clone()),
        );

        let tx = ctoken_contract.method::<_, H256>("borrow", amount)?.tx;
        Ok(tx.into())
    }

    pub async fn repay(&self, chain_id: u64, ctoken: Address, amount: U256) -> Result<TransactionRequest> {
        let provider = self.chain_manager.get_provider(chain_id).await?;
        let ctoken_contract = Contract::new(
            ctoken,
            Self::get_ctoken_abi()?,
            Arc::new(provider.provider.clone()),
        );

        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Unsupported chain: {}", chain_id))?;

        let tx = if ctoken == contracts.ceth {
            // For ETH, use repayBorrow() with value
            let mut repay_tx = ctoken_contract.method::<_, H256>("repayBorrow", ())?;
            repay_tx.tx.set_value(amount);
            repay_tx.tx
        } else {
            // For ERC20 tokens, use repayBorrow(amount)
            ctoken_contract.method::<_, H256>("repayBorrow", amount)?.tx
        };

        Ok(tx.into())
    }

    pub async fn redeem(&self, chain_id: u64, ctoken: Address, ctokens_amount: U256) -> Result<TransactionRequest> {
        let provider = self.chain_manager.get_provider(chain_id).await?;
        let ctoken_contract = Contract::new(
            ctoken,
            Self::get_ctoken_abi()?,
            Arc::new(provider.provider.clone()),
        );

        let tx = ctoken_contract.method::<_, H256>("redeem", ctokens_amount)?.tx;
        Ok(tx.into())
    }

    pub async fn redeem_underlying(&self, chain_id: u64, ctoken: Address, underlying_amount: U256) -> Result<TransactionRequest> {
        let provider = self.chain_manager.get_provider(chain_id).await?;
        let ctoken_contract = Contract::new(
            ctoken,
            Self::get_ctoken_abi()?,
            Arc::new(provider.provider.clone()),
        );

        let tx = ctoken_contract.method::<_, H256>("redeemUnderlying", underlying_amount)?.tx;
        Ok(tx.into())
    }

    pub async fn enter_markets(&self, chain_id: u64, ctokens: Vec<Address>) -> Result<TransactionRequest> {
        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Unsupported chain: {}", chain_id))?;

        let provider = self.chain_manager.get_provider(chain_id).await?;
        let comptroller_contract = Contract::new(
            contracts.comptroller,
            Self::get_comptroller_abi()?,
            Arc::new(provider.provider.clone()),
        );

        let tx = comptroller_contract.method::<_, H256>("enterMarkets", ctokens)?.tx;
        Ok(tx.into())
    }

    pub async fn exit_market(&self, chain_id: u64, ctoken: Address) -> Result<TransactionRequest> {
        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Unsupported chain: {}", chain_id))?;

        let provider = self.chain_manager.get_provider(chain_id).await?;
        let comptroller_contract = Contract::new(
            contracts.comptroller,
            Self::get_comptroller_abi()?,
            Arc::new(provider.provider.clone()),
        );

        let tx = comptroller_contract.method::<_, H256>("exitMarket", ctoken)?.tx;
        Ok(tx.into())
    }

    pub async fn claim_comp(&self, chain_id: u64, holder: Address, ctokens: Vec<Address>) -> Result<TransactionRequest> {
        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Unsupported chain: {}", chain_id))?;

        let provider = self.chain_manager.get_provider(chain_id).await?;
        let comptroller_contract = Contract::new(
            contracts.comptroller,
            Self::get_comptroller_abi()?,
            Arc::new(provider.provider.clone()),
        );

        let tx = comptroller_contract.method::<_, H256>("claimComp", (holder, ctokens))?.tx;
        Ok(tx.into())
    }

    pub async fn liquidate_borrow(&self, chain_id: u64, ctoken_borrowed: Address, ctoken_collateral: Address, borrower: Address, repay_amount: U256) -> Result<TransactionRequest> {
        let provider = self.chain_manager.get_provider(chain_id).await?;
        let ctoken_contract = Contract::new(
            ctoken_borrowed,
            Self::get_ctoken_abi()?,
            Arc::new(provider.provider.clone()),
        );

        let tx = ctoken_contract.method::<_, H256>("liquidateBorrow", (borrower, repay_amount, ctoken_collateral))?.tx;
        Ok(tx.into())
    }

    pub async fn get_yield_strategies(&self, chain_id: u64, asset: Address, amount: U256) -> Result<Vec<CompoundYieldStrategy>> {
        let mut strategies = Vec::new();

        // Get relevant cToken
        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Unsupported chain: {}", chain_id))?;

        let ctoken = if asset == "0x0000000000000000000000000000000000000000".parse()? {
            contracts.ceth
        } else {
            contracts.cusdc // Mock - would need proper mapping
        };

        // Strategy 1: Simple supply
        strategies.push(CompoundYieldStrategy {
            strategy_id: "compound_supply".to_string(),
            name: "Compound Supply".to_string(),
            description: "Simple supply to Compound to earn interest and COMP rewards".to_string(),
            estimated_apy: 4.2,
            risk_level: RiskLevel::Low,
            min_deposit: U256::from(1000u64),
            assets_involved: vec![asset],
            steps: vec![
                CompoundStep::Supply { ctoken, amount_ratio: 1.0 },
                CompoundStep::EnterMarkets { ctokens: vec![ctoken] },
            ],
        });

        // Strategy 2: Leveraged supply
        strategies.push(CompoundYieldStrategy {
            strategy_id: "compound_leveraged".to_string(),
            name: "Leveraged Compound Supply".to_string(),
            description: "Supply collateral, borrow same asset, re-supply for higher returns".to_string(),
            estimated_apy: 8.7,
            risk_level: RiskLevel::High,
            min_deposit: U256::from(10000u64),
            assets_involved: vec![asset],
            steps: vec![
                CompoundStep::Supply { ctoken, amount_ratio: 1.0 },
                CompoundStep::EnterMarkets { ctokens: vec![ctoken] },
                CompoundStep::Borrow { ctoken, amount_ratio: 0.75 },
                CompoundStep::Supply { ctoken, amount_ratio: 0.75 },
            ],
        });

        // Strategy 3: COMP farming
        strategies.push(CompoundYieldStrategy {
            strategy_id: "compound_comp_farming".to_string(),
            name: "COMP Token Farming".to_string(),
            description: "Optimize for maximum COMP rewards through borrowing and supplying".to_string(),
            estimated_apy: 15.3,
            risk_level: RiskLevel::Medium,
            min_deposit: U256::from(5000u64),
            assets_involved: vec![asset, contracts.comp_token],
            steps: vec![
                CompoundStep::Supply { ctoken: contracts.cusdc, amount_ratio: 1.0 },
                CompoundStep::EnterMarkets { ctokens: vec![contracts.cusdc] },
                CompoundStep::Borrow { ctoken: contracts.cdai, amount_ratio: 0.8 },
                CompoundStep::ClaimComp { account: Address::zero() },
                CompoundStep::SwapCompForAsset { asset },
            ],
        });

        Ok(strategies)
    }

    pub async fn find_liquidation_opportunities(&self, chain_id: u64) -> Result<Vec<LiquidationOpportunity>> {
        let mut opportunities = Vec::new();

        // Mock implementation - in production would scan all accounts
        let mock_accounts = vec![
            "0x1234567890123456789012345678901234567890".parse::<Address>()?,
            "0x2345678901234567890123456789012345678901".parse::<Address>()?,
        ];

        for account in mock_accounts {
            let user_data = self.get_user_compound_data(chain_id, account).await?;
            
            if user_data.shortfall > U256::zero() {
                // Account is under-collateralized, find liquidation opportunity
                for position in &user_data.positions {
                    if position.borrow_balance > U256::zero() {
                        let opportunity = LiquidationOpportunity {
                            account,
                            ctoken_borrowed: position.ctoken,
                            ctoken_collateral: position.ctoken, // Simplified
                            repay_amount: position.borrow_balance / U256::from(2), // Max 50% can be repaid
                            seize_amount: position.borrow_balance * U256::from(108) / U256::from(100), // 8% liquidation incentive
                            profit_estimate: position.borrow_balance * U256::from(8) / U256::from(100),
                            health_factor: user_data.health_factor,
                            liquidation_incentive: 8.0,
                        };
                        opportunities.push(opportunity);
                    }
                }
            }
        }

        Ok(opportunities)
    }

    pub async fn find_arbitrage_opportunities(&self, chain_id: u64) -> Result<Vec<CompArbitrageOpportunity>> {
        let mut opportunities = Vec::new();

        // Strategy 1: Rate arbitrage between Compound and Aave
        let compound_rates = self.get_all_borrow_rates(chain_id).await?;
        
        for (ctoken, borrow_rate) in compound_rates {
            // Mock Aave rate comparison
            let aave_supply_rate = U256::from(35000000000000000u64); // 3.5%
            
            if borrow_rate < aave_supply_rate {
                let profit_per_year = aave_supply_rate - borrow_rate;
                let required_capital = U256::from(100000u64); // $100k example
                let profit_estimate = required_capital * profit_per_year / U256::from(1e18 as u64);
                
                opportunities.push(CompArbitrageOpportunity {
                    strategy_type: "Rate Arbitrage".to_string(),
                    profit_estimate,
                    gas_estimate: U256::from(500000u64), // Mock gas cost
                    net_profit: profit_estimate - U256::from(500000u64),
                    required_capital,
                    success_probability: 0.85,
                    operations: vec![
                        ArbitrageOperation::BorrowCompound { ctoken, amount: required_capital },
                        ArbitrageOperation::SwapDex { 
                            token_in: ctoken, 
                            token_out: "0xA0b86a33E6441E5A3D3CdeC19A4F6BbBc2A906b4".parse()?, // Mock USDC
                            amount: required_capital 
                        },
                    ],
                });
            }
        }

        // Strategy 2: Liquidation arbitrage
        let liquidation_ops = self.find_liquidation_opportunities(chain_id).await?;
        for liq_op in liquidation_ops {
            opportunities.push(CompArbitrageOpportunity {
                strategy_type: "Liquidation Arbitrage".to_string(),
                profit_estimate: liq_op.profit_estimate,
                gas_estimate: U256::from(300000u64),
                net_profit: liq_op.profit_estimate - U256::from(300000u64),
                required_capital: liq_op.repay_amount,
                success_probability: 0.95,
                operations: vec![
                    ArbitrageOperation::RepayCompound { 
                        ctoken: liq_op.ctoken_borrowed, 
                        amount: liq_op.repay_amount 
                    },
                ],
            });
        }

        Ok(opportunities)
    }

    pub async fn get_all_borrow_rates(&self, chain_id: u64) -> Result<Vec<(Address, U256)>> {
        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Unsupported chain: {}", chain_id))?;

        let mut rates = Vec::new();
        let ctokens = vec![
            contracts.ceth,
            contracts.cdai,
            contracts.cusdc,
            contracts.cwbtc,
        ];

        for ctoken in ctokens {
            let ctoken_info = self.get_ctoken_info(chain_id, ctoken).await?;
            rates.push((ctoken, ctoken_info.borrow_rate_per_block));
        }

        Ok(rates)
    }

    pub async fn calculate_liquidation_profit(&self, chain_id: u64, opportunity: &LiquidationOpportunity) -> Result<U256> {
        // Get current exchange rates and prices
        let ctoken_info = self.get_ctoken_info(chain_id, opportunity.ctoken_borrowed).await?;
        let collateral_info = self.get_ctoken_info(chain_id, opportunity.ctoken_collateral).await?;

        // Calculate profit considering gas costs and slippage
        let base_profit = opportunity.seize_amount - opportunity.repay_amount;
        let gas_cost = U256::from(300000u64); // Mock gas cost in USD
        let slippage_cost = base_profit * U256::from(3) / U256::from(100); // 3% slippage

        let net_profit = if base_profit > gas_cost + slippage_cost {
            base_profit - gas_cost - slippage_cost
        } else {
            U256::zero()
        };

        Ok(net_profit)
    }

    fn get_ctoken_abi() -> Result<Abi> {
        let abi_json = r#"[
            {
                "inputs": [],
                "name": "mint",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "payable",
                "type": "function"
            },
            {
                "inputs": [{"internalType": "uint256", "name": "mintAmount", "type": "uint256"}],
                "name": "mint",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "nonpayable",
                "type": "function"
            },
            {
                "inputs": [{"internalType": "uint256", "name": "borrowAmount", "type": "uint256"}],
                "name": "borrow",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "nonpayable",
                "type": "function"
            },
            {
                "inputs": [],
                "name": "repayBorrow",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "payable",
                "type": "function"
            },
            {
                "inputs": [{"internalType": "uint256", "name": "repayAmount", "type": "uint256"}],
                "name": "repayBorrow",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "nonpayable",
                "type": "function"
            },
            {
                "inputs": [{"internalType": "uint256", "name": "redeemTokens", "type": "uint256"}],
                "name": "redeem",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "nonpayable",
                "type": "function"
            },
            {
                "inputs": [{"internalType": "uint256", "name": "redeemAmount", "type": "uint256"}],
                "name": "redeemUnderlying",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "nonpayable",
                "type": "function"
            },
            {
                "inputs": [
                    {"internalType": "address", "name": "borrower", "type": "address"},
                    {"internalType": "uint256", "name": "repayAmount", "type": "uint256"},
                    {"internalType": "address", "name": "cTokenCollateral", "type": "address"}
                ],
                "name": "liquidateBorrow",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "payable",
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
                "inputs": [{"internalType": "address", "name": "account", "type": "address"}],
                "name": "borrowBalanceStored",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [],
                "name": "exchangeRateStored",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [],
                "name": "supplyRatePerBlock",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [],
                "name": "borrowRatePerBlock",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
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
                "inputs": [],
                "name": "totalBorrows",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [],
                "name": "totalReserves",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [],
                "name": "getCash",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [],
                "name": "reserveFactorMantissa",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [],
                "name": "underlying",
                "outputs": [{"internalType": "address", "name": "", "type": "address"}],
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
                "name": "decimals",
                "outputs": [{"internalType": "uint8", "name": "", "type": "uint8"}],
                "stateMutability": "view",
                "type": "function"
            }
        ]"#;

        let abi: Abi = serde_json::from_str(abi_json)?;
        Ok(abi)
    }

    fn get_comptroller_abi() -> Result<Abi> {
        let abi_json = r#"[
            {
                "inputs": [{"internalType": "address[]", "name": "cTokens", "type": "address[]"}],
                "name": "enterMarkets",
                "outputs": [{"internalType": "uint256[]", "name": "", "type": "uint256[]"}],
                "stateMutability": "nonpayable",
                "type": "function"
            },
            {
                "inputs": [{"internalType": "address", "name": "cTokenAddress", "type": "address"}],
                "name": "exitMarket",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "nonpayable",
                "type": "function"
            },
            {
                "inputs": [{"internalType": "address", "name": "account", "type": "address"}],
                "name": "getAccountLiquidity",
                "outputs": [
                    {"internalType": "uint256", "name": "", "type": "uint256"},
                    {"internalType": "uint256", "name": "", "type": "uint256"},
                    {"internalType": "uint256", "name": "", "type": "uint256"}
                ],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [{"internalType": "address", "name": "account", "type": "address"}],
                "name": "getAssetsIn",
                "outputs": [{"internalType": "address[]", "name": "", "type": "address[]"}],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [{"internalType": "address", "name": "", "type": "address"}],
                "name": "markets",
                "outputs": [
                    {"internalType": "bool", "name": "isListed", "type": "bool"},
                    {"internalType": "uint256", "name": "collateralFactorMantissa", "type": "uint256"},
                    {"internalType": "bool", "name": "isComped", "type": "bool"}
                ],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [
                    {"internalType": "address", "name": "holder", "type": "address"},
                    {"internalType": "address[]", "name": "cTokens", "type": "address[]"}
                ],
                "name": "claimComp",
                "outputs": [],
                "stateMutability": "nonpayable",
                "type": "function"
            },
            {
                "inputs": [{"internalType": "address", "name": "", "type": "address"}],
                "name": "compAccrued",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [{"internalType": "address", "name": "", "type": "address"}],
                "name": "compSupplySpeeds",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [{"internalType": "address", "name": "", "type": "address"}],
                "name": "compBorrowSpeeds",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [],
                "name": "liquidationIncentiveMantissa",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "view",
                "type": "function"
            }
        ]"#;

        let abi: Abi = serde_json::from_str(abi_json)?;
        Ok(abi)
    }
}


