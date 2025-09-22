use anyhow::{Result, anyhow};
use ethers::{
    prelude::*,
    types::{Address, U256, TransactionRequest, H256},
};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub overall_risk_score: f64,
    pub risk_factors: Vec<RiskFactor>,
    pub risk_level: RiskLevel,
    pub recommended_actions: Vec<String>,
    pub confidence: f64,
    pub assessed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub factor_type: RiskFactorType,
    pub severity: f64, // 0.0 to 1.0
    pub weight: f64,   // Importance weight
    pub description: String,
    pub mitigation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskFactorType {
    // Market risks
    PriceVolatility,
    LiquidityRisk,
    ImpermanentLoss,
    
    // Technical risks
    SmartContractRisk,
    OracleRisk,
    BridgeRisk,
    
    // Operational risks
    ProtocolRisk,
    GovernanceRisk,
    CustodialRisk,
    
    // Attack risks
    FlashLoanRisk,
    MEVRisk,
    ReentrancyRisk,
    FrontrunningRisk,
    
    // Regulatory risks
    ComplianceRisk,
    JurisdictionRisk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    VeryLow,    // 0.0 - 0.2
    Low,        // 0.2 - 0.4
    Medium,     // 0.4 - 0.6
    High,       // 0.6 - 0.8
    VeryHigh,   // 0.8 - 1.0
}

#[derive(Debug, Clone)]
pub struct RiskModel {
    pub model_name: String,
    pub version: String,
    pub weights: HashMap<RiskFactorType, f64>,
    pub thresholds: HashMap<RiskLevel, f64>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct MarketData {
    pub token_price: U256,
    pub volatility: f64,
    pub liquidity: U256,
    pub trading_volume_24h: U256,
    pub market_cap: U256,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ProtocolMetrics {
    pub tvl: U256,
    pub utilization_rate: f64,
    pub yield_rate: f64,
    pub collateralization_ratio: f64,
    pub liquidation_threshold: f64,
    pub governance_activity: f64,
}

pub struct RiskEngine {
    provider: Arc<Provider<Http>>,
    risk_models: Arc<RwLock<HashMap<String, RiskModel>>>,
    market_data: Arc<RwLock<HashMap<Address, VecDeque<MarketData>>>>,
    protocol_metrics: Arc<RwLock<HashMap<Address, ProtocolMetrics>>>,
    historical_assessments: Arc<RwLock<VecDeque<RiskAssessment>>>,
    risk_calculator: Arc<RwLock<RiskCalculator>>,
    stress_tester: Arc<RwLock<StressTester>>,
}

#[derive(Debug, Clone)]
struct RiskCalculator {
    correlation_matrix: HashMap<(Address, Address), f64>,
    volatility_models: HashMap<Address, VolatilityModel>,
    liquidity_models: HashMap<Address, LiquidityModel>,
}

#[derive(Debug, Clone)]
struct VolatilityModel {
    current_volatility: f64,
    historical_volatility: Vec<f64>,
    volatility_forecast: Vec<f64>,
    confidence_interval: (f64, f64),
}

#[derive(Debug, Clone)]
struct LiquidityModel {
    current_liquidity: U256,
    liquidity_depth: HashMap<f64, U256>, // Price impact -> Available liquidity
    slippage_model: Vec<(U256, f64)>, // Trade size -> Expected slippage
}

#[derive(Debug, Clone)]
struct StressTester {
    stress_scenarios: Vec<StressScenario>,
    scenario_results: HashMap<String, StressTestResult>,
}

#[derive(Debug, Clone)]
struct StressScenario {
    pub name: String,
    pub description: String,
    pub market_shock: f64, // Price shock percentage
    pub liquidity_drain: f64, // Liquidity reduction percentage
    pub correlation_increase: f64, // Correlation increase during stress
    pub duration: Duration,
}

#[derive(Debug, Clone)]
struct StressTestResult {
    pub scenario_name: String,
    pub portfolio_loss: f64,
    pub max_drawdown: f64,
    pub liquidation_probability: f64,
    pub recovery_time: Duration,
}

impl RiskEngine {
    pub fn new(provider: Arc<Provider<Http>>) -> Self {
        Self {
            provider,
            risk_models: Arc::new(RwLock::new(HashMap::new())),
            market_data: Arc::new(RwLock::new(HashMap::new())),
            protocol_metrics: Arc::new(RwLock::new(HashMap::new())),
            historical_assessments: Arc::new(RwLock::new(VecDeque::with_capacity(10000))),
            risk_calculator: Arc::new(RwLock::new(RiskCalculator {
                correlation_matrix: HashMap::new(),
                volatility_models: HashMap::new(),
                liquidity_models: HashMap::new(),
            })),
            stress_tester: Arc::new(RwLock::new(StressTester {
                stress_scenarios: Vec::new(),
                scenario_results: HashMap::new(),
            })),
        }
    }

    /// Initialize the risk engine with default models
    pub async fn initialize(&self) -> Result<()> {
        self.load_default_risk_models().await?;
        self.initialize_stress_scenarios().await?;
        self.start_market_data_collection().await?;
        
        tracing::info!("Risk engine initialized");
        Ok(())
    }

    /// Assess risk for a specific transaction
    pub async fn assess_transaction_risk(&self, tx: &TransactionRequest) -> Result<RiskAssessment> {
        let mut risk_factors = Vec::new();
        
        // Analyze smart contract risks
        if let Some(contract_risk) = self.assess_smart_contract_risk(tx).await? {
            risk_factors.push(contract_risk);
        }
        
        // Analyze market risks
        if let Some(market_risk) = self.assess_market_risk(tx).await? {
            risk_factors.push(market_risk);
        }
        
        // Analyze liquidity risks
        if let Some(liquidity_risk) = self.assess_liquidity_risk(tx).await? {
            risk_factors.push(liquidity_risk);
        }
        
        // Analyze MEV risks
        if let Some(mev_risk) = self.assess_mev_risk(tx).await? {
            risk_factors.push(mev_risk);
        }
        
        // Analyze flash loan risks
        if let Some(flash_loan_risk) = self.assess_flash_loan_risk(tx).await? {
            risk_factors.push(flash_loan_risk);
        }
        
        // Calculate overall risk score
        let overall_risk_score = self.calculate_overall_risk_score(&risk_factors).await?;
        let risk_level = self.determine_risk_level(overall_risk_score);
        let recommended_actions = self.generate_risk_recommendations(&risk_factors, overall_risk_score).await?;
        
        let assessment = RiskAssessment {
            overall_risk_score,
            risk_factors,
            risk_level,
            recommended_actions,
            confidence: 0.85, // Would calculate based on data quality
            assessed_at: Utc::now(),
        };
        
        // Store assessment for future analysis
        let mut history = self.historical_assessments.write().await;
        history.push_back(assessment.clone());
        if history.len() > 10000 {
            history.pop_front();
        }
        
        Ok(assessment)
    }

    /// Assess portfolio-level risks
    pub async fn assess_portfolio_risk(&self, positions: &[PortfolioPosition]) -> Result<RiskAssessment> {
        let mut risk_factors = Vec::new();
        
        // Calculate concentration risk
        risk_factors.push(self.assess_concentration_risk(positions).await?);
        
        // Calculate correlation risk
        risk_factors.push(self.assess_correlation_risk(positions).await?);
        
        // Calculate liquidation risk
        risk_factors.push(self.assess_liquidation_risk(positions).await?);
        
        // Calculate impermanent loss risk
        risk_factors.push(self.assess_impermanent_loss_risk(positions).await?);
        
        let overall_risk_score = self.calculate_overall_risk_score(&risk_factors).await?;
        let risk_level = self.determine_risk_level(overall_risk_score);
        let recommended_actions = self.generate_portfolio_recommendations(&risk_factors, positions).await?;
        
        Ok(RiskAssessment {
            overall_risk_score,
            risk_factors,
            risk_level,
            recommended_actions,
            confidence: 0.80,
            assessed_at: Utc::now(),
        })
    }

    /// Perform stress testing
    pub async fn run_stress_tests(&self, positions: &[PortfolioPosition]) -> Result<Vec<StressTestResult>> {
        let stress_tester = self.stress_tester.read().await;
        let mut results = Vec::new();
        
        for scenario in &stress_tester.stress_scenarios {
            let result = self.simulate_stress_scenario(scenario, positions).await?;
            results.push(result);
        }
        
        Ok(results)
    }

    /// Assess smart contract risks
    async fn assess_smart_contract_risk(&self, tx: &TransactionRequest) -> Result<Option<RiskFactor>> {
        if let Some(to) = &tx.to {
            let to_address = match to {
                NameOrAddress::Address(addr) => *addr,
                NameOrAddress::Name(_) => return Ok(None), // Skip ENS names
            };
            
            // Check if contract is verified
            let is_verified = self.is_contract_verified(to_address).await?;
            let audit_status = self.get_audit_status(to_address).await?;
            
            let severity = match (is_verified, audit_status.as_str()) {
                (true, "audited") => 0.1,
                (true, "partial") => 0.3,
                (true, "none") => 0.5,
                (true, _) => 0.4, // Unknown audit status but verified
                (false, _) => 0.8,
            };
            
            return Ok(Some(RiskFactor {
                factor_type: RiskFactorType::SmartContractRisk,
                severity,
                weight: 0.8, // High importance
                description: format!("Contract verification: {}, Audit status: {}", is_verified, audit_status),
                mitigation: Some("Use only verified and audited contracts".to_string()),
            }));
        }
        
        Ok(None)
    }

    /// Assess market risks
    async fn assess_market_risk(&self, tx: &TransactionRequest) -> Result<Option<RiskFactor>> {
        if let Some(to) = &tx.to {
            let to_address = match to {
                NameOrAddress::Address(addr) => *addr,
                NameOrAddress::Name(_) => return Ok(None), // Skip ENS names
            };
            
            let market_data = self.market_data.read().await;
            
            if let Some(data_queue) = market_data.get(&to_address) {
                if let Some(latest_data) = data_queue.back() {
                    let volatility = latest_data.volatility;
                    
                    let severity = match volatility {
                        v if v < 0.1 => 0.1,
                        v if v < 0.2 => 0.3,
                        v if v < 0.4 => 0.5,
                        v if v < 0.6 => 0.7,
                        _ => 0.9,
                    };
                    
                    return Ok(Some(RiskFactor {
                        factor_type: RiskFactorType::PriceVolatility,
                        severity,
                        weight: 0.6,
                        description: format!("Current volatility: {:.2}%", volatility * 100.0),
                        mitigation: Some("Consider position sizing and stop losses".to_string()),
                    }));
                }
            }
        }
        
        Ok(None)
    }

    /// Assess liquidity risks
    async fn assess_liquidity_risk(&self, tx: &TransactionRequest) -> Result<Option<RiskFactor>> {
        if let Some(to) = &tx.to {
            let to_address = match to {
                NameOrAddress::Address(addr) => *addr,
                NameOrAddress::Name(_) => return Ok(None), // Skip ENS names
            };
            
            let value = tx.value.unwrap_or(U256::zero());
            
            // Get current liquidity
            let liquidity = self.get_contract_liquidity(to_address).await?;
            
            if liquidity > U256::zero() {
                let impact_ratio = value.as_u128() as f64 / liquidity.as_u128() as f64;
                
                let severity = match impact_ratio {
                    r if r < 0.01 => 0.1,
                    r if r < 0.05 => 0.3,
                    r if r < 0.1 => 0.5,
                    r if r < 0.2 => 0.7,
                    _ => 0.9,
                };
                
                return Ok(Some(RiskFactor {
                    factor_type: RiskFactorType::LiquidityRisk,
                    severity,
                    weight: 0.7,
                    description: format!("Transaction impact: {:.2}% of available liquidity", impact_ratio * 100.0),
                    mitigation: Some("Consider splitting large transactions".to_string()),
                }));
            }
        }
        
        Ok(None)
    }

    /// Assess MEV risks
    async fn assess_mev_risk(&self, tx: &TransactionRequest) -> Result<Option<RiskFactor>> {
        let gas_price = tx.gas_price.unwrap_or(U256::zero());
        let market_gas_price = self.provider.get_gas_price().await?;
        
        let gas_premium = if market_gas_price > U256::zero() {
            (gas_price.as_u128() as f64) / (market_gas_price.as_u128() as f64) - 1.0
        } else {
            0.0
        };
        
        let severity = match gas_premium {
            p if p < 0.1 => 0.1,
            p if p < 0.3 => 0.4,
            p if p < 0.5 => 0.6,
            _ => 0.8,
        };
        
        Ok(Some(RiskFactor {
            factor_type: RiskFactorType::MEVRisk,
            severity,
            weight: 0.5,
            description: format!("Gas price premium: {:.1}%", gas_premium * 100.0),
            mitigation: Some("Use private mempools or commit-reveal schemes".to_string()),
        }))
    }

    /// Assess flash loan risks
    async fn assess_flash_loan_risk(&self, tx: &TransactionRequest) -> Result<Option<RiskFactor>> {
        if let Some(data) = &tx.data {
            if self.contains_flash_loan_pattern(data).await {
                return Ok(Some(RiskFactor {
                    factor_type: RiskFactorType::FlashLoanRisk,
                    severity: 0.7,
                    weight: 0.8,
                    description: "Transaction contains flash loan patterns".to_string(),
                    mitigation: Some("Ensure flash loan is properly secured and tested".to_string()),
                }));
            }
        }
        
        Ok(None)
    }

    /// Assess concentration risk in portfolio
    async fn assess_concentration_risk(&self, positions: &[PortfolioPosition]) -> Result<RiskFactor> {
        let total_value: f64 = positions.iter().map(|p| p.value_usd).sum();
        
        // Find largest position
        let max_position_value = positions.iter()
            .map(|p| p.value_usd)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        
        let concentration_ratio = if total_value > 0.0 {
            max_position_value / total_value
        } else {
            0.0
        };
        
        let severity = match concentration_ratio {
            r if r < 0.2 => 0.1,
            r if r < 0.4 => 0.3,
            r if r < 0.6 => 0.5,
            r if r < 0.8 => 0.7,
            _ => 0.9,
        };
        
        Ok(RiskFactor {
            factor_type: RiskFactorType::ProtocolRisk,
            severity,
            weight: 0.6,
            description: format!("Largest position represents {:.1}% of portfolio", concentration_ratio * 100.0),
            mitigation: Some("Diversify across multiple protocols and assets".to_string()),
        })
    }

    /// Assess correlation risk
    async fn assess_correlation_risk(&self, positions: &[PortfolioPosition]) -> Result<RiskFactor> {
        let calculator = self.risk_calculator.read().await;
        let mut avg_correlation = 0.0;
        let mut correlation_count = 0;
        
        // Calculate average correlation between positions
        for (i, pos1) in positions.iter().enumerate() {
            for pos2 in positions.iter().skip(i + 1) {
                if let Some(&correlation) = calculator.correlation_matrix.get(&(pos1.token_address, pos2.token_address)) {
                    avg_correlation += correlation.abs();
                    correlation_count += 1;
                }
            }
        }
        
        if correlation_count > 0 {
            avg_correlation /= correlation_count as f64;
        }
        
        let severity = match avg_correlation {
            c if c < 0.2 => 0.1,
            c if c < 0.4 => 0.3,
            c if c < 0.6 => 0.5,
            c if c < 0.8 => 0.7,
            _ => 0.9,
        };
        
        Ok(RiskFactor {
            factor_type: RiskFactorType::PriceVolatility,
            severity,
            weight: 0.5,
            description: format!("Average correlation between positions: {:.2}", avg_correlation),
            mitigation: Some("Reduce correlation by diversifying across uncorrelated assets".to_string()),
        })
    }

    /// Assess liquidation risk
    async fn assess_liquidation_risk(&self, positions: &[PortfolioPosition]) -> Result<RiskFactor> {
        let mut min_health_factor = f64::INFINITY;
        let mut positions_at_risk = 0;
        
        for position in positions {
            if position.is_leveraged {
                let health_factor = position.collateral_value / position.debt_value.max(1.0);
                min_health_factor = min_health_factor.min(health_factor);
                
                if health_factor < 1.5 {
                    positions_at_risk += 1;
                }
            }
        }
        
        let severity = if min_health_factor == f64::INFINITY {
            0.0 // No leveraged positions
        } else {
            match min_health_factor {
                h if h > 2.0 => 0.1,
                h if h > 1.5 => 0.3,
                h if h > 1.2 => 0.6,
                h if h > 1.0 => 0.8,
                _ => 1.0,
            }
        };
        
        Ok(RiskFactor {
            factor_type: RiskFactorType::LiquidityRisk,
            severity,
            weight: 0.9, // Very important
            description: format!("Minimum health factor: {:.2}, Positions at risk: {}", min_health_factor, positions_at_risk),
            mitigation: Some("Increase collateral or reduce debt to improve health factors".to_string()),
        })
    }

    /// Assess impermanent loss risk
    async fn assess_impermanent_loss_risk(&self, positions: &[PortfolioPosition]) -> Result<RiskFactor> {
        let mut max_il_exposure: f64 = 0.0;
        
        for position in positions {
            if position.position_type == "LP" { // Liquidity provider position
                // Calculate potential impermanent loss based on price correlation
                let il_risk = self.calculate_impermanent_loss_risk(position).await?;
                max_il_exposure = max_il_exposure.max(il_risk);
            }
        }
        
        let severity = match max_il_exposure {
            r if r < 0.05 => 0.1,
            r if r < 0.1 => 0.3,
            r if r < 0.2 => 0.5,
            r if r < 0.3 => 0.7,
            _ => 0.9,
        };
        
        Ok(RiskFactor {
            factor_type: RiskFactorType::ImpermanentLoss,
            severity,
            weight: 0.4,
            description: format!("Maximum impermanent loss exposure: {:.1}%", max_il_exposure * 100.0),
            mitigation: Some("Consider single-sided staking or correlated asset pairs".to_string()),
        })
    }

    /// Calculate overall risk score
    async fn calculate_overall_risk_score(&self, risk_factors: &[RiskFactor]) -> Result<f64> {
        if risk_factors.is_empty() {
            return Ok(0.0);
        }
        
        let weighted_sum: f64 = risk_factors.iter()
            .map(|factor| factor.severity * factor.weight)
            .sum();
        
        let total_weight: f64 = risk_factors.iter().map(|factor| factor.weight).sum();
        
        if total_weight > 0.0 {
            Ok((weighted_sum / total_weight).min(1.0))
        } else {
            Ok(0.0)
        }
    }

    /// Determine risk level from score
    fn determine_risk_level(&self, score: f64) -> RiskLevel {
        match score {
            s if s < 0.2 => RiskLevel::VeryLow,
            s if s < 0.4 => RiskLevel::Low,
            s if s < 0.6 => RiskLevel::Medium,
            s if s < 0.8 => RiskLevel::High,
            _ => RiskLevel::VeryHigh,
        }
    }

    /// Generate risk recommendations
    async fn generate_risk_recommendations(&self, risk_factors: &[RiskFactor], overall_score: f64) -> Result<Vec<String>> {
        let mut recommendations = Vec::new();
        
        // Add specific recommendations based on risk factors
        for factor in risk_factors {
            if factor.severity > 0.6 {
                if let Some(mitigation) = &factor.mitigation {
                    recommendations.push(mitigation.clone());
                }
            }
        }
        
        // Add general recommendations based on overall score
        match self.determine_risk_level(overall_score) {
            RiskLevel::VeryHigh => {
                recommendations.push("URGENT: Consider exiting positions or reducing exposure immediately".to_string());
                recommendations.push("Enable emergency stop mechanisms".to_string());
            }
            RiskLevel::High => {
                recommendations.push("Reduce position sizes and increase monitoring frequency".to_string());
                recommendations.push("Consider hedging strategies".to_string());
            }
            RiskLevel::Medium => {
                recommendations.push("Monitor positions closely and set up alerts".to_string());
                recommendations.push("Review risk management parameters".to_string());
            }
            _ => {}
        }
        
        Ok(recommendations)
    }

    /// Generate portfolio-specific recommendations
    async fn generate_portfolio_recommendations(&self, risk_factors: &[RiskFactor], positions: &[PortfolioPosition]) -> Result<Vec<String>> {
        let mut recommendations = Vec::new();
        
        // Analyze portfolio composition
        let total_value: f64 = positions.iter().map(|p| p.value_usd).sum();
        let leveraged_positions = positions.iter().filter(|p| p.is_leveraged).count();
        
        if leveraged_positions > positions.len() / 2 {
            recommendations.push("Consider reducing leverage across the portfolio".to_string());
        }
        
        // Check for over-concentration
        for factor in risk_factors {
            if matches!(factor.factor_type, RiskFactorType::ProtocolRisk) && factor.severity > 0.6 {
                recommendations.push("Diversify across more protocols and asset classes".to_string());
                break;
            }
        }
        
        Ok(recommendations)
    }

    /// Helper functions
    async fn is_contract_verified(&self, _address: Address) -> Result<bool> {
        // Would check against Etherscan or similar
        Ok(true)
    }

    async fn get_audit_status(&self, _address: Address) -> Result<String> {
        // Would check audit databases
        Ok("audited".to_string())
    }

    async fn get_contract_liquidity(&self, _address: Address) -> Result<U256> {
        // Would query DEX liquidity
        Ok(U256::from(1000000)) // Placeholder
    }

    async fn contains_flash_loan_pattern(&self, _data: &ethers::types::Bytes) -> bool {
        // Would analyze call data for flash loan patterns
        false
    }

    async fn calculate_impermanent_loss_risk(&self, _position: &PortfolioPosition) -> Result<f64> {
        // Would calculate IL risk based on asset correlation
        Ok(0.1)
    }

    async fn simulate_stress_scenario(&self, _scenario: &StressScenario, _positions: &[PortfolioPosition]) -> Result<StressTestResult> {
        // Would run Monte Carlo simulation
        Ok(StressTestResult {
            scenario_name: "test".to_string(),
            portfolio_loss: 0.1,
            max_drawdown: 0.15,
            liquidation_probability: 0.05,
            recovery_time: Duration::days(7),
        })
    }

    async fn load_default_risk_models(&self) -> Result<()> {
        // Load risk models from configuration
        Ok(())
    }

    async fn initialize_stress_scenarios(&self) -> Result<()> {
        // Initialize stress test scenarios
        Ok(())
    }

    async fn start_market_data_collection(&self) -> Result<()> {
        // Start background market data collection
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PortfolioPosition {
    pub token_address: Address,
    pub position_type: String, // "long", "short", "LP", etc.
    pub value_usd: f64,
    pub is_leveraged: bool,
    pub collateral_value: f64,
    pub debt_value: f64,
}

impl RiskLevel {
    pub fn to_string(&self) -> &'static str {
        match self {
            RiskLevel::VeryLow => "Very Low",
            RiskLevel::Low => "Low", 
            RiskLevel::Medium => "Medium",
            RiskLevel::High => "High",
            RiskLevel::VeryHigh => "Very High",
        }
    }
}
