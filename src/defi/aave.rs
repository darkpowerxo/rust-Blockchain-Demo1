use std::{sync::Arc, collections::HashMap};
use ethers::types::{Address, U256, H256, Bytes, TransactionRequest};
use ethers::abi::{Abi, Token, ParamType, AbiEncode};
use ethers::contract::Contract;
use crate::chains::ChainManager;
use crate::dex::DexManager;
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AaveContracts {
    pub lending_pool: Address,
    pub lending_pool_addresses_provider: Address,
    pub price_oracle: Address,
    pub data_provider: Address,
    pub flash_loan_receiver: Address,
    pub weth_gateway: Address,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReserveData {
    pub asset: Address,
    pub symbol: String,
    pub decimals: u8,
    pub ltv: u16,
    pub liquidation_threshold: u16,
    pub liquidation_bonus: u16,
    pub reserve_factor: u16,
    pub usage_as_collateral_enabled: bool,
    pub borrowing_enabled: bool,
    pub stable_rate_borrowing_enabled: bool,
    pub is_active: bool,
    pub is_frozen: bool,
    pub liquidity_rate: U256,
    pub variable_borrow_rate: U256,
    pub stable_borrow_rate: U256,
    pub liquidity_index: U256,
    pub variable_borrow_index: U256,
    pub a_token_address: Address,
    pub stable_debt_token_address: Address,
    pub variable_debt_token_address: Address,
    pub interest_rate_strategy_address: Address,
    pub last_update_timestamp: u64,
    pub available_liquidity: U256,
    pub total_stable_debt: U256,
    pub total_variable_debt: U256,
    pub utilization_rate: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccountData {
    pub total_collateral_eth: U256,
    pub total_debt_eth: U256,
    pub available_borrows_eth: U256,
    pub current_liquidation_threshold: U256,
    pub ltv: U256,
    pub health_factor: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserReserveData {
    pub asset: Address,
    pub current_a_token_balance: U256,
    pub current_stable_debt: U256,
    pub current_variable_debt: U256,
    pub principal_stable_debt: U256,
    pub scaled_variable_debt: U256,
    pub stable_borrow_rate: U256,
    pub liquidity_rate: U256,
    pub stable_rate_last_updated: u64,
    pub usage_as_collateral_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashLoanParams {
    pub assets: Vec<Address>,
    pub amounts: Vec<U256>,
    pub modes: Vec<u8>, // 0 = no debt, 1 = stable rate, 2 = variable rate
    pub receiver: Address,
    pub params: Bytes,
    pub referral_code: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashLoanStrategy {
    pub strategy_name: String,
    pub description: String,
    pub target_profit: U256,
    pub max_gas_fee: U256,
    pub operations: Vec<FlashLoanOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlashLoanOperation {
    Swap {
        dex: String,
        token_in: Address,
        token_out: Address,
        amount_in: U256,
        min_amount_out: U256,
    },
    Supply {
        protocol: String,
        asset: Address,
        amount: U256,
    },
    Borrow {
        protocol: String,
        asset: Address,
        amount: U256,
        interest_rate_mode: u8,
    },
    Repay {
        protocol: String,
        asset: Address,
        amount: U256,
        interest_rate_mode: u8,
    },
    Withdraw {
        protocol: String,
        asset: Address,
        amount: U256,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LendingPosition {
    pub user: Address,
    pub asset: Address,
    pub supplied_amount: U256,
    pub borrowed_amount_stable: U256,
    pub borrowed_amount_variable: U256,
    pub collateral_value_eth: U256,
    pub debt_value_eth: U256,
    pub health_factor: U256,
    pub liquidation_threshold: U256,
    pub available_borrows: U256,
    pub apy_supplied: f64,
    pub apy_borrowed_stable: f64,
    pub apy_borrowed_variable: f64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YieldStrategy {
    pub strategy_id: String,
    pub name: String,
    pub description: String,
    pub estimated_apy: f64,
    pub risk_level: RiskLevel,
    pub min_deposit: U256,
    pub assets_involved: Vec<Address>,
    pub steps: Vec<YieldStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum YieldStep {
    Supply { protocol: String, asset: Address, amount_ratio: f64 },
    Borrow { protocol: String, asset: Address, amount_ratio: f64, rate_mode: u8 },
    Swap { dex: String, token_in: Address, token_out: Address },
    Farm { protocol: String, pool_address: Address },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    VeryHigh,
}

pub struct AaveManager {
    chain_manager: Arc<ChainManager>,
    dex_manager: Arc<DexManager>,
    contracts: HashMap<u64, AaveContracts>,
    reserves_cache: Arc<tokio::sync::RwLock<HashMap<(u64, Address), ReserveData>>>,
    user_data_cache: Arc<tokio::sync::RwLock<HashMap<(u64, Address), UserAccountData>>>,
}

impl AaveManager {
    pub async fn new(chain_manager: Arc<ChainManager>, dex_manager: Arc<DexManager>) -> Result<Self> {
        let mut contracts = HashMap::new();
        
        // Ethereum mainnet contracts
        contracts.insert(1, AaveContracts {
            lending_pool: "0x7d2768dE32b0b80b7a3454c06BdAc94A69DDc7A9".parse()?,
            lending_pool_addresses_provider: "0xB53C1a33016B2DC2fF3653530bfF1848a515c8c5".parse()?,
            price_oracle: "0xA50ba011c48153De246E5192C8f9258A2ba79Ca9".parse()?,
            data_provider: "0x057835Ad21a177dbdd3090bB1CAE03EaCF78Fc6d".parse()?,
            flash_loan_receiver: "0x1234567890123456789012345678901234567890".parse()?, // Placeholder
            weth_gateway: "0xcc9a0B7c43DC2a5F023Bb9b738E45B0Ef6B06E04".parse()?,
        });

        // Polygon contracts
        contracts.insert(137, AaveContracts {
            lending_pool: "0x8dFf5E27EA6b7AC08EbFdf9eB090F32ee9a30fcf".parse()?,
            lending_pool_addresses_provider: "0xd05e3E715d945B59290df0ae8eF85c1BdB684744".parse()?,
            price_oracle: "0x0229F777B0fAb107F9591a41d5F02E4e98dB6f2d".parse()?,
            data_provider: "0x7551b5D2763519d4e37e8B81929D336De671d46d".parse()?,
            flash_loan_receiver: "0x1234567890123456789012345678901234567890".parse()?,
            weth_gateway: "0xbEadf48d62aCC944a06EEaE0A9054A90E5A7dc97".parse()?,
        });

        Ok(Self {
            chain_manager,
            dex_manager,
            contracts,
            reserves_cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            user_data_cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        })
    }

    pub async fn get_reserve_data(&self, chain_id: u64, asset: Address) -> Result<ReserveData> {
        // Check cache first
        {
            let cache = self.reserves_cache.read().await;
            if let Some(data) = cache.get(&(chain_id, asset)) {
                return Ok(data.clone());
            }
        }

        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Unsupported chain: {}", chain_id))?;

        let provider = self.chain_manager.get_provider(chain_id).await?;
        let data_provider_contract = Contract::new(
            contracts.data_provider,
            Self::get_data_provider_abi()?,
            Arc::new(provider.provider.clone()),
        );

        // Get reserve data
        let reserve_data: (
            U256, U256, U256, U256, U256, U256, bool, bool, bool, bool
        ) = data_provider_contract
            .method::<_, (U256, U256, U256, U256, U256, U256, bool, bool, bool, bool)>("getReserveData", asset)?
            .call()
            .await?;

        // Get reserve configuration
        let config_data: (
            u16, u16, u16, u16, bool, bool, bool, bool
        ) = data_provider_contract
            .method::<_, (u16, u16, u16, u16, bool, bool, bool, bool)>("getReserveConfigurationData", asset)?
            .call()
            .await?;

        // Get token addresses
        let token_addresses: (Address, Address, Address) = data_provider_contract
            .method::<_, (Address, Address, Address)>("getReserveTokensAddresses", asset)?
            .call()
            .await?;

        // Get symbol and decimals (mock for now)
        let symbol = format!("TOKEN_{}", &format!("{:?}", asset)[2..6].to_uppercase());
        let decimals = 18u8;

        let reserve_data = ReserveData {
            asset,
            symbol,
            decimals,
            ltv: config_data.0,
            liquidation_threshold: config_data.1,
            liquidation_bonus: config_data.2,
            reserve_factor: config_data.3,
            usage_as_collateral_enabled: config_data.4,
            borrowing_enabled: config_data.5,
            stable_rate_borrowing_enabled: config_data.6,
            is_active: config_data.7,
            is_frozen: reserve_data.8,
            liquidity_rate: reserve_data.0,
            variable_borrow_rate: reserve_data.1,
            stable_borrow_rate: reserve_data.2,
            liquidity_index: reserve_data.3,
            variable_borrow_index: reserve_data.4,
            a_token_address: token_addresses.0,
            stable_debt_token_address: token_addresses.1,
            variable_debt_token_address: token_addresses.2,
            interest_rate_strategy_address: "0x0000000000000000000000000000000000000000".parse()?,
            last_update_timestamp: reserve_data.5.as_u64(),
            available_liquidity: U256::zero(),
            total_stable_debt: U256::zero(),
            total_variable_debt: U256::zero(),
            utilization_rate: U256::zero(),
        };

        // Cache the result
        {
            let mut cache = self.reserves_cache.write().await;
            cache.insert((chain_id, asset), reserve_data.clone());
        }

        Ok(reserve_data)
    }

    pub async fn get_user_account_data(&self, chain_id: u64, user: Address) -> Result<UserAccountData> {
        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Unsupported chain: {}", chain_id))?;

        let provider = self.chain_manager.get_provider(chain_id).await?;
        let lending_pool_contract = Contract::new(
            contracts.lending_pool,
            Self::get_lending_pool_abi()?,
            Arc::new(provider.provider.clone()),
        );

        let account_data: (U256, U256, U256, U256, U256, U256) = lending_pool_contract
            .method::<_, (U256, U256, U256, U256, U256, U256)>("getUserAccountData", user)?
            .call()
            .await?;

        Ok(UserAccountData {
            total_collateral_eth: account_data.0,
            total_debt_eth: account_data.1,
            available_borrows_eth: account_data.2,
            current_liquidation_threshold: account_data.3,
            ltv: account_data.4,
            health_factor: account_data.5,
        })
    }

    pub async fn supply(&self, chain_id: u64, asset: Address, amount: U256, user: Address, referral_code: u16) -> Result<TransactionRequest> {
        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Unsupported chain: {}", chain_id))?;

        let provider = self.chain_manager.get_provider(chain_id).await?;
        let lending_pool_contract = Contract::new(
            contracts.lending_pool,
            Self::get_lending_pool_abi()?,
            Arc::new(provider.provider.clone()),
        );

        let tx = lending_pool_contract
            .method::<_, H256>("deposit", (asset, amount, user, referral_code))?
            .tx;

        Ok(tx.into())
    }

    pub async fn borrow(&self, chain_id: u64, asset: Address, amount: U256, interest_rate_mode: u8, referral_code: u16, user: Address) -> Result<TransactionRequest> {
        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Unsupported chain: {}", chain_id))?;

        let provider = self.chain_manager.get_provider(chain_id).await?;
        let lending_pool_contract = Contract::new(
            contracts.lending_pool,
            Self::get_lending_pool_abi()?,
            Arc::new(provider.provider.clone()),
        );

        let tx = lending_pool_contract
            .method::<_, H256>("borrow", (asset, amount, interest_rate_mode, referral_code, user))?
            .tx;

        Ok(tx.into())
    }

    pub async fn repay(&self, chain_id: u64, asset: Address, amount: U256, rate_mode: u8, user: Address) -> Result<TransactionRequest> {
        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Unsupported chain: {}", chain_id))?;

        let provider = self.chain_manager.get_provider(chain_id).await?;
        let lending_pool_contract = Contract::new(
            contracts.lending_pool,
            Self::get_lending_pool_abi()?,
            Arc::new(provider.provider.clone()),
        );

        let tx = lending_pool_contract
            .method::<_, H256>("repay", (asset, amount, rate_mode, user))?
            .tx;

        Ok(tx.into())
    }

    pub async fn withdraw(&self, chain_id: u64, asset: Address, amount: U256, user: Address) -> Result<TransactionRequest> {
        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Unsupported chain: {}", chain_id))?;

        let provider = self.chain_manager.get_provider(chain_id).await?;
        let lending_pool_contract = Contract::new(
            contracts.lending_pool,
            Self::get_lending_pool_abi()?,
            Arc::new(provider.provider.clone()),
        );

        let tx = lending_pool_contract
            .method::<_, H256>("withdraw", (asset, amount, user))?
            .tx;

        Ok(tx.into())
    }

    pub async fn flash_loan(&self, chain_id: u64, params: FlashLoanParams) -> Result<TransactionRequest> {
        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Unsupported chain: {}", chain_id))?;

        let provider = self.chain_manager.get_provider(chain_id).await?;
        let lending_pool_contract = Contract::new(
            contracts.lending_pool,
            Self::get_lending_pool_abi()?,
            Arc::new(provider.provider.clone()),
        );

        let tx = lending_pool_contract
            .method::<_, H256>("flashLoan", (
                params.receiver,
                params.assets,
                params.amounts,
                params.modes,
                Address::zero(),
                params.params,
                params.referral_code
            ))?
            .tx;

        Ok(tx.into())
    }

    pub async fn execute_flash_loan_strategy(&self, chain_id: u64, strategy: FlashLoanStrategy) -> Result<Vec<TransactionRequest>> {
        let mut transactions = Vec::new();
        
        // Step 1: Initiate flash loan
        let flash_loan_assets = strategy.operations.iter()
            .filter_map(|op| match op {
                FlashLoanOperation::Swap { token_in, .. } => Some(*token_in),
                _ => None,
            })
            .collect::<Vec<_>>();

        let flash_loan_amounts = vec![U256::from(1000000u64); flash_loan_assets.len()]; // Mock amounts

        let flash_loan_params = FlashLoanParams {
            assets: flash_loan_assets.clone(),
            amounts: flash_loan_amounts,
            modes: vec![0; flash_loan_assets.len()],
            receiver: "0x1234567890123456789012345678901234567890".parse()?,
            params: Bytes::from(vec![]),
            referral_code: 0,
        };

        let flash_loan_tx = self.flash_loan(chain_id, flash_loan_params).await?;
        transactions.push(flash_loan_tx);

        // Step 2: Execute strategy operations (would be handled by flash loan receiver contract)
        for operation in &strategy.operations {
            match operation {
                FlashLoanOperation::Swap { dex, token_in, token_out, amount_in, min_amount_out } => {
                    // This would be handled by the DEX manager in the flash loan callback
                    println!("Flash loan swap: {} {} -> {} {} on {}", amount_in, token_in, min_amount_out, token_out, dex);
                },
                FlashLoanOperation::Supply { protocol, asset, amount } => {
                    let supply_tx = self.supply(chain_id, *asset, *amount, Address::zero(), 0).await?;
                    println!("Flash loan supply: {} {} to {}", amount, asset, protocol);
                },
                FlashLoanOperation::Borrow { asset, amount, interest_rate_mode, .. } => {
                    let borrow_tx = self.borrow(chain_id, *asset, *amount, *interest_rate_mode, 0, Address::zero()).await?;
                    println!("Flash loan borrow: {} {} at rate mode {}", amount, asset, interest_rate_mode);
                },
                _ => {}
            }
        }

        Ok(transactions)
    }

    pub async fn get_lending_position(&self, chain_id: u64, user: Address) -> Result<Vec<LendingPosition>> {
        let account_data = self.get_user_account_data(chain_id, user).await?;
        let mut positions = Vec::new();

        // Mock implementation - in reality, we'd get all reserves and check user balances
        let mock_assets = vec![
            "0xA0b86a33E6441E5A3D3CdeC19A4F6BbBc2A906b4".parse::<Address>()?, // Mock USDC
            "0x2170Ed0880ac9A755fd29B2688956BD959F933F8".parse::<Address>()?, // Mock ETH
        ];

        for asset in mock_assets {
            let reserve_data = self.get_reserve_data(chain_id, asset).await?;
            
            let position = LendingPosition {
                user,
                asset,
                supplied_amount: U256::from(1000000u64), // Mock data
                borrowed_amount_stable: U256::zero(),
                borrowed_amount_variable: U256::from(500000u64),
                collateral_value_eth: account_data.total_collateral_eth / U256::from(2),
                debt_value_eth: account_data.total_debt_eth / U256::from(2),
                health_factor: account_data.health_factor,
                liquidation_threshold: account_data.current_liquidation_threshold,
                available_borrows: account_data.available_borrows_eth,
                apy_supplied: (reserve_data.liquidity_rate.as_u128() as f64) / 1e27 * 100.0,
                apy_borrowed_stable: (reserve_data.stable_borrow_rate.as_u128() as f64) / 1e27 * 100.0,
                apy_borrowed_variable: (reserve_data.variable_borrow_rate.as_u128() as f64) / 1e27 * 100.0,
                last_updated: Utc::now(),
            };
            
            positions.push(position);
        }

        Ok(positions)
    }

    pub async fn get_yield_strategies(&self, chain_id: u64, asset: Address, amount: U256) -> Result<Vec<YieldStrategy>> {
        let mut strategies = Vec::new();

        // Strategy 1: Simple supply
        strategies.push(YieldStrategy {
            strategy_id: "aave_supply".to_string(),
            name: "Aave Supply".to_string(),
            description: "Simple supply to Aave for earning interest".to_string(),
            estimated_apy: 3.5,
            risk_level: RiskLevel::Low,
            min_deposit: U256::from(1000u64),
            assets_involved: vec![asset],
            steps: vec![
                YieldStep::Supply { 
                    protocol: "aave".to_string(), 
                    asset, 
                    amount_ratio: 1.0 
                }
            ],
        });

        // Strategy 2: Leveraged farming
        strategies.push(YieldStrategy {
            strategy_id: "aave_leveraged_farming".to_string(),
            name: "Leveraged Yield Farming".to_string(),
            description: "Supply collateral, borrow stablecoin, farm on DEX".to_string(),
            estimated_apy: 12.5,
            risk_level: RiskLevel::High,
            min_deposit: U256::from(10000u64),
            assets_involved: vec![asset, "0xA0b86a33E6441E5A3D3CdeC19A4F6BbBc2A906b4".parse()?], // Mock USDC
            steps: vec![
                YieldStep::Supply { 
                    protocol: "aave".to_string(), 
                    asset, 
                    amount_ratio: 1.0 
                },
                YieldStep::Borrow { 
                    protocol: "aave".to_string(), 
                    asset: "0xA0b86a33E6441E5A3D3CdeC19A4F6BbBc2A906b4".parse()?, 
                    amount_ratio: 0.7, 
                    rate_mode: 2 
                },
                YieldStep::Farm { 
                    protocol: "sushiswap".to_string(), 
                    pool_address: "0x1234567890123456789012345678901234567890".parse()? 
                },
            ],
        });

        // Strategy 3: Flash loan arbitrage
        strategies.push(YieldStrategy {
            strategy_id: "aave_flash_arbitrage".to_string(),
            name: "Flash Loan Arbitrage".to_string(),
            description: "Use flash loans for cross-DEX arbitrage opportunities".to_string(),
            estimated_apy: 25.0,
            risk_level: RiskLevel::VeryHigh,
            min_deposit: U256::from(50000u64),
            assets_involved: vec![asset],
            steps: vec![
                YieldStep::Swap { 
                    dex: "uniswap".to_string(), 
                    token_in: asset, 
                    token_out: "0xA0b86a33E6441E5A3D3CdeC19A4F6BbBc2A906b4".parse()? 
                },
                YieldStep::Swap { 
                    dex: "sushiswap".to_string(), 
                    token_in: "0xA0b86a33E6441E5A3D3CdeC19A4F6BbBc2A906b4".parse()?, 
                    token_out: asset 
                },
            ],
        });

        Ok(strategies)
    }

    pub async fn calculate_health_factor(&self, chain_id: u64, user: Address, additional_supply: Option<(Address, U256)>, additional_borrow: Option<(Address, U256)>) -> Result<U256> {
        let mut account_data = self.get_user_account_data(chain_id, user).await?;

        // Simulate additional supply
        if let Some((asset, amount)) = additional_supply {
            let reserve_data = self.get_reserve_data(chain_id, asset).await?;
            let price = self.get_asset_price(chain_id, asset).await?;
            let additional_collateral = amount * price * U256::from(reserve_data.ltv) / U256::from(10000);
            account_data.total_collateral_eth += additional_collateral;
        }

        // Simulate additional borrow
        if let Some((asset, amount)) = additional_borrow {
            let price = self.get_asset_price(chain_id, asset).await?;
            let additional_debt = amount * price;
            account_data.total_debt_eth += additional_debt;
        }

        // Calculate health factor
        if account_data.total_debt_eth.is_zero() {
            return Ok(U256::max_value());
        }

        let health_factor = account_data.total_collateral_eth * account_data.current_liquidation_threshold / (account_data.total_debt_eth * U256::from(10000));
        Ok(health_factor)
    }

    pub async fn get_asset_price(&self, chain_id: u64, asset: Address) -> Result<U256> {
        let contracts = self.contracts.get(&chain_id)
            .ok_or_else(|| anyhow!("Unsupported chain: {}", chain_id))?;

        let provider = self.chain_manager.get_provider(chain_id).await?;
        let oracle_contract = Contract::new(
            contracts.price_oracle,
            Self::get_price_oracle_abi()?,
            Arc::new(provider.provider.clone()),
        );

        let price: U256 = oracle_contract
            .method("getAssetPrice", asset)?
            .call()
            .await?;

        Ok(price)
    }

    fn get_lending_pool_abi() -> Result<Abi> {
        let abi_json = r#"[
            {
                "inputs": [
                    {"internalType": "address", "name": "asset", "type": "address"},
                    {"internalType": "uint256", "name": "amount", "type": "uint256"},
                    {"internalType": "address", "name": "onBehalfOf", "type": "address"},
                    {"internalType": "uint16", "name": "referralCode", "type": "uint16"}
                ],
                "name": "deposit",
                "outputs": [],
                "stateMutability": "nonpayable",
                "type": "function"
            },
            {
                "inputs": [
                    {"internalType": "address", "name": "asset", "type": "address"},
                    {"internalType": "uint256", "name": "amount", "type": "uint256"},
                    {"internalType": "uint256", "name": "interestRateMode", "type": "uint256"},
                    {"internalType": "uint16", "name": "referralCode", "type": "uint16"},
                    {"internalType": "address", "name": "onBehalfOf", "type": "address"}
                ],
                "name": "borrow",
                "outputs": [],
                "stateMutability": "nonpayable",
                "type": "function"
            },
            {
                "inputs": [
                    {"internalType": "address", "name": "asset", "type": "address"},
                    {"internalType": "uint256", "name": "amount", "type": "uint256"},
                    {"internalType": "uint256", "name": "rateMode", "type": "uint256"},
                    {"internalType": "address", "name": "onBehalfOf", "type": "address"}
                ],
                "name": "repay",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "nonpayable",
                "type": "function"
            },
            {
                "inputs": [
                    {"internalType": "address", "name": "asset", "type": "address"},
                    {"internalType": "uint256", "name": "amount", "type": "uint256"},
                    {"internalType": "address", "name": "to", "type": "address"}
                ],
                "name": "withdraw",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "nonpayable",
                "type": "function"
            },
            {
                "inputs": [
                    {"internalType": "address", "name": "receiverAddress", "type": "address"},
                    {"internalType": "address[]", "name": "assets", "type": "address[]"},
                    {"internalType": "uint256[]", "name": "amounts", "type": "uint256[]"},
                    {"internalType": "uint256[]", "name": "modes", "type": "uint256[]"},
                    {"internalType": "address", "name": "onBehalfOf", "type": "address"},
                    {"internalType": "bytes", "name": "params", "type": "bytes"},
                    {"internalType": "uint16", "name": "referralCode", "type": "uint16"}
                ],
                "name": "flashLoan",
                "outputs": [],
                "stateMutability": "nonpayable",
                "type": "function"
            },
            {
                "inputs": [{"internalType": "address", "name": "user", "type": "address"}],
                "name": "getUserAccountData",
                "outputs": [
                    {"internalType": "uint256", "name": "totalCollateralETH", "type": "uint256"},
                    {"internalType": "uint256", "name": "totalDebtETH", "type": "uint256"},
                    {"internalType": "uint256", "name": "availableBorrowsETH", "type": "uint256"},
                    {"internalType": "uint256", "name": "currentLiquidationThreshold", "type": "uint256"},
                    {"internalType": "uint256", "name": "ltv", "type": "uint256"},
                    {"internalType": "uint256", "name": "healthFactor", "type": "uint256"}
                ],
                "stateMutability": "view",
                "type": "function"
            }
        ]"#;

        let abi: Abi = serde_json::from_str(abi_json)?;
        Ok(abi)
    }

    fn get_data_provider_abi() -> Result<Abi> {
        let abi_json = r#"[
            {
                "inputs": [{"internalType": "address", "name": "asset", "type": "address"}],
                "name": "getReserveData",
                "outputs": [
                    {"internalType": "uint256", "name": "liquidityRate", "type": "uint256"},
                    {"internalType": "uint256", "name": "variableBorrowRate", "type": "uint256"},
                    {"internalType": "uint256", "name": "stableBorrowRate", "type": "uint256"},
                    {"internalType": "uint256", "name": "liquidityIndex", "type": "uint256"},
                    {"internalType": "uint256", "name": "variableBorrowIndex", "type": "uint256"},
                    {"internalType": "uint256", "name": "lastUpdateTimestamp", "type": "uint256"},
                    {"internalType": "bool", "name": "usageAsCollateralEnabled", "type": "bool"},
                    {"internalType": "bool", "name": "borrowingEnabled", "type": "bool"},
                    {"internalType": "bool", "name": "stableBorrowRateEnabled", "type": "bool"},
                    {"internalType": "bool", "name": "isActive", "type": "bool"}
                ],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [{"internalType": "address", "name": "asset", "type": "address"}],
                "name": "getReserveConfigurationData",
                "outputs": [
                    {"internalType": "uint256", "name": "ltv", "type": "uint256"},
                    {"internalType": "uint256", "name": "liquidationThreshold", "type": "uint256"},
                    {"internalType": "uint256", "name": "liquidationBonus", "type": "uint256"},
                    {"internalType": "uint256", "name": "reserveFactor", "type": "uint256"},
                    {"internalType": "bool", "name": "usageAsCollateralEnabled", "type": "bool"},
                    {"internalType": "bool", "name": "borrowingEnabled", "type": "bool"},
                    {"internalType": "bool", "name": "stableBorrowRateEnabled", "type": "bool"},
                    {"internalType": "bool", "name": "isActive", "type": "bool"}
                ],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [{"internalType": "address", "name": "asset", "type": "address"}],
                "name": "getReserveTokensAddresses",
                "outputs": [
                    {"internalType": "address", "name": "aTokenAddress", "type": "address"},
                    {"internalType": "address", "name": "stableDebtTokenAddress", "type": "address"},
                    {"internalType": "address", "name": "variableDebtTokenAddress", "type": "address"}
                ],
                "stateMutability": "view",
                "type": "function"
            }
        ]"#;

        let abi: Abi = serde_json::from_str(abi_json)?;
        Ok(abi)
    }

    fn get_price_oracle_abi() -> Result<Abi> {
        let abi_json = r#"[
            {
                "inputs": [{"internalType": "address", "name": "asset", "type": "address"}],
                "name": "getAssetPrice",
                "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
                "stateMutability": "view",
                "type": "function"
            }
        ]"#;

        let abi: Abi = serde_json::from_str(abi_json)?;
        Ok(abi)
    }

    // API Support Methods
    
    /// Supply asset to Aave (API-friendly wrapper)
    pub async fn supply_asset(
        &self,
        chain_id: u64,
        asset: Address,
        amount: U256,
        user: Address,
    ) -> Result<TransactionRequest> {
        // Use the existing supply method with default referral code
        self.supply(chain_id, asset, amount, user, 0).await
    }

    /// Withdraw asset from Aave (API-friendly wrapper)
    pub async fn withdraw_asset(
        &self,
        chain_id: u64,
        asset: Address,
        amount: U256,
        user: Address,
    ) -> Result<TransactionRequest> {
        // Use the existing withdraw method
        self.withdraw(chain_id, asset, amount, user).await
    }

    /// Borrow asset from Aave (API-friendly wrapper)
    pub async fn borrow_asset(
        &self,
        chain_id: u64,
        asset: Address,
        amount: U256,
        user: Address,
    ) -> Result<TransactionRequest> {
        // Use the existing borrow method with default parameters
        // interest_rate_mode: 2 = variable rate, referral_code: 0
        self.borrow(chain_id, asset, amount, 2, 0, user).await
    }

    /// Repay asset to Aave (API-friendly wrapper)
    pub async fn repay_asset(
        &self,
        chain_id: u64,
        asset: Address,
        amount: U256,
        user: Address,
    ) -> Result<TransactionRequest> {
        // Use the existing repay method with default parameters
        // interest_rate_mode: 2 = variable rate
        self.repay(chain_id, asset, amount, 2, user).await
    }
}

