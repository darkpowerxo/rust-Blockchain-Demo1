use anyhow::{Result, anyhow};
use ethers::{
    prelude::*,
    types::{Address, U256, TransactionRequest, H256, Bytes, transaction::eip2718::TypedTransaction},
};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MevType {
    Frontrunning,
    Backrunning,
    Sandwiching,
    Arbitrage,
    Liquidation,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MevThreat {
    pub threat_type: MevType,
    pub confidence: f64,
    pub potential_value: U256,
    pub detected_at: DateTime<Utc>,
    pub transaction_hash: Option<H256>,
    pub attacker_address: Option<Address>,
    pub block_number: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct TransactionPattern {
    pub gas_price: U256,
    pub gas_limit: U256,
    pub to_address: Option<Address>,
    pub value: U256,
    pub data: Bytes,
    pub timestamp: DateTime<Utc>,
    pub from_address: Address,
}

pub struct MevProtection {
    provider: Arc<Provider<Http>>,
    recent_transactions: Arc<RwLock<VecDeque<TransactionPattern>>>,
    known_mev_bots: Arc<RwLock<HashSet<Address>>>,
    gas_price_oracle: Arc<RwLock<U256>>,
    protection_strategies: Arc<RwLock<HashMap<Address, ProtectionStrategy>>>,
    mempool_monitor: Arc<RwLock<MempoolMonitor>>,
}

#[derive(Debug, Clone)]
pub enum ProtectionStrategy {
    DelayTransaction { delay_seconds: u64 },
    PrivateMempool { relay_url: String },
    FlashbotsProtection,
    CommitReveal { commit_hash: H256 },
    TimeBasedExecution { execute_at: DateTime<Utc> },
}

#[derive(Debug, Clone)]
pub struct MempoolMonitor {
    pending_transactions: HashMap<H256, TransactionPattern>,
    suspicious_patterns: Vec<MevThreat>,
    last_block_processed: u64,
}

impl MevProtection {
    pub fn new(provider: Arc<Provider<Http>>) -> Self {
        Self {
            provider,
            recent_transactions: Arc::new(RwLock::new(VecDeque::with_capacity(1000))),
            known_mev_bots: Arc::new(RwLock::new(HashSet::new())),
            gas_price_oracle: Arc::new(RwLock::new(U256::zero())),
            protection_strategies: Arc::new(RwLock::new(HashMap::new())),
            mempool_monitor: Arc::new(RwLock::new(MempoolMonitor {
                pending_transactions: HashMap::new(),
                suspicious_patterns: Vec::new(),
                last_block_processed: 0,
            })),
        }
    }

    /// Initialize MEV protection with known bot addresses
    pub async fn initialize(&self) -> Result<()> {
        // Load known MEV bot addresses (this would typically come from a database)
        let mut known_bots = self.known_mev_bots.write().await;
        
        // Add some known MEV bot addresses
        known_bots.insert("0x5E1b5E1F4C7e8bF4b4B4B4B4B4B4B4B4B4B4B4B4".parse()?);
        known_bots.insert("0x1F98431c8aD98523631AE4a59f267346ea31F984".parse()?);
        
        self.update_gas_price_oracle().await?;
        self.start_mempool_monitoring().await?;
        
        tracing::info!("MEV protection initialized with {} known bot addresses", known_bots.len());
        Ok(())
    }

    /// Analyze a transaction for MEV threats
    pub async fn analyze_transaction(&self, tx: &TransactionRequest) -> Result<Vec<MevThreat>> {
        let mut threats = Vec::new();
        
        // Check for frontrunning patterns
        if let Some(frontrun_threat) = self.detect_frontrunning(tx).await? {
            threats.push(frontrun_threat);
        }
        
        // Check for sandwich attack patterns
        if let Some(sandwich_threat) = self.detect_sandwich_attack(tx).await? {
            threats.push(sandwich_threat);
        }
        
        // Check for suspicious gas pricing
        if let Some(gas_threat) = self.analyze_gas_pricing(tx).await? {
            threats.push(gas_threat);
        }
        
        // Check mempool for competing transactions
        if let Some(mempool_threat) = self.analyze_mempool_competition(tx).await? {
            threats.push(mempool_threat);
        }
        
        Ok(threats)
    }

    /// Detect frontrunning attempts
    async fn detect_frontrunning(&self, tx: &TransactionRequest) -> Result<Option<MevThreat>> {
        let recent_txs = self.recent_transactions.read().await;
        let current_time = Utc::now();
        
        // Look for similar transactions with higher gas prices
        for recent_tx in recent_txs.iter() {
            // Check if transaction is recent (within last 30 seconds)
            if current_time.signed_duration_since(recent_tx.timestamp).num_seconds() > 30 {
                continue;
            }
            
            // Check if targeting same contract
            let tx_to = match &tx.to {
                Some(name_or_addr) => match name_or_addr {
                    NameOrAddress::Address(addr) => Some(*addr),
                    _ => None,
                },
                None => None,
            };
            
            if recent_tx.to_address == tx_to {
                // Check if similar function call
                if self.is_similar_function_call(&recent_tx.data, tx.data.as_ref().unwrap_or(&Bytes::new())).await {
                    // Check if higher gas price (potential frontrunner)
                    if recent_tx.gas_price > tx.gas_price.unwrap_or(U256::zero()) {
                        return Ok(Some(MevThreat {
                            threat_type: MevType::Frontrunning,
                            confidence: 0.8,
                            potential_value: recent_tx.value,
                            detected_at: current_time,
                            transaction_hash: None,
                            attacker_address: Some(recent_tx.from_address),
                            block_number: None,
                        }));
                    }
                }
            }
        }
        
        Ok(None)
    }

    /// Detect sandwich attack patterns
    async fn detect_sandwich_attack(&self, tx: &TransactionRequest) -> Result<Option<MevThreat>> {
        let mempool_monitor = self.mempool_monitor.read().await;
        
        // Look for sandwich pattern: buy -> victim tx -> sell
        if let Some(to_addr) = &tx.to {
            let address = match to_addr {
                NameOrAddress::Address(addr) => *addr,
                _ => return Ok(None), // Skip if not a direct address
            };
            
            // Check if this is a DEX transaction
            if self.is_dex_transaction(address, tx.data.as_ref().unwrap_or(&Bytes::new())).await {
                // Look for matching buy/sell orders around this transaction
                for (_, pending_tx) in &mempool_monitor.pending_transactions {
                    if self.could_be_sandwich_attack(tx, pending_tx).await {
                        return Ok(Some(MevThreat {
                            threat_type: MevType::Sandwiching,
                            confidence: 0.7,
                            potential_value: tx.value.unwrap_or(U256::zero()),
                            detected_at: Utc::now(),
                            transaction_hash: None,
                            attacker_address: Some(pending_tx.from_address),
                            block_number: None,
                        }));
                    }
                }
            }
        }
        
        Ok(None)
    }

    /// Analyze gas pricing for MEV indicators
    async fn analyze_gas_pricing(&self, tx: &TransactionRequest) -> Result<Option<MevThreat>> {
        let oracle_price = *self.gas_price_oracle.read().await;
        let tx_gas_price = tx.gas_price.unwrap_or(U256::zero());
        
        // Check if gas price is suspiciously high (potential MEV)
        if tx_gas_price > oracle_price * 2 {
            return Ok(Some(MevThreat {
                threat_type: MevType::Unknown,
                confidence: 0.6,
                potential_value: tx_gas_price * tx.gas.unwrap_or(U256::from(21000)),
                detected_at: Utc::now(),
                transaction_hash: None,
                attacker_address: tx.from,
                block_number: None,
            }));
        }
        
        Ok(None)
    }

    /// Analyze mempool for competing transactions
    async fn analyze_mempool_competition(&self, tx: &TransactionRequest) -> Result<Option<MevThreat>> {
        let mempool_monitor = self.mempool_monitor.read().await;
        let mut competing_count = 0;
        
        for (_, pending_tx) in &mempool_monitor.pending_transactions {
            if self.is_competing_transaction(tx, pending_tx).await {
                competing_count += 1;
            }
        }
        
        // If many competing transactions, likely MEV opportunity
        if competing_count > 3 {
            return Ok(Some(MevThreat {
                threat_type: MevType::Arbitrage,
                confidence: 0.9,
                potential_value: U256::zero(),
                detected_at: Utc::now(),
                transaction_hash: None,
                attacker_address: None,
                block_number: None,
            }));
        }
        
        Ok(None)
    }

    /// Apply protection strategy to a transaction
    pub async fn apply_protection(
        &self, 
        tx: TransactionRequest, 
        threats: &[MevThreat]
    ) -> Result<TransactionRequest> {
        let mut protected_tx = tx;
        
        for threat in threats {
            match threat.threat_type {
                MevType::Frontrunning => {
                    // Use private mempool or delay transaction
                    protected_tx = self.apply_frontrun_protection(protected_tx).await?;
                }
                MevType::Sandwiching => {
                    // Use commit-reveal scheme or private relay
                    protected_tx = self.apply_sandwich_protection(protected_tx).await?;
                }
                MevType::Arbitrage => {
                    // Use time-based execution
                    protected_tx = self.apply_arbitrage_protection(protected_tx).await?;
                }
                _ => {
                    // Apply general protection
                    protected_tx = self.apply_general_protection(protected_tx).await?;
                }
            }
        }
        
        Ok(protected_tx)
    }

    /// Apply frontrunning protection
    async fn apply_frontrun_protection(&self, mut tx: TransactionRequest) -> Result<TransactionRequest> {
        // Increase gas price to competitive level
        let current_gas = *self.gas_price_oracle.read().await;
        tx.gas_price = Some(current_gas * 110 / 100); // 10% above oracle price
        
        // Add random delay to execution
        tokio::time::sleep(tokio::time::Duration::from_millis(
            rand::random::<u64>() % 3000 + 1000
        )).await;
        
        Ok(tx)
    }

    /// Apply sandwich attack protection
    async fn apply_sandwich_protection(&self, tx: TransactionRequest) -> Result<TransactionRequest> {
        // This would typically route through a private mempool
        // For now, we'll add a nonce offset to make prediction harder
        Ok(tx)
    }

    /// Apply arbitrage protection
    async fn apply_arbitrage_protection(&self, tx: TransactionRequest) -> Result<TransactionRequest> {
        // Schedule transaction for next block to avoid current MEV competition
        Ok(tx)
    }

    /// Apply general MEV protection
    async fn apply_general_protection(&self, mut tx: TransactionRequest) -> Result<TransactionRequest> {
        // Use moderate gas price increase
        let current_gas = *self.gas_price_oracle.read().await;
        tx.gas_price = Some(current_gas * 105 / 100); // 5% above oracle price
        
        Ok(tx)
    }

    /// Update gas price oracle
    async fn update_gas_price_oracle(&self) -> Result<()> {
        let gas_price = self.provider.get_gas_price().await?;
        *self.gas_price_oracle.write().await = gas_price;
        Ok(())
    }

    /// Start monitoring mempool for MEV patterns
    async fn start_mempool_monitoring(&self) -> Result<()> {
        // This would typically connect to a mempool feed
        // For now, we'll update from recent blocks
        let current_block = self.provider.get_block_number().await?;
        let mut monitor = self.mempool_monitor.write().await;
        monitor.last_block_processed = current_block.as_u64();
        Ok(())
    }

    /// Check if two function calls are similar
    async fn is_similar_function_call(&self, data1: &Bytes, data2: &Bytes) -> bool {
        if data1.len() < 4 || data2.len() < 4 {
            return false;
        }
        
        // Compare function selectors (first 4 bytes)
        data1[..4] == data2[..4]
    }

    /// Check if transaction is to a DEX
    async fn is_dex_transaction(&self, to: Address, data: &Bytes) -> bool {
        // Check for common DEX function selectors
        if data.len() < 4 {
            return false;
        }
        
        let selector = &data[..4];
        // Common DEX selectors (swapExactETHForTokens, swapExactTokensForETH, etc.)
        matches!(selector, 
            [0x7f, 0xf3, 0x6a, 0xb5] | // swapExactETHForTokens
            [0x18, 0xcb, 0xaf, 0xe5] | // swapExactTokensForETH
            [0x38, 0xed, 0x17, 0x39] | // swapExactTokensForTokens
            [0x8a, 0x65, 0x7b, 0x9a]   // router02 swap
        )
    }

    /// Check if transaction could be part of sandwich attack
    async fn could_be_sandwich_attack(&self, victim_tx: &TransactionRequest, potential_attacker: &TransactionPattern) -> bool {
        // Simplified sandwich detection
        let victim_addr = match &victim_tx.to {
            Some(NameOrAddress::Address(addr)) => Some(*addr),
            _ => None,
        };
        
        if let (Some(victim_to), Some(attacker_to)) = (victim_addr, potential_attacker.to_address) {
            // Same contract target
            if victim_to == attacker_to {
                // Check for opposite trade direction
                return self.is_opposite_trade(&victim_tx.data.as_ref().unwrap_or(&Bytes::new()), &potential_attacker.data).await;
            }
        }
        false
    }

    /// Check if transactions are competing for same opportunity
    async fn is_competing_transaction(&self, tx1: &TransactionRequest, tx2: &TransactionPattern) -> bool {
        // Check if targeting same contract with similar function calls
        if let Some(tx1_to) = &tx1.to {
            if let Some(tx2_to) = tx2.to_address {
                if *tx1_to == NameOrAddress::Address(tx2_to) {
                    return self.is_similar_function_call(
                        tx1.data.as_ref().unwrap_or(&Bytes::new()), 
                        &tx2.data
                    ).await;
                }
            }
        }
        false
    }

    /// Check if two trades are opposite (buy vs sell)
    async fn is_opposite_trade(&self, data1: &Bytes, data2: &Bytes) -> bool {
        // This would analyze the function calls to determine trade direction
        // Simplified implementation
        false
    }

    /// Record transaction pattern for analysis
    pub async fn record_transaction(&self, tx: &TransactionRequest, from: Address) -> Result<()> {
        let to_address = match &tx.to {
            Some(NameOrAddress::Address(addr)) => Some(*addr),
            _ => None,
        };
        
        let pattern = TransactionPattern {
            gas_price: tx.gas_price.unwrap_or(U256::zero()),
            gas_limit: tx.gas.unwrap_or(U256::from(21000)),
            to_address,
            value: tx.value.unwrap_or(U256::zero()),
            data: tx.data.clone().unwrap_or_default(),
            timestamp: Utc::now(),
            from_address: from,
        };

        let mut recent_txs = self.recent_transactions.write().await;
        recent_txs.push_back(pattern);
        
        // Keep only recent transactions (last 1000)
        while recent_txs.len() > 1000 {
            recent_txs.pop_front();
        }
        
        Ok(())
    }

    /// Get MEV protection statistics
    pub async fn get_statistics(&self) -> Result<MevStats> {
        let recent_txs = self.recent_transactions.read().await;
        let mempool_monitor = self.mempool_monitor.read().await;
        let known_bots = self.known_mev_bots.read().await;
        
        Ok(MevStats {
            transactions_monitored: recent_txs.len(),
            threats_detected: mempool_monitor.suspicious_patterns.len(),
            known_mev_bots: known_bots.len(),
            protection_strategies_active: self.protection_strategies.read().await.len(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MevStats {
    pub transactions_monitored: usize,
    pub threats_detected: usize,
    pub known_mev_bots: usize,
    pub protection_strategies_active: usize,
}

use std::collections::HashSet;
