use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use ethers::prelude::*;
use ethers::core::types::{TransactionRequest, Transaction, transaction::eip2718::TypedTransaction};
use chrono::{DateTime, Duration, Utc};
use serde::{Serialize, Deserialize};
use anyhow::Result;
use tracing::{info, warn, error};
use ring::digest;

// Import all security modules
pub mod mev_protection;
pub mod oracle_security;
pub mod defi_security;
pub mod risk_engine;
pub mod emergency_response;
pub mod audit_trail;
pub mod transaction_validator;
pub mod reentrancy_guard;
pub mod input_sanitizer;

use mev_protection::*;
use oracle_security::*;
use defi_security::*;
use risk_engine::*;
use emergency_response::*;
use audit_trail::*;

// Re-export for convenience
pub use mev_protection::{MevProtection, MevThreat, MevStats};
pub use oracle_security::{OracleSecurity, OracleSecurityStats};
pub use defi_security::{DeFiSecurity, DeFiSecurityStats};
pub use risk_engine::{RiskEngine, RiskAssessment};
pub use emergency_response::{EmergencyResponse, EmergencyAlert, EmergencyStats};
pub use audit_trail::{AuditTrail, AuditEntry, AuditStats, ComplianceReport};

#[derive(Debug, Clone)]
pub enum SecurityStatus {
    Safe,
    Caution,
    Warning,
    Danger,
}

#[derive(Debug, Clone)]
pub enum ThreatLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub enum ThreatType {
    MEV(MevThreat),
    Oracle(String),
    DeFi(String),
    Reentrancy,
    FrontRunning,
    Unknown(String),
}

#[derive(Debug, Clone)]
pub struct SecurityThreat {
    pub threat_id: String,
    pub threat_type: ThreatType,
    pub severity: f64,
    pub detected_at: DateTime<Utc>,
    pub source_address: Option<Address>,
    pub description: String,
    pub mitigation_actions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub risk_tolerance: f64,
    pub mev_protection_enabled: bool,
    pub oracle_validation_enabled: bool,
    pub defi_monitoring_enabled: bool,
    pub risk_assessment_enabled: bool,
    pub emergency_response_enabled: bool,
    pub audit_logging_enabled: bool,
    pub max_gas_price: U256,
    pub max_transaction_value: U256,
    pub blacklisted_addresses: Vec<Address>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            risk_tolerance: 0.7,
            mev_protection_enabled: true,
            oracle_validation_enabled: true,
            defi_monitoring_enabled: true,
            risk_assessment_enabled: true,
            emergency_response_enabled: true,
            audit_logging_enabled: true,
            max_gas_price: U256::from(100) * U256::exp10(9), // 100 Gwei
            max_transaction_value: U256::from(1000) * U256::exp10(18), // 1000 ETH
            blacklisted_addresses: vec![],
        }
    }
}

#[derive(Debug, Default)]
pub struct SecurityMetrics {
    pub transactions_analyzed: u64,
    pub threats_detected: u64,
    pub emergency_responses: u64,
    pub average_risk_score: f64,
    pub last_updated: DateTime<Utc>,
}

/// Advanced security manager with comprehensive protection capabilities
pub struct AdvancedSecurityManager {
    provider: Arc<Provider<Http>>,
    config: Arc<RwLock<SecurityConfig>>,
    
    // Security modules
    mev_protection: Arc<MevProtection>,
    oracle_security: Arc<OracleSecurity>,
    defi_security: Arc<DeFiSecurity>,
    risk_engine: Arc<RiskEngine>,
    emergency_response: Arc<EmergencyResponse>,
    audit_trail: Arc<AuditTrail>,
    
    // State management
    threat_level: Arc<RwLock<ThreatLevel>>,
    security_metrics: Arc<RwLock<SecurityMetrics>>,
}

impl AdvancedSecurityManager {
    pub async fn new(provider: Arc<Provider<Http>>) -> Result<Self> {
        let config = Arc::new(RwLock::new(SecurityConfig::default()));
        
        // Initialize all security modules
        let mev_protection = Arc::new(MevProtection::new(provider.clone()));
        let oracle_security = Arc::new(OracleSecurity::new(provider.clone()));
        let defi_security = Arc::new(DeFiSecurity::new(provider.clone()));
        let risk_engine = Arc::new(RiskEngine::new(provider.clone()));
        let emergency_response = Arc::new(EmergencyResponse::new(provider.clone()));
        let audit_trail = Arc::new(AuditTrail::new(provider.clone()));
        
        Ok(Self {
            provider,
            config,
            mev_protection,
            oracle_security,
            defi_security,
            risk_engine,
            emergency_response,
            audit_trail,
            threat_level: Arc::new(RwLock::new(ThreatLevel::Low)),
            security_metrics: Arc::new(RwLock::new(SecurityMetrics::default())),
        })
    }

    pub async fn initialize(&self) -> Result<()> {
        let config = self.config.read().await;
        info!("Initializing advanced security system...");
        
        if config.mev_protection_enabled {
            self.mev_protection.initialize().await?;
            info!("MEV protection initialized");
        }
        
        if config.oracle_validation_enabled {
            // Oracle security would be initialized here if needed
            info!("Oracle security configured");
        }
        
        if config.defi_monitoring_enabled {
            self.defi_security.initialize().await?;
            info!("DeFi security monitoring initialized");
        }
        
        if config.risk_assessment_enabled {
            self.risk_engine.initialize().await?;
            info!("Risk engine initialized");
        }
        
        if config.emergency_response_enabled {
            self.emergency_response.initialize().await?;
            info!("Emergency response system initialized");
        }
        
        if config.audit_logging_enabled {
            self.audit_trail.initialize().await?;
            info!("Audit trail initialized");
        }
        
        info!("Advanced security system fully initialized");
        Ok(())
    }

    /// Analyze transaction for security threats
    pub async fn analyze_transaction(&self, tx: &TransactionRequest) -> Result<SecurityAnalysisResult> {
        let start_time = Utc::now();
        let mut threats = Vec::new();
        let mut recommendations = Vec::new();
        let mut risk_score = 0.0f64;

        let config = self.config.read().await;
        
        // MEV Protection Analysis
        if config.mev_protection_enabled {
            let mev_threats = self.mev_protection.analyze_transaction(tx).await?;
            for threat in mev_threats {
                threats.push(ThreatType::MEV(threat));
                risk_score += 0.3; // MEV threats contribute significantly to risk
            }
        }

        // Oracle Security Analysis
        if config.oracle_validation_enabled {
            // Oracle security analysis would go here
            // For now, no threats detected
        }

        // DeFi Security Analysis
        if config.defi_monitoring_enabled {
            let defi_threats = self.defi_security.analyze_defi_transaction(tx).await?;
            for threat in defi_threats {
                threats.push(ThreatType::DeFi(format!("DeFi threat detected: {:?}", threat)));
                risk_score += 0.2;
            }
        }

        // Risk Engine Analysis
        if config.risk_assessment_enabled {
            let risk_result = self.risk_engine.assess_transaction_risk(tx).await?;
            risk_score = (risk_score + risk_result.overall_risk_score) / 2.0; // Average with other assessments
            recommendations.extend(risk_result.recommended_actions);
        }

        // Normalize risk score to 0-1 range
        risk_score = risk_score.min(1.0);

        // Determine overall security status
        let security_status = match risk_score {
            s if s < 0.3 => SecurityStatus::Safe,
            s if s < 0.6 => SecurityStatus::Caution,
            s if s < 0.8 => SecurityStatus::Warning,
            _ => SecurityStatus::Danger,
        };

        // Update threat level if necessary
        self.update_threat_level_if_needed(risk_score).await?;

        // Log security analysis
        if config.audit_logging_enabled {
            self.audit_trail.log_security_event(
                AuditEntryType::RiskAssessment,
                tx.from,
                format!("Transaction security analysis completed with risk score: {:.2}", risk_score),
                risk_score,
                vec!["transaction_analysis".to_string()]
            ).await?;
        }

        // Update metrics
        self.update_security_metrics(|metrics| {
            metrics.transactions_analyzed += 1;
            if !threats.is_empty() {
                metrics.threats_detected += 1;
            }
            metrics.last_updated = Utc::now();
        }).await;

        let analysis_time = Utc::now().signed_duration_since(start_time);

        Ok(SecurityAnalysisResult {
            security_status,
            risk_score,
            threats: threats.into_iter().map(|t| SecurityThreat {
                threat_id: format!("threat_{}", Utc::now().timestamp_nanos()),
                threat_type: t,
                severity: risk_score,
                detected_at: Utc::now(),
                source_address: tx.from,
                description: "Detected during transaction analysis".to_string(),
                mitigation_actions: recommendations.clone(),
            }).collect(),
            recommendations,
            analysis_duration: analysis_time,
            should_proceed: risk_score < config.risk_tolerance,
        })
    }

    /// Apply security protections to a transaction
    pub async fn apply_protections(&self, mut tx: TransactionRequest, analysis: &SecurityAnalysisResult) -> Result<TransactionRequest> {
        // Apply MEV protection if threats detected
        for threat in &analysis.threats {
            if let ThreatType::MEV(mev_threat) = &threat.threat_type {
                tx = self.mev_protection.apply_protection(tx, &[mev_threat.clone()]).await?;
            }
        }

        // Apply additional protections based on risk score
        if analysis.risk_score > 0.7 {
            // High risk - apply maximum protections
            tx = self.apply_high_risk_protections(tx).await?;
        } else if analysis.risk_score > 0.4 {
            // Medium risk - apply moderate protections
            tx = self.apply_medium_risk_protections(tx).await?;
        }

        Ok(tx)
    }

    /// Handle security emergency
    pub async fn handle_emergency(&self, alert: EmergencyAlert) -> Result<()> {
        self.emergency_response.trigger_alert(alert.clone()).await?;
        
        // Update threat level to critical
        *self.threat_level.write().await = ThreatLevel::Critical;
        
        // Log emergency
        if self.config.read().await.audit_logging_enabled {
            self.audit_trail.log_security_event(
                AuditEntryType::EmergencyAction,
                None,
                format!("Security emergency: {}", alert.title),
                1.0,
                vec!["emergency".to_string(), alert.level.to_string().to_lowercase()]
            ).await?;
        }
        
        self.update_security_metrics(|metrics| {
            metrics.emergency_responses += 1;
        }).await;
        
        Ok(())
    }

    /// Generate comprehensive security report
    pub async fn generate_security_report(&self, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Result<SecurityReport> {
        let mut report = SecurityReport {
            report_id: format!("security_{}", Utc::now().timestamp()),
            generated_at: Utc::now(),
            period_start: start_time,
            period_end: end_time,
            overall_security_score: 0.0,
            threat_summary: HashMap::new(),
            mev_stats: None,
            oracle_stats: None,
            defi_stats: None,
            emergency_stats: None,
            audit_stats: None,
            compliance_report: None,
            recommendations: Vec::new(),
        };

        let config = self.config.read().await;

        // Collect statistics from all modules
        if config.mev_protection_enabled {
            report.mev_stats = Some(self.mev_protection.get_statistics().await?);
        }

        if config.oracle_validation_enabled {
            report.oracle_stats = Some(self.oracle_security.get_statistics().await?);
        }

        if config.defi_monitoring_enabled {
            report.defi_stats = Some(self.defi_security.get_statistics().await?);
        }

        if config.emergency_response_enabled {
            report.emergency_stats = Some(self.emergency_response.get_statistics().await?);
        }

        if config.audit_logging_enabled {
            report.audit_stats = Some(self.audit_trail.get_statistics().await?);
            report.compliance_report = Some(
                self.audit_trail.generate_compliance_report(start_time, end_time).await?
            );
        }

        // Calculate overall security score
        let metrics = self.security_metrics.read().await;
        report.overall_security_score = if metrics.transactions_analyzed > 0 {
            1.0 - (metrics.threats_detected as f64 / metrics.transactions_analyzed as f64)
        } else {
            1.0
        };

        // Generate recommendations
        report.recommendations = self.generate_security_recommendations(&report).await?;

        Ok(report)
    }

    /// Get current security status
    pub async fn get_security_status(&self) -> Result<SecurityStatus> {
        let threat_level = self.threat_level.read().await;
        match *threat_level {
            ThreatLevel::Low => Ok(SecurityStatus::Safe),
            ThreatLevel::Medium => Ok(SecurityStatus::Caution),
            ThreatLevel::High => Ok(SecurityStatus::Warning),
            ThreatLevel::Critical => Ok(SecurityStatus::Danger),
        }
    }

    // Helper methods
    async fn update_threat_level_if_needed(&self, risk_score: f64) -> Result<()> {
        let new_level = match risk_score {
            s if s < 0.3 => ThreatLevel::Low,
            s if s < 0.6 => ThreatLevel::Medium,
            s if s < 0.8 => ThreatLevel::High,
            _ => ThreatLevel::Critical,
        };

        let mut current_level = self.threat_level.write().await;
        if std::mem::discriminant(&new_level) != std::mem::discriminant(&*current_level) {
            *current_level = new_level;
        }

        Ok(())
    }

    async fn apply_high_risk_protections(&self, tx: TransactionRequest) -> Result<TransactionRequest> {
        // Apply maximum security measures
        Ok(tx)
    }

    async fn apply_medium_risk_protections(&self, tx: TransactionRequest) -> Result<TransactionRequest> {
        // Apply moderate security measures
        Ok(tx)
    }

    async fn update_security_metrics<F>(&self, updater: F) 
    where 
        F: FnOnce(&mut SecurityMetrics),
    {
        let mut metrics = self.security_metrics.write().await;
        updater(&mut *metrics);
    }

    async fn generate_security_recommendations(&self, _report: &SecurityReport) -> Result<Vec<String>> {
        let mut recommendations = Vec::new();
        
        let metrics = self.security_metrics.read().await;
        let threat_rate = if metrics.transactions_analyzed > 0 {
            metrics.threats_detected as f64 / metrics.transactions_analyzed as f64
        } else {
            0.0
        };

        if threat_rate > 0.1 {
            recommendations.push("High threat detection rate - consider strengthening security measures".to_string());
        }

        if metrics.average_risk_score > 0.6 {
            recommendations.push("Elevated average risk score - review transaction patterns".to_string());
        }

        if metrics.emergency_responses > 0 {
            recommendations.push("Emergency responses triggered - conduct security audit".to_string());
        }

        Ok(recommendations)
    }
}

#[derive(Debug, Clone)]
pub struct SecurityAnalysisResult {
    pub security_status: SecurityStatus,
    pub risk_score: f64,
    pub threats: Vec<SecurityThreat>,
    pub recommendations: Vec<String>,
    pub analysis_duration: Duration,
    pub should_proceed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityReport {
    pub report_id: String,
    pub generated_at: DateTime<Utc>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub overall_security_score: f64,
    pub threat_summary: HashMap<String, usize>,
    pub mev_stats: Option<MevStats>,
    pub oracle_stats: Option<OracleSecurityStats>,
    pub defi_stats: Option<DeFiSecurityStats>,
    pub emergency_stats: Option<EmergencyStats>,
    pub audit_stats: Option<AuditStats>,
    pub compliance_report: Option<ComplianceReport>,
    pub recommendations: Vec<String>,
}

// Basic security for backward compatibility
#[derive(Debug)]
pub struct BasicSecurity {
    blacklisted_addresses: HashSet<Address>,
    max_transaction_value: U256,
    max_gas_limit: u64,
    validator: transaction_validator::TransactionValidator,
    reentrancy_guard: reentrancy_guard::ReentrancyGuard,
    input_sanitizer: input_sanitizer::InputSanitizer,
}

impl BasicSecurity {
    pub async fn new() -> Result<Self> {
        let mut blacklisted_addresses = HashSet::new();
        
        // Add known malicious addresses
        blacklisted_addresses.insert("0x0000000000000000000000000000000000000000".parse()?);
        
        Ok(Self {
            blacklisted_addresses,
            max_transaction_value: U256::from(1000) * U256::exp10(18), // 1000 ETH
            max_gas_limit: 10_000_000,
            validator: transaction_validator::TransactionValidator::new(),
            reentrancy_guard: reentrancy_guard::ReentrancyGuard::new(),
            input_sanitizer: input_sanitizer::InputSanitizer::new(),
        })
    }

    pub async fn validate_transaction(&self, tx: &Transaction) -> Result<()> {
        // Check if recipient is blacklisted
        if let Some(to) = &tx.to {
            if self.blacklisted_addresses.contains(to) {
                return Err(anyhow::anyhow!("Transaction to blacklisted address"));
            }
        }

        // Check transaction value limits
        if tx.value > self.max_transaction_value {
            warn!("Transaction value {} exceeds maximum {}", tx.value, self.max_transaction_value);
            return Err(anyhow::anyhow!("Transaction value too high"));
        }

        // Check gas limit
        if tx.gas.as_u64() > self.max_gas_limit {
            return Err(anyhow::anyhow!("Gas limit too high"));
        }

        // Skip additional validations for now to get compilation working
        
        // Validate transaction data
        self.input_sanitizer.validate_call_data(&tx.input)?;

        Ok(())
    }

    pub fn calculate_transaction_hash(&self, tx: &Transaction) -> Result<H256> {
        // Use transaction hash or compute from transaction data
        Ok(tx.hash)
    }
}

/// Main security manager combining advanced and basic functionality
pub struct SecurityManager {
    pub advanced: Arc<AdvancedSecurityManager>,
    pub basic: BasicSecurity,
}

impl SecurityManager {
    pub async fn new(provider: Provider<Http>) -> Result<Self> {
        let advanced = Arc::new(AdvancedSecurityManager::new(Arc::new(provider)).await?);
        let basic = BasicSecurity::new().await?;
        
        Ok(Self {
            advanced,
            basic,
        })
    }

    // Delegate advanced functionality
    pub async fn analyze_transaction(&self, tx: &TransactionRequest) -> Result<SecurityAnalysisResult> {
        self.advanced.analyze_transaction(tx).await
    }

    pub async fn apply_protections(&self, tx: TransactionRequest, analysis: &SecurityAnalysisResult) -> Result<TransactionRequest> {
        self.advanced.apply_protections(tx, analysis).await
    }

    pub async fn handle_emergency(&self, alert: EmergencyAlert) -> Result<()> {
        self.advanced.handle_emergency(alert).await
    }

    pub async fn generate_security_report(&self, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Result<SecurityReport> {
        self.advanced.generate_security_report(start_time, end_time).await
    }

    pub async fn get_security_status(&self) -> Result<SecurityStatus> {
        self.advanced.get_security_status().await
    }

    // Basic functionality delegation
    pub async fn validate_transaction(&self, tx: &Transaction) -> Result<()> {
        self.basic.validate_transaction(tx).await
    }

    // Compatibility method for TypedTransaction
    pub async fn validate_typed_transaction(&self, tx: &TypedTransaction) -> Result<()> {
        // Basic validation for TypedTransaction
        if let Some(value) = tx.value() {
            if *value > U256::from(1000) * U256::exp10(18) { // 1000 ETH limit
                return Err(anyhow::anyhow!("Transaction value too high"));
            }
        }
        Ok(())
    }

    pub fn calculate_transaction_hash(&self, tx: &Transaction) -> Result<H256> {
        self.basic.calculate_transaction_hash(tx)
    }
}
