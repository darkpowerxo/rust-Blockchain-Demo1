use anyhow::{Result, anyhow};
use ethers::{
    prelude::*,
    types::{Address, U256, TransactionRequest, H256, Bytes, TransactionReceipt},
};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub entry_type: AuditEntryType,
    pub timestamp: DateTime<Utc>,
    pub user_address: Option<Address>,
    pub transaction_hash: Option<H256>,
    pub contract_address: Option<Address>,
    pub function_called: Option<String>,
    pub parameters: HashMap<String, String>,
    pub gas_used: Option<U256>,
    pub gas_price: Option<U256>,
    pub value: Option<U256>,
    pub success: bool,
    pub error_message: Option<String>,
    pub risk_score: Option<f64>,
    pub security_flags: Vec<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditEntryType {
    // Transaction events
    TransactionSubmitted,
    TransactionExecuted,
    TransactionFailed,
    
    // Security events
    SecurityViolation,
    SuspiciousActivity,
    RiskAssessment,
    ThreatDetected,
    
    // System events
    SystemStart,
    SystemStop,
    ConfigurationChange,
    EmergencyAction,
    
    // User events
    UserLogin,
    UserAction,
    AdminAction,
    
    // DeFi events
    LiquidationEvent,
    FlashLoanExecution,
    ArbitrageTransaction,
    GovernanceVote,
    
    // Oracle events
    PriceUpdate,
    OracleFailure,
    PriceDeviation,
    
    // Protocol events
    ProtocolUpgrade,
    ParameterChange,
    PauseEvent,
    UnpauseEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditQuery {
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub entry_types: Vec<AuditEntryType>,
    pub user_address: Option<Address>,
    pub contract_address: Option<Address>,
    pub transaction_hash: Option<H256>,
    pub risk_score_min: Option<f64>,
    pub risk_score_max: Option<f64>,
    pub security_flags: Vec<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub report_id: String,
    pub generated_at: DateTime<Utc>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_transactions: usize,
    pub high_risk_transactions: usize,
    pub security_violations: usize,
    pub compliance_score: f64,
    pub recommendations: Vec<String>,
    pub detailed_entries: Vec<AuditEntry>,
}

pub struct AuditTrail {
    provider: Arc<Provider<Http>>,
    audit_log: Arc<RwLock<VecDeque<AuditEntry>>>,
    indexed_entries: Arc<RwLock<HashMap<String, Vec<String>>>>, // Index by different fields
    compliance_rules: Arc<RwLock<HashMap<String, ComplianceRule>>>,
    retention_policy: Arc<RwLock<RetentionPolicy>>,
    encryption_key: Arc<RwLock<Vec<u8>>>,
    storage_backend: Arc<RwLock<StorageBackend>>,
}

#[derive(Debug, Clone)]
struct ComplianceRule {
    name: String,
    description: String,
    rule_type: ComplianceRuleType,
    threshold: f64,
    action: ComplianceAction,
    enabled: bool,
}

#[derive(Debug, Clone)]
enum ComplianceRuleType {
    TransactionValue(U256),
    RiskScore(f64),
    SecurityFlag(String),
    TransactionFrequency(usize), // transactions per time period
    GasUsage(U256),
}

#[derive(Debug, Clone)]
enum ComplianceAction {
    LogWarning,
    BlockTransaction,
    RequireApproval,
    NotifyCompliance,
    GenerateReport,
}

#[derive(Debug, Clone)]
struct RetentionPolicy {
    default_retention_days: i64,
    high_risk_retention_days: i64,
    compliance_retention_days: i64,
    archive_after_days: i64,
    delete_after_days: i64,
}

#[derive(Debug, Clone)]
enum StorageBackend {
    Memory,
    Database(String), // Connection string
    IPFS(String),     // IPFS node
    S3(String),       // S3 bucket
}

impl AuditTrail {
    pub fn new(provider: Arc<Provider<Http>>) -> Self {
        Self {
            provider,
            audit_log: Arc::new(RwLock::new(VecDeque::with_capacity(100000))),
            indexed_entries: Arc::new(RwLock::new(HashMap::new())),
            compliance_rules: Arc::new(RwLock::new(HashMap::new())),
            retention_policy: Arc::new(RwLock::new(RetentionPolicy {
                default_retention_days: 90,
                high_risk_retention_days: 365,
                compliance_retention_days: 2555, // 7 years
                archive_after_days: 30,
                delete_after_days: 2555,
            })),
            encryption_key: Arc::new(RwLock::new(vec![0u8; 32])), // Would use proper key management
            storage_backend: Arc::new(RwLock::new(StorageBackend::Memory)),
        }
    }

    /// Initialize audit trail system
    pub async fn initialize(&self) -> Result<()> {
        self.setup_compliance_rules().await?;
        self.generate_encryption_key().await?;
        self.create_indices().await?;
        
        // Log system initialization
        self.log_entry(AuditEntry {
            id: self.generate_id(),
            entry_type: AuditEntryType::SystemStart,
            timestamp: Utc::now(),
            user_address: None,
            transaction_hash: None,
            contract_address: None,
            function_called: None,
            parameters: HashMap::new(),
            gas_used: None,
            gas_price: None,
            value: None,
            success: true,
            error_message: None,
            risk_score: None,
            security_flags: Vec::new(),
            metadata: [("system".to_string(), "audit_trail".to_string())].into(),
        }).await?;
        
        tracing::info!("Audit trail system initialized");
        Ok(())
    }

    /// Log an audit entry
    pub async fn log_entry(&self, entry: AuditEntry) -> Result<()> {
        let entry_id = entry.id.clone();
        
        // Check compliance rules
        self.check_compliance_rules(&entry).await?;
        
        // Encrypt sensitive data if needed
        let encrypted_entry = self.encrypt_entry(entry).await?;
        
        // Store in memory log
        let mut log = self.audit_log.write().await;
        log.push_back(encrypted_entry.clone());
        
        // Apply retention policy
        self.apply_retention_policy(&mut log).await?;
        
        // Update indices
        self.update_indices(&encrypted_entry).await?;
        
        // Persist to storage backend if configured
        self.persist_to_backend(&encrypted_entry).await?;
        
        tracing::debug!("Audit entry logged: {}", entry_id);
        Ok(())
    }

    /// Log a transaction
    pub async fn log_transaction(&self, tx: &TransactionRequest, tx_hash: Option<H256>, receipt: Option<&TransactionReceipt>, risk_score: Option<f64>) -> Result<()> {
        let success = receipt.map(|r| r.status.unwrap_or_default() == U64::from(1)).unwrap_or(false);
        let gas_used = receipt.map(|r| r.gas_used.unwrap_or_default());
        
        let entry = AuditEntry {
            id: self.generate_id(),
            entry_type: if success { AuditEntryType::TransactionExecuted } else { AuditEntryType::TransactionFailed },
            timestamp: Utc::now(),
            user_address: tx.from,
            transaction_hash: tx_hash,
            contract_address: tx.to.as_ref().and_then(|to| match to {
                NameOrAddress::Address(addr) => Some(*addr),
                NameOrAddress::Name(_) => None,
            }),
            function_called: self.extract_function_name(&tx.data).await?,
            parameters: self.extract_parameters(&tx.data).await?,
            gas_used,
            gas_price: tx.gas_price,
            value: tx.value,
            success,
            error_message: None, // Would extract from receipt
            risk_score,
            security_flags: Vec::new(), // Would be populated by security modules
            metadata: HashMap::new(),
        };
        
        self.log_entry(entry).await
    }

    /// Log a security event
    pub async fn log_security_event(&self, event_type: AuditEntryType, address: Option<Address>, description: String, risk_score: f64, flags: Vec<String>) -> Result<()> {
        let entry = AuditEntry {
            id: self.generate_id(),
            entry_type: event_type,
            timestamp: Utc::now(),
            user_address: address,
            transaction_hash: None,
            contract_address: None,
            function_called: None,
            parameters: [("description".to_string(), description)].into(),
            gas_used: None,
            gas_price: None,
            value: None,
            success: true,
            error_message: None,
            risk_score: Some(risk_score),
            security_flags: flags,
            metadata: HashMap::new(),
        };
        
        self.log_entry(entry).await
    }

    /// Query audit entries
    pub async fn query_entries(&self, query: AuditQuery) -> Result<Vec<AuditEntry>> {
        let log = self.audit_log.read().await;
        let mut results = Vec::new();
        
        let entries: Vec<_> = if let Some(limit) = query.limit {
            let start = query.offset.unwrap_or(0);
            log.iter().skip(start).take(limit).collect()
        } else {
            log.iter().collect()
        };
        
        for entry in entries {
            let decrypted_entry = self.decrypt_entry(entry.clone()).await?;
            
            if self.matches_query(&decrypted_entry, &query) {
                results.push(decrypted_entry);
            }
        }
        
        // Sort by timestamp (newest first)
        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        Ok(results)
    }

    /// Generate compliance report
    pub async fn generate_compliance_report(&self, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Result<ComplianceReport> {
        let query = AuditQuery {
            start_time: Some(start_time),
            end_time: Some(end_time),
            entry_types: vec![
                AuditEntryType::TransactionExecuted,
                AuditEntryType::SecurityViolation,
                AuditEntryType::SuspiciousActivity,
            ],
            user_address: None,
            contract_address: None,
            transaction_hash: None,
            risk_score_min: None,
            risk_score_max: None,
            security_flags: Vec::new(),
            limit: None,
            offset: None,
        };
        
        let entries = self.query_entries(query).await?;
        let total_transactions = entries.len();
        let high_risk_transactions = entries.iter().filter(|e| e.risk_score.unwrap_or(0.0) > 0.7).count();
        let security_violations = entries.iter().filter(|e| matches!(e.entry_type, AuditEntryType::SecurityViolation)).count();
        
        // Calculate compliance score (simplified)
        let compliance_score = if total_transactions > 0 {
            1.0 - (security_violations as f64 / total_transactions as f64)
        } else {
            1.0
        };
        
        let recommendations = self.generate_compliance_recommendations(&entries).await?;
        
        Ok(ComplianceReport {
            report_id: self.generate_id(),
            generated_at: Utc::now(),
            period_start: start_time,
            period_end: end_time,
            total_transactions,
            high_risk_transactions,
            security_violations,
            compliance_score,
            recommendations,
            detailed_entries: entries,
        })
    }

    /// Get audit statistics
    pub async fn get_statistics(&self) -> Result<AuditStats> {
        let log = self.audit_log.read().await;
        let now = Utc::now();
        let last_24h = now - chrono::Duration::hours(24);
        
        let total_entries = log.len();
        let recent_entries = log.iter().filter(|e| e.timestamp >= last_24h).count();
        
        let mut entry_counts = HashMap::new();
        for entry in log.iter() {
            *entry_counts.entry(format!("{:?}", entry.entry_type)).or_insert(0) += 1;
        }
        
        let high_risk_entries = log.iter().filter(|e| e.risk_score.unwrap_or(0.0) > 0.7).count();
        let security_events = log.iter().filter(|e| {
            matches!(e.entry_type, 
                AuditEntryType::SecurityViolation | 
                AuditEntryType::SuspiciousActivity | 
                AuditEntryType::ThreatDetected
            )
        }).count();
        
        Ok(AuditStats {
            total_entries,
            entries_last_24h: recent_entries,
            high_risk_entries,
            security_events,
            entry_type_counts: entry_counts,
            oldest_entry: log.front().map(|e| e.timestamp),
            newest_entry: log.back().map(|e| e.timestamp),
        })
    }

    /// Check compliance rules
    async fn check_compliance_rules(&self, entry: &AuditEntry) -> Result<()> {
        let rules = self.compliance_rules.read().await;
        
        for (name, rule) in rules.iter() {
            if !rule.enabled {
                continue;
            }
            
            let violation = match &rule.rule_type {
                ComplianceRuleType::TransactionValue(threshold) => {
                    entry.value.unwrap_or(U256::zero()) > *threshold
                }
                ComplianceRuleType::RiskScore(threshold) => {
                    entry.risk_score.unwrap_or(0.0) > *threshold
                }
                ComplianceRuleType::SecurityFlag(flag) => {
                    entry.security_flags.contains(flag)
                }
                ComplianceRuleType::GasUsage(threshold) => {
                    entry.gas_used.unwrap_or(U256::zero()) > *threshold
                }
                ComplianceRuleType::TransactionFrequency(_) => {
                    // Would check transaction frequency for the user
                    false
                }
            };
            
            if violation {
                tracing::warn!("Compliance rule violation: {} for entry {}", name, entry.id);
                
                match &rule.action {
                    ComplianceAction::LogWarning => {
                        // Already logged above
                    }
                    ComplianceAction::BlockTransaction => {
                        return Err(anyhow!("Transaction blocked by compliance rule: {}", name));
                    }
                    ComplianceAction::RequireApproval => {
                        // Would trigger approval workflow
                        tracing::info!("Transaction requires approval due to rule: {}", name);
                    }
                    ComplianceAction::NotifyCompliance => {
                        // Would send notification to compliance team
                        tracing::info!("Compliance team notified due to rule: {}", name);
                    }
                    ComplianceAction::GenerateReport => {
                        // Would generate detailed report
                        tracing::info!("Compliance report generated due to rule: {}", name);
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Encrypt sensitive entry data
    async fn encrypt_entry(&self, mut entry: AuditEntry) -> Result<AuditEntry> {
        // In a real implementation, this would encrypt sensitive fields
        // For now, just return the entry as-is
        Ok(entry)
    }

    /// Decrypt entry data
    async fn decrypt_entry(&self, entry: AuditEntry) -> Result<AuditEntry> {
        // In a real implementation, this would decrypt the entry
        Ok(entry)
    }

    /// Check if entry matches query criteria
    fn matches_query(&self, entry: &AuditEntry, query: &AuditQuery) -> bool {
        // Time range check
        if let Some(start) = query.start_time {
            if entry.timestamp < start {
                return false;
            }
        }
        
        if let Some(end) = query.end_time {
            if entry.timestamp > end {
                return false;
            }
        }
        
        // Entry type check
        if !query.entry_types.is_empty() && !query.entry_types.contains(&entry.entry_type) {
            return false;
        }
        
        // Address checks
        if let Some(addr) = query.user_address {
            if entry.user_address != Some(addr) {
                return false;
            }
        }
        
        if let Some(addr) = query.contract_address {
            if entry.contract_address != Some(addr) {
                return false;
            }
        }
        
        // Risk score checks
        if let Some(min_risk) = query.risk_score_min {
            if entry.risk_score.unwrap_or(0.0) < min_risk {
                return false;
            }
        }
        
        if let Some(max_risk) = query.risk_score_max {
            if entry.risk_score.unwrap_or(0.0) > max_risk {
                return false;
            }
        }
        
        // Security flags check
        if !query.security_flags.is_empty() {
            if !query.security_flags.iter().any(|flag| entry.security_flags.contains(flag)) {
                return false;
            }
        }
        
        true
    }

    /// Apply retention policy to log
    async fn apply_retention_policy(&self, log: &mut VecDeque<AuditEntry>) -> Result<()> {
        let policy = self.retention_policy.read().await;
        let cutoff_time = Utc::now() - chrono::Duration::days(policy.default_retention_days);
        
        while let Some(entry) = log.front() {
            if entry.timestamp < cutoff_time {
                // Check if entry should be retained longer
                let should_retain = entry.risk_score.unwrap_or(0.0) > 0.7 || 
                                   matches!(entry.entry_type, AuditEntryType::SecurityViolation);
                
                if !should_retain {
                    log.pop_front();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        Ok(())
    }

    /// Update search indices
    async fn update_indices(&self, entry: &AuditEntry) -> Result<()> {
        let mut indices = self.indexed_entries.write().await;
        
        // Index by user address
        if let Some(addr) = entry.user_address {
            indices.entry(format!("user:{}", addr))
                  .or_insert_with(Vec::new)
                  .push(entry.id.clone());
        }
        
        // Index by contract address
        if let Some(addr) = entry.contract_address {
            indices.entry(format!("contract:{}", addr))
                  .or_insert_with(Vec::new)
                  .push(entry.id.clone());
        }
        
        // Index by entry type
        indices.entry(format!("type:{:?}", entry.entry_type))
              .or_insert_with(Vec::new)
              .push(entry.id.clone());
        
        Ok(())
    }

    /// Helper functions
    fn generate_id(&self) -> String {
        format!("audit_{}", Utc::now().timestamp_nanos())
    }

    async fn extract_function_name(&self, data: &Option<Bytes>) -> Result<Option<String>> {
        // Would decode function selector and look up function name
        Ok(data.as_ref().map(|_| "unknown".to_string()))
    }

    async fn extract_parameters(&self, data: &Option<Bytes>) -> Result<HashMap<String, String>> {
        // Would decode function parameters
        Ok(HashMap::new())
    }

    async fn setup_compliance_rules(&self) -> Result<()> {
        let mut rules = self.compliance_rules.write().await;
        
        // High value transaction rule
        rules.insert("high_value_transaction".to_string(), ComplianceRule {
            name: "High Value Transaction".to_string(),
            description: "Transactions above 100 ETH require additional monitoring".to_string(),
            rule_type: ComplianceRuleType::TransactionValue(U256::from(100) * U256::exp10(18)),
            threshold: 1.0,
            action: ComplianceAction::NotifyCompliance,
            enabled: true,
        });
        
        // High risk transaction rule
        rules.insert("high_risk_transaction".to_string(), ComplianceRule {
            name: "High Risk Transaction".to_string(),
            description: "Transactions with risk score > 0.8 require approval".to_string(),
            rule_type: ComplianceRuleType::RiskScore(0.8),
            threshold: 0.8,
            action: ComplianceAction::RequireApproval,
            enabled: true,
        });
        
        Ok(())
    }

    async fn generate_encryption_key(&self) -> Result<()> {
        // In production, would use proper key management
        let key = (0..32).map(|_| rand::random::<u8>()).collect();
        *self.encryption_key.write().await = key;
        Ok(())
    }

    async fn create_indices(&self) -> Result<()> {
        // Initialize index structures
        Ok(())
    }

    async fn persist_to_backend(&self, _entry: &AuditEntry) -> Result<()> {
        // Would persist to configured storage backend
        Ok(())
    }

    async fn generate_compliance_recommendations(&self, entries: &[AuditEntry]) -> Result<Vec<String>> {
        let mut recommendations = Vec::new();
        
        let high_risk_ratio = entries.iter().filter(|e| e.risk_score.unwrap_or(0.0) > 0.7).count() as f64 / entries.len() as f64;
        
        if high_risk_ratio > 0.1 {
            recommendations.push("Consider implementing stricter risk controls".to_string());
        }
        
        let security_events = entries.iter().filter(|e| matches!(e.entry_type, AuditEntryType::SecurityViolation)).count();
        if security_events > 0 {
            recommendations.push("Review and strengthen security measures".to_string());
        }
        
        Ok(recommendations)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditStats {
    pub total_entries: usize,
    pub entries_last_24h: usize,
    pub high_risk_entries: usize,
    pub security_events: usize,
    pub entry_type_counts: HashMap<String, usize>,
    pub oldest_entry: Option<DateTime<Utc>>,
    pub newest_entry: Option<DateTime<Utc>>,
}
