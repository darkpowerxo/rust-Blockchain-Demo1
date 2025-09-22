use anyhow::{Result, anyhow};
use ethers::{
    prelude::*,
    types::{Address, U256, H256, Bytes},
};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OracleType {
    Chainlink,
    Uniswap,
    Band,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    pub price: U256,
    pub timestamp: DateTime<Utc>,
    pub confidence: f64,
    pub source: OracleType,
    pub block_number: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleAnomaly {
    pub anomaly_type: AnomalyType,
    pub severity: Severity,
    pub detected_at: DateTime<Utc>,
    pub oracle_address: Address,
    pub expected_price: U256,
    pub actual_price: U256,
    pub deviation_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyType {
    PriceManipulation,
    FlashLoanAttack,
    StalePrice,
    ExtremeVolatility,
    CircuitBreaker,
    Outlier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct OracleConfig {
    pub address: Address,
    pub oracle_type: OracleType,
    pub max_deviation: f64,
    pub max_staleness: Duration,
    pub min_confirmations: u8,
    pub circuit_breaker_threshold: f64,
    pub aggregation_method: AggregationMethod,
}

#[derive(Debug, Clone)]
pub enum AggregationMethod {
    Median,
    Mean,
    WeightedAverage(Vec<f64>),
    TrimmedMean(f64), // percentage to trim from each end
}

pub struct OracleSecurity {
    provider: Arc<Provider<Http>>,
    oracle_configs: Arc<RwLock<HashMap<Address, OracleConfig>>>,
    price_history: Arc<RwLock<HashMap<Address, VecDeque<PriceData>>>>,
    anomaly_detector: Arc<RwLock<AnomalyDetector>>,
    reference_prices: Arc<RwLock<HashMap<Address, Vec<PriceData>>>>,
    circuit_breakers: Arc<RwLock<HashMap<Address, CircuitBreaker>>>,
}

#[derive(Debug, Clone)]
struct AnomalyDetector {
    price_windows: HashMap<Address, VecDeque<U256>>,
    volatility_thresholds: HashMap<Address, f64>,
    correlation_matrix: HashMap<(Address, Address), f64>,
}

#[derive(Debug, Clone)]
struct CircuitBreaker {
    triggered: bool,
    trigger_time: Option<DateTime<Utc>>,
    threshold: f64,
    cooldown_period: Duration,
    last_price: U256,
}

impl OracleSecurity {
    pub fn new(provider: Arc<Provider<Http>>) -> Self {
        Self {
            provider,
            oracle_configs: Arc::new(RwLock::new(HashMap::new())),
            price_history: Arc::new(RwLock::new(HashMap::new())),
            anomaly_detector: Arc::new(RwLock::new(AnomalyDetector {
                price_windows: HashMap::new(),
                volatility_thresholds: HashMap::new(),
                correlation_matrix: HashMap::new(),
            })),
            reference_prices: Arc::new(RwLock::new(HashMap::new())),
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register an oracle for monitoring
    pub async fn register_oracle(&self, config: OracleConfig) -> Result<()> {
        let address = config.address;
        
        // Initialize price history
        self.price_history.write().await.insert(address, VecDeque::with_capacity(1000));
        
        // Initialize circuit breaker
        self.circuit_breakers.write().await.insert(address, CircuitBreaker {
            triggered: false,
            trigger_time: None,
            threshold: config.circuit_breaker_threshold,
            cooldown_period: Duration::minutes(10),
            last_price: U256::zero(),
        });
        
        // Store configuration
        self.oracle_configs.write().await.insert(address, config.clone());
        
        tracing::info!("Oracle registered: {:?} ({})", config.oracle_type, address);
        Ok(())
    }

    /// Validate price data from an oracle
    pub async fn validate_price(&self, oracle_address: Address, price: U256) -> Result<bool> {
        let configs = self.oracle_configs.read().await;
        let config = configs.get(&oracle_address)
            .ok_or_else(|| anyhow!("Oracle not registered: {}", oracle_address))?;

        // Check circuit breaker status
        if self.is_circuit_breaker_triggered(oracle_address).await? {
            return Ok(false);
        }

        // Get reference prices for comparison
        let reference_prices = self.get_reference_prices(oracle_address).await?;
        
        // Perform validation checks
        let mut validations = Vec::new();
        
        // 1. Deviation check
        validations.push(self.check_price_deviation(price, &reference_prices, config.max_deviation).await?);
        
        // 2. Staleness check  
        validations.push(self.check_price_staleness(oracle_address, config.max_staleness).await?);
        
        // 3. Volatility check
        validations.push(self.check_volatility(oracle_address, price).await?);
        
        // 4. Flash loan attack detection
        validations.push(self.detect_flash_loan_attack(oracle_address, price).await?);
        
        // 5. Correlation analysis
        validations.push(self.analyze_price_correlation(oracle_address, price).await?);

        let is_valid = validations.iter().all(|&v| v);
        
        if !is_valid {
            self.handle_invalid_price(oracle_address, price, &reference_prices).await?;
        } else {
            self.record_price_data(oracle_address, price).await?;
        }

        Ok(is_valid)
    }

    /// Check if price deviates too much from reference prices
    async fn check_price_deviation(&self, price: U256, reference_prices: &[PriceData], max_deviation: f64) -> Result<bool> {
        if reference_prices.is_empty() {
            return Ok(true); // No reference data yet
        }

        let aggregated_price = self.aggregate_prices(reference_prices, &AggregationMethod::Median).await?;
        let deviation = self.calculate_deviation(price, aggregated_price);
        
        Ok(deviation <= max_deviation)
    }

    /// Check if price data is stale
    async fn check_price_staleness(&self, oracle_address: Address, max_staleness: Duration) -> Result<bool> {
        let history = self.price_history.read().await;
        
        if let Some(price_history) = history.get(&oracle_address) {
            if let Some(last_price) = price_history.back() {
                let time_diff = Utc::now().signed_duration_since(last_price.timestamp);
                return Ok(time_diff <= max_staleness);
            }
        }
        
        Ok(true) // No history yet, assume fresh
    }

    /// Check for extreme volatility
    async fn check_volatility(&self, oracle_address: Address, price: U256) -> Result<bool> {
        let history = self.price_history.read().await;
        
        if let Some(price_history) = history.get(&oracle_address) {
            if price_history.len() < 10 {
                return Ok(true); // Not enough data
            }

            // Calculate recent volatility
            let recent_prices: Vec<U256> = price_history
                .iter()
                .rev()
                .take(10)
                .map(|p| p.price)
                .collect();
            
            let volatility = self.calculate_volatility(&recent_prices);
            let threshold = 0.1; // 10% volatility threshold
            
            // Check if current price would cause excessive volatility
            let mut test_prices = recent_prices;
            test_prices.push(price);
            let new_volatility = self.calculate_volatility(&test_prices);
            
            Ok(new_volatility - volatility <= threshold)
        } else {
            Ok(true)
        }
    }

    /// Detect potential flash loan price manipulation attacks
    async fn detect_flash_loan_attack(&self, oracle_address: Address, price: U256) -> Result<bool> {
        // Get recent transaction history
        let current_block = self.provider.get_block_number().await?;
        let recent_blocks = 5; // Check last 5 blocks
        
        for i in 0..recent_blocks {
            if let Some(block_num) = current_block.checked_sub(U64::from(i)) {
                if let Ok(Some(block)) = self.provider.get_block(block_num).await {
                    // Check for large flash loan transactions
                    for tx_hash in &block.transactions {
                        if let Ok(Some(tx)) = self.provider.get_transaction(*tx_hash).await {
                            if self.is_potential_flash_loan(&tx).await {
                                // If flash loan detected, be more conservative
                                return self.validate_against_multiple_sources(oracle_address, price).await;
                            }
                        }
                    }
                }
            }
        }
        
        Ok(true)
    }

    /// Analyze price correlation with other oracles
    async fn analyze_price_correlation(&self, oracle_address: Address, price: U256) -> Result<bool> {
        let configs = self.oracle_configs.read().await;
        let mut correlations = Vec::new();
        
        // Compare with other oracles
        for (&other_address, _) in configs.iter() {
            if other_address != oracle_address {
                if let Some(correlation) = self.get_price_correlation(oracle_address, other_address, price).await? {
                    correlations.push(correlation);
                }
            }
        }
        
        if correlations.is_empty() {
            return Ok(true); // No other oracles to compare
        }
        
        // Check if price is correlated with majority of other oracles
        let correlated_count = correlations.iter().filter(|&&c| c > 0.7).count();
        let correlation_ratio = correlated_count as f64 / correlations.len() as f64;
        
        Ok(correlation_ratio >= 0.5) // At least 50% correlation
    }

    /// Handle invalid price detection
    async fn handle_invalid_price(&self, oracle_address: Address, price: U256, reference_prices: &[PriceData]) -> Result<()> {
        let aggregated_price = self.aggregate_prices(reference_prices, &AggregationMethod::Median).await?;
        let deviation = self.calculate_deviation(price, aggregated_price);
        
        let anomaly = OracleAnomaly {
            anomaly_type: if deviation > 0.5 {
                AnomalyType::PriceManipulation
            } else {
                AnomalyType::Outlier
            },
            severity: match deviation {
                d if d > 0.5 => Severity::Critical,
                d if d > 0.2 => Severity::High,
                d if d > 0.1 => Severity::Medium,
                _ => Severity::Low,
            },
            detected_at: Utc::now(),
            oracle_address,
            expected_price: aggregated_price,
            actual_price: price,
            deviation_percentage: deviation * 100.0,
        };
        
        // Trigger circuit breaker if severe
        if matches!(anomaly.severity, Severity::High | Severity::Critical) {
            self.trigger_circuit_breaker(oracle_address).await?;
        }
        
        tracing::warn!("Oracle anomaly detected: {:?}", anomaly);
        Ok(())
    }

    /// Trigger circuit breaker for an oracle
    async fn trigger_circuit_breaker(&self, oracle_address: Address) -> Result<()> {
        let mut breakers = self.circuit_breakers.write().await;
        
        if let Some(breaker) = breakers.get_mut(&oracle_address) {
            breaker.triggered = true;
            breaker.trigger_time = Some(Utc::now());
            tracing::error!("Circuit breaker triggered for oracle: {}", oracle_address);
        }
        
        Ok(())
    }

    /// Check if circuit breaker is triggered
    async fn is_circuit_breaker_triggered(&self, oracle_address: Address) -> Result<bool> {
        let mut breakers = self.circuit_breakers.write().await;
        
        if let Some(breaker) = breakers.get_mut(&oracle_address) {
            if breaker.triggered {
                if let Some(trigger_time) = breaker.trigger_time {
                    // Check if cooldown period has passed
                    if Utc::now().signed_duration_since(trigger_time) >= breaker.cooldown_period {
                        breaker.triggered = false;
                        breaker.trigger_time = None;
                        tracing::info!("Circuit breaker reset for oracle: {}", oracle_address);
                        return Ok(false);
                    }
                }
                return Ok(true);
            }
        }
        
        Ok(false)
    }

    /// Get reference prices from multiple sources
    async fn get_reference_prices(&self, oracle_address: Address) -> Result<Vec<PriceData>> {
        let reference_prices = self.reference_prices.read().await;
        
        if let Some(prices) = reference_prices.get(&oracle_address) {
            // Return recent prices (last 10 minutes)
            let cutoff = Utc::now() - Duration::minutes(10);
            Ok(prices.iter()
                .filter(|p| p.timestamp >= cutoff)
                .cloned()
                .collect())
        } else {
            Ok(Vec::new())
        }
    }

    /// Aggregate multiple prices using specified method
    async fn aggregate_prices(&self, prices: &[PriceData], method: &AggregationMethod) -> Result<U256> {
        if prices.is_empty() {
            return Ok(U256::zero());
        }
        
        let mut price_values: Vec<U256> = prices.iter().map(|p| p.price).collect();
        price_values.sort();
        
        match method {
            AggregationMethod::Median => {
                let mid = price_values.len() / 2;
                Ok(if price_values.len() % 2 == 0 {
                    (price_values[mid - 1] + price_values[mid]) / 2
                } else {
                    price_values[mid]
                })
            }
            AggregationMethod::Mean => {
                let sum: U256 = price_values.iter().fold(U256::zero(), |acc, x| acc + x);
                Ok(sum / U256::from(price_values.len()))
            }
            AggregationMethod::WeightedAverage(weights) => {
                if weights.len() != prices.len() {
                    return Ok(price_values[0]); // Fallback to first price
                }
                
                let mut weighted_sum = U256::zero();
                let mut weight_sum = 0.0;
                
                for (price, &weight) in prices.iter().zip(weights.iter()) {
                    weighted_sum += price.price * U256::from((weight * 1000.0) as u64);
                    weight_sum += weight;
                }
                
                Ok(weighted_sum / U256::from((weight_sum * 1000.0) as u64))
            }
            AggregationMethod::TrimmedMean(trim_percentage) => {
                let trim_count = (price_values.len() as f64 * trim_percentage) as usize;
                if trim_count * 2 >= price_values.len() {
                    return Ok(price_values[price_values.len() / 2]); // Fallback to median
                }
                
                let trimmed: Vec<U256> = price_values.iter()
                    .skip(trim_count)
                    .take(price_values.len() - 2 * trim_count)
                    .cloned()
                    .collect();
                
                let sum: U256 = trimmed.iter().fold(U256::zero(), |acc, x| acc + x);
                Ok(sum / U256::from(trimmed.len()))
            }
        }
    }

    /// Calculate price deviation percentage
    fn calculate_deviation(&self, price1: U256, price2: U256) -> f64 {
        if price2.is_zero() {
            return 1.0; // 100% deviation if reference is zero
        }
        
        let diff = if price1 > price2 {
            price1 - price2
        } else {
            price2 - price1
        };
        
        (diff.as_u128() as f64) / (price2.as_u128() as f64)
    }

    /// Calculate price volatility
    fn calculate_volatility(&self, prices: &[U256]) -> f64 {
        if prices.len() < 2 {
            return 0.0;
        }
        
        let prices_f64: Vec<f64> = prices.iter().map(|p| p.as_u128() as f64).collect();
        let mean = prices_f64.iter().sum::<f64>() / prices_f64.len() as f64;
        
        let variance = prices_f64.iter()
            .map(|p| (p - mean).powi(2))
            .sum::<f64>() / prices_f64.len() as f64;
        
        variance.sqrt() / mean // Coefficient of variation
    }

    /// Check if transaction could be a flash loan
    async fn is_potential_flash_loan(&self, tx: &Transaction) -> bool {
        // Check for high value transfers or DEX interactions
        if tx.value > U256::from(10).pow(U256::from(21)) { // > 1000 ETH
            return true;
        }
        
        // Check if interacting with known flash loan providers
        // This would contain known flash loan contract addresses
        let flash_loan_contracts = vec![
            "0x398eC7346DcD622eDc5ae82352F02bE94C62d119", // Aave V2
            "0x7d2768dE32b0b80b7a3454c06BdAc94A69DDc7A9", // Aave V2 Pool
        ];
        
        if let Some(to) = tx.to {
            return flash_loan_contracts.iter().any(|&addr| {
                if let Ok(flash_addr) = addr.parse::<Address>() {
                    flash_addr == to
                } else {
                    false
                }
            });
        }
        
        false
    }

    /// Validate price against multiple external sources
    async fn validate_against_multiple_sources(&self, _oracle_address: Address, _price: U256) -> Result<bool> {
        // This would query multiple external price sources
        // For now, return true (would implement actual validation)
        Ok(true)
    }

    /// Get price correlation between two oracles
    async fn get_price_correlation(&self, _oracle1: Address, _oracle2: Address, _current_price: U256) -> Result<Option<f64>> {
        // This would calculate correlation coefficient
        // For now, return a placeholder
        Ok(Some(0.8))
    }

    /// Record price data for analysis
    async fn record_price_data(&self, oracle_address: Address, price: U256) -> Result<()> {
        let price_data = PriceData {
            price,
            timestamp: Utc::now(),
            confidence: 1.0,
            source: OracleType::Custom("internal".to_string()),
            block_number: self.provider.get_block_number().await?.as_u64(),
        };
        
        let mut history = self.price_history.write().await;
        if let Some(oracle_history) = history.get_mut(&oracle_address) {
            oracle_history.push_back(price_data);
            
            // Keep only recent data (last 1000 entries)
            while oracle_history.len() > 1000 {
                oracle_history.pop_front();
            }
        }
        
        Ok(())
    }

    /// Get oracle security statistics
    pub async fn get_statistics(&self) -> Result<OracleSecurityStats> {
        let configs = self.oracle_configs.read().await;
        let history = self.price_history.read().await;
        let breakers = self.circuit_breakers.read().await;
        
        let active_breakers = breakers.values().filter(|b| b.triggered).count();
        let total_price_points: usize = history.values().map(|h| h.len()).sum();
        
        Ok(OracleSecurityStats {
            registered_oracles: configs.len(),
            active_circuit_breakers: active_breakers,
            total_price_validations: total_price_points,
            anomalies_detected: 0, // Would track this in real implementation
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OracleSecurityStats {
    pub registered_oracles: usize,
    pub active_circuit_breakers: usize,
    pub total_price_validations: usize,
    pub anomalies_detected: usize,
}
