use anyhow::{Result, anyhow};
use ethers::{
    prelude::*,
    types::{Address, U256, TransactionRequest, H256, Bytes},
};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeFiThreat {
    FlashLoanAttack {
        loan_amount: U256,
        loan_token: Address,
        attack_vector: AttackVector,
    },
    LiquidationAttack {
        target_position: Address,
        liquidation_amount: U256,
        profit_estimation: U256,
    },
    GovernanceAttack {
        proposal_id: U256,
        attack_type: GovernanceAttackType,
    },
    PriceManipulation {
        target_token: Address,
        manipulation_method: ManipulationMethod,
    },
    ReentrancyAttack {
        target_function: String,
        call_depth: u8,
    },
    ArbitrageBot {
        opportunity_value: U256,
        execution_route: Vec<Address>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttackVector {
    PriceOracle,
    Liquidity,
    Governance,
    ReentrancyExploit,
    LogicBomb,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GovernanceAttackType {
    ProposalManipulation,
    VotingPowerConcentration,
    FlashLoanGovernance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ManipulationMethod {
    LiquidityDrain,
    FlashLoanPump,
    OracleManipulation,
    SandwichAttack,
}

#[derive(Debug, Clone)]
pub struct DeFiProtocolConfig {
    pub protocol_address: Address,
    pub protocol_type: ProtocolType,
    pub risk_level: RiskLevel,
    pub max_transaction_value: U256,
    pub allowed_functions: HashSet<String>,
    pub rate_limits: RateLimits,
    pub emergency_pause: bool,
}

#[derive(Debug, Clone)]
pub enum ProtocolType {
    Lending(LendingConfig),
    Dex(DexConfig),
    Yield(YieldConfig),
    Insurance(InsuranceConfig),
    Governance(GovernanceConfig),
}

#[derive(Debug, Clone)]
pub struct LendingConfig {
    pub max_ltv: f64,
    pub liquidation_threshold: f64,
    pub min_health_factor: f64,
}

#[derive(Debug, Clone)]
pub struct DexConfig {
    pub max_slippage: f64,
    pub min_liquidity: U256,
    pub max_price_impact: f64,
}

#[derive(Debug, Clone)]
pub struct YieldConfig {
    pub max_apy: f64,
    pub min_lock_period: Duration,
    pub penalty_threshold: f64,
}

#[derive(Debug, Clone)]
pub struct InsuranceConfig {
    pub coverage_ratio: f64,
    pub claim_period: Duration,
    pub max_claim_amount: U256,
}

#[derive(Debug, Clone)]
pub struct GovernanceConfig {
    pub min_voting_power: U256,
    pub proposal_threshold: U256,
    pub voting_period: Duration,
}

#[derive(Debug, Clone)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct RateLimits {
    pub max_transactions_per_minute: u32,
    pub max_value_per_hour: U256,
    pub cooldown_period: Duration,
}

pub struct DeFiSecurity {
    provider: Arc<Provider<Http>>,
    protocol_configs: Arc<RwLock<HashMap<Address, DeFiProtocolConfig>>>,
    transaction_history: Arc<RwLock<HashMap<Address, Vec<DeFiTransaction>>>>,
    threat_detector: Arc<RwLock<ThreatDetector>>,
    position_monitor: Arc<RwLock<PositionMonitor>>,
    rate_limiter: Arc<RwLock<RateLimiter>>,
}

#[derive(Debug, Clone)]
struct DeFiTransaction {
    pub hash: H256,
    pub from: Address,
    pub to: Address,
    pub value: U256,
    pub function_selector: [u8; 4],
    pub timestamp: DateTime<Utc>,
    pub gas_used: U256,
    pub success: bool,
}

#[derive(Debug, Clone)]
struct ThreatDetector {
    flash_loan_patterns: HashMap<Address, Vec<FlashLoanPattern>>,
    liquidation_targets: HashMap<Address, LiquidationRisk>,
    suspicious_addresses: HashSet<Address>,
    attack_signatures: Vec<AttackSignature>,
}

#[derive(Debug, Clone)]
struct FlashLoanPattern {
    pub loan_provider: Address,
    pub loan_amount: U256,
    pub repay_amount: U256,
    pub intermediate_calls: Vec<Address>,
    pub profit: U256,
}

#[derive(Debug, Clone)]
struct LiquidationRisk {
    pub position_value: U256,
    pub collateral_ratio: f64,
    pub health_factor: f64,
    pub liquidation_price: U256,
}

#[derive(Debug, Clone)]
struct AttackSignature {
    pub name: String,
    pub function_selectors: Vec<[u8; 4]>,
    pub gas_pattern: (U256, U256), // min, max gas
    pub value_pattern: (U256, U256), // min, max value
}

#[derive(Debug, Clone)]
struct PositionMonitor {
    positions: HashMap<Address, Position>,
    collateral_ratios: HashMap<Address, f64>,
    liquidation_queue: Vec<Address>,
}

#[derive(Debug, Clone)]
struct Position {
    pub owner: Address,
    pub collateral: U256,
    pub debt: U256,
    pub collateral_token: Address,
    pub debt_token: Address,
    pub last_update: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct RateLimiter {
    transaction_counts: HashMap<Address, Vec<DateTime<Utc>>>,
    value_sums: HashMap<Address, Vec<(DateTime<Utc>, U256)>>,
    cooldowns: HashMap<Address, DateTime<Utc>>,
}

impl DeFiSecurity {
    pub fn new(provider: Arc<Provider<Http>>) -> Self {
        Self {
            provider,
            protocol_configs: Arc::new(RwLock::new(HashMap::new())),
            transaction_history: Arc::new(RwLock::new(HashMap::new())),
            threat_detector: Arc::new(RwLock::new(ThreatDetector {
                flash_loan_patterns: HashMap::new(),
                liquidation_targets: HashMap::new(),
                suspicious_addresses: HashSet::new(),
                attack_signatures: Vec::new(),
            })),
            position_monitor: Arc::new(RwLock::new(PositionMonitor {
                positions: HashMap::new(),
                collateral_ratios: HashMap::new(),
                liquidation_queue: Vec::new(),
            })),
            rate_limiter: Arc::new(RwLock::new(RateLimiter {
                transaction_counts: HashMap::new(),
                value_sums: HashMap::new(),
                cooldowns: HashMap::new(),
            })),
        }
    }

    /// Initialize DeFi security with protocol configurations
    pub async fn initialize(&self) -> Result<()> {
        self.load_attack_signatures().await?;
        self.initialize_protocol_configs().await?;
        
        tracing::info!("DeFi security initialized");
        Ok(())
    }

    /// Register a DeFi protocol for monitoring
    pub async fn register_protocol(&self, config: DeFiProtocolConfig) -> Result<()> {
        let address = config.protocol_address;
        self.protocol_configs.write().await.insert(address, config.clone());
        
        tracing::info!("DeFi protocol registered: {} ({:?})", address, config.protocol_type);
        Ok(())
    }

    /// Analyze transaction for DeFi-specific threats
    pub async fn analyze_defi_transaction(&self, tx: &TransactionRequest) -> Result<Vec<DeFiThreat>> {
        let mut threats = Vec::new();
        
        // Check for flash loan attacks
        if let Some(flash_threat) = self.detect_flash_loan_attack(tx).await? {
            threats.push(flash_threat);
        }
        
        // Check for liquidation attacks
        if let Some(liq_threat) = self.detect_liquidation_attack(tx).await? {
            threats.push(liq_threat);
        }
        
        // Check for governance attacks
        if let Some(gov_threat) = self.detect_governance_attack(tx).await? {
            threats.push(gov_threat);
        }
        
        // Check for price manipulation
        if let Some(price_threat) = self.detect_price_manipulation(tx).await? {
            threats.push(price_threat);
        }
        
        // Check for reentrancy attacks
        if let Some(reentrancy_threat) = self.detect_reentrancy_attack(tx).await? {
            threats.push(reentrancy_threat);
        }
        
        Ok(threats)
    }

    /// Detect flash loan attack patterns
    async fn detect_flash_loan_attack(&self, tx: &TransactionRequest) -> Result<Option<DeFiThreat>> {
        if let Some(to) = &tx.to {
            let to_address = match to {
                NameOrAddress::Address(addr) => *addr,
                NameOrAddress::Name(_) => return Ok(None), // Skip ENS names for now
            };
            
            // Check if interacting with known flash loan providers
            if self.is_flash_loan_provider(to_address).await {
                if let Some(data) = &tx.data {
                    // Analyze the call data for flash loan signatures
                    if self.is_flash_loan_call(data).await {
                        return Ok(Some(DeFiThreat::FlashLoanAttack {
                            loan_amount: tx.value.unwrap_or(U256::zero()),
                            loan_token: to_address, // Simplified
                            attack_vector: AttackVector::PriceOracle, // Would determine from analysis
                        }));
                    }
                }
            }
        }
        Ok(None)
    }

    /// Detect liquidation attack patterns
    async fn detect_liquidation_attack(&self, tx: &TransactionRequest) -> Result<Option<DeFiThreat>> {
        if let Some(data) = &tx.data {
            if data.len() >= 4 {
                let selector = &data[..4];
                
                // Check for liquidation function signatures
                if self.is_liquidation_function(selector).await {
                    let position_monitor = self.position_monitor.read().await;
                    
                    // Check if targeting a vulnerable position
                    if let Some(to) = &tx.to {
                        let to_address = match to {
                            NameOrAddress::Address(addr) => *addr,
                            NameOrAddress::Name(_) => return Ok(None), // Skip ENS names for now
                        };
                        
                        if position_monitor.liquidation_queue.contains(&to_address) {
                            return Ok(Some(DeFiThreat::LiquidationAttack {
                                target_position: to_address,
                                liquidation_amount: tx.value.unwrap_or(U256::zero()),
                                profit_estimation: U256::zero(), // Would calculate actual profit
                            }));
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    /// Detect governance attack patterns
    async fn detect_governance_attack(&self, tx: &TransactionRequest) -> Result<Option<DeFiThreat>> {
        if let Some(data) = &tx.data {
            if data.len() >= 4 {
                let selector = &data[..4];
                
                // Check for governance function signatures
                if self.is_governance_function(selector).await {
                    // Analyze voting power concentration
                    if let Some(from) = tx.from {
                        let to_address = match &tx.to {
                            Some(NameOrAddress::Address(addr)) => *addr,
                            _ => return Ok(None), // Skip if no valid address
                        };
                        
                        let voting_power = self.get_voting_power(from, to_address).await?;
                        
                        // Check for suspicious voting patterns
                        if self.is_suspicious_voting_pattern(from, voting_power).await? {
                            return Ok(Some(DeFiThreat::GovernanceAttack {
                                proposal_id: U256::zero(), // Would extract from call data
                                attack_type: GovernanceAttackType::VotingPowerConcentration,
                            }));
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    /// Detect price manipulation attempts
    async fn detect_price_manipulation(&self, tx: &TransactionRequest) -> Result<Option<DeFiThreat>> {
        if let Some(to) = &tx.to {
            let to_address = match to {
                NameOrAddress::Address(addr) => *addr,
                NameOrAddress::Name(_) => return Ok(None), // Skip ENS names for now
            };
            
            if self.is_dex_contract(to_address).await {
                let value = tx.value.unwrap_or(U256::zero());
                
                // Check for large trades that could impact price
                if value > U256::from(10).pow(U256::from(20)) { // > 100 ETH
                    return Ok(Some(DeFiThreat::PriceManipulation {
                        target_token: to_address, // Simplified
                        manipulation_method: ManipulationMethod::FlashLoanPump,
                    }));
                }
            }
        }
        Ok(None)
    }

    /// Detect reentrancy attack patterns
    async fn detect_reentrancy_attack(&self, tx: &TransactionRequest) -> Result<Option<DeFiThreat>> {
        // This would analyze call stack depth and patterns
        // For now, return None (complex implementation needed)
        Ok(None)
    }

    /// Validate transaction against protocol rules
    pub async fn validate_protocol_interaction(&self, tx: &TransactionRequest) -> Result<bool> {
        if let Some(to) = &tx.to {
            let to_address = match to {
                NameOrAddress::Address(addr) => *addr,
                NameOrAddress::Name(_) => return Ok(true), // Allow ENS names to pass through
            };
            
            let configs = self.protocol_configs.read().await;
            
            if let Some(config) = configs.get(&to_address) {
                // Check if protocol is in emergency pause
                if config.emergency_pause {
                    tracing::warn!("Transaction blocked: protocol {:?} is paused", to_address);
                    return Ok(false);
                }
                
                // Check transaction value limits
                let value = tx.value.unwrap_or(U256::zero());
                if value > config.max_transaction_value {
                    tracing::warn!("Transaction blocked: value {} exceeds limit {}", value, config.max_transaction_value);
                    return Ok(false);
                }
                
                // Check function allowlist
                if let Some(data) = &tx.data {
                    if data.len() >= 4 {
                        let selector = ethers::utils::hex::encode(&data[..4]);
                        if !config.allowed_functions.contains(&selector) && !config.allowed_functions.is_empty() {
                            tracing::warn!("Transaction blocked: function {} not allowed", selector);
                            return Ok(false);
                        }
                    }
                }
                
                // Check rate limits
                if !self.check_rate_limits(tx.from.unwrap_or_default(), value, &config.rate_limits).await? {
                    tracing::warn!("Transaction blocked: rate limit exceeded");
                    return Ok(false);
                }
            }
        }
        
        Ok(true)
    }

    /// Check rate limits for an address
    async fn check_rate_limits(&self, address: Address, value: U256, limits: &RateLimits) -> Result<bool> {
        let mut rate_limiter = self.rate_limiter.write().await;
        let now = Utc::now();
        
        // Check cooldown
        if let Some(&cooldown_until) = rate_limiter.cooldowns.get(&address) {
            if now < cooldown_until {
                return Ok(false);
            }
        }
        
        // Check transaction count limit
        let now = Utc::now();
        let mut should_record = true;
        let mut tx_count_ok = false;
        let mut value_ok = false;
        
        {
            let tx_times = rate_limiter.transaction_counts.entry(address).or_insert_with(Vec::new);
            tx_times.retain(|&time| now.signed_duration_since(time) <= Duration::minutes(1));
            
            if tx_times.len() >= limits.max_transactions_per_minute as usize {
                // Apply cooldown
                rate_limiter.cooldowns.insert(address, now + limits.cooldown_period);
                return Ok(false);
            }
            tx_count_ok = true;
        }
        
        {
            // Check value limit
            let value_history = rate_limiter.value_sums.entry(address).or_insert_with(Vec::new);
            value_history.retain(|(time, _)| now.signed_duration_since(*time) <= Duration::hours(1));
            
            let total_value: U256 = value_history.iter().map(|(_, v)| *v).fold(U256::zero(), |acc, x| acc + x);
            if total_value + value > limits.max_value_per_hour {
                return Ok(false);
            }
            value_ok = true;
        }
        
        // Record transaction if both checks passed
        if tx_count_ok && value_ok {
            rate_limiter.transaction_counts.get_mut(&address).unwrap().push(now);
            rate_limiter.value_sums.get_mut(&address).unwrap().push((now, value));
        }
        
        Ok(true)
    }

    /// Monitor DeFi positions for liquidation risks
    pub async fn monitor_positions(&self) -> Result<()> {
        let mut position_monitor = self.position_monitor.write().await;
        let mut at_risk_positions = Vec::new();
        
        for (address, position) in &position_monitor.positions {
            let health_factor = self.calculate_health_factor(position).await?;
            
            if health_factor < 1.1 { // Liquidation threshold
                at_risk_positions.push(*address);
                tracing::warn!("Position at liquidation risk: {} (health factor: {:.2})", address, health_factor);
            }
        }
        
        position_monitor.liquidation_queue = at_risk_positions;
        Ok(())
    }

    /// Calculate health factor for a position
    async fn calculate_health_factor(&self, position: &Position) -> Result<f64> {
        // Simplified calculation
        let collateral_value = position.collateral.as_u128() as f64;
        let debt_value = position.debt.as_u128() as f64;
        
        if debt_value == 0.0 {
            return Ok(f64::INFINITY);
        }
        
        Ok(collateral_value / debt_value)
    }

    /// Load known attack signatures
    async fn load_attack_signatures(&self) -> Result<()> {
        let mut detector = self.threat_detector.write().await;
        
        // Add known attack signatures
        detector.attack_signatures.push(AttackSignature {
            name: "Flash Loan Arbitrage".to_string(),
            function_selectors: vec![[0xa9, 0x05, 0x9c, 0xbb]], // flashLoan selector
            gas_pattern: (U256::from(200_000), U256::from(2_000_000)),
            value_pattern: (U256::zero(), U256::from(10).pow(U256::from(21))),
        });
        
        Ok(())
    }

    /// Initialize default protocol configurations
    async fn initialize_protocol_configs(&self) -> Result<()> {
        // This would load from configuration files or database
        Ok(())
    }

    /// Helper functions for threat detection
    async fn is_flash_loan_provider(&self, address: Address) -> bool {
        // Known flash loan providers
        let providers = vec![
            "0x398eC7346DcD622eDc5ae82352F02bE94C62d119", // Aave V2
            "0x7d2768dE32b0b80b7a3454c06BdAc94A69DDc7A9", // Aave V2 Pool
        ];
        
        providers.iter().any(|&addr| {
            if let Ok(provider_addr) = addr.parse::<Address>() {
                provider_addr == address
            } else {
                false
            }
        })
    }

    async fn is_flash_loan_call(&self, data: &Bytes) -> bool {
        if data.len() < 4 {
            return false;
        }
        
        let selector = &data[..4];
        matches!(selector, [0xa9, 0x05, 0x9c, 0xbb]) // flashLoan
    }

    async fn is_liquidation_function(&self, selector: &[u8]) -> bool {
        matches!(selector, [0xf5, 0x29, 0x8a, 0xcf]) // liquidateBorrow
    }

    async fn is_governance_function(&self, selector: &[u8]) -> bool {
        matches!(selector, 
            [0x56, 0x78, 0x1d, 0xf0] | // castVote
            [0xda, 0x95, 0x69, 0x1d]   // propose
        )
    }

    async fn is_dex_contract(&self, _address: Address) -> bool {
        // Would check against known DEX addresses
        true
    }

    async fn get_voting_power(&self, _address: Address, _protocol: Address) -> Result<U256> {
        // Would query actual voting power from governance contract
        Ok(U256::zero())
    }

    async fn is_suspicious_voting_pattern(&self, _address: Address, _voting_power: U256) -> Result<bool> {
        // Would analyze voting patterns for suspicious behavior
        Ok(false)
    }

    /// Get DeFi security statistics
    pub async fn get_statistics(&self) -> Result<DeFiSecurityStats> {
        let configs = self.protocol_configs.read().await;
        let history = self.transaction_history.read().await;
        let detector = self.threat_detector.read().await;
        let monitor = self.position_monitor.read().await;
        
        Ok(DeFiSecurityStats {
            monitored_protocols: configs.len(),
            total_transactions_analyzed: history.values().map(|v| v.len()).sum(),
            threats_detected: detector.suspicious_addresses.len(),
            positions_monitored: monitor.positions.len(),
            positions_at_risk: monitor.liquidation_queue.len(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeFiSecurityStats {
    pub monitored_protocols: usize,
    pub total_transactions_analyzed: usize,
    pub threats_detected: usize,
    pub positions_monitored: usize,
    pub positions_at_risk: usize,
}
