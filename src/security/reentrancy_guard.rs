use anyhow::Result;
use ethers::{prelude::*, types::transaction::eip2718::TypedTransaction};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ReentrancyGuard {
    active_transactions: Arc<RwLock<HashSet<H256>>>,
    suspicious_patterns: Vec<Vec<u8>>,
}

impl ReentrancyGuard {
    pub fn new() -> Self {
        let mut suspicious_patterns = Vec::new();
        
        // Add known reentrancy patterns (function selectors)
        suspicious_patterns.push(vec![0xa9, 0x05, 0x9c, 0xbb]); // transfer
        suspicious_patterns.push(vec![0x23, 0xb8, 0x72, 0xdd]); // transferFrom
        suspicious_patterns.push(vec![0x2e, 0x1a, 0x7d, 0x4d]); // call

        Self {
            active_transactions: Arc::new(RwLock::new(HashSet::new())),
            suspicious_patterns,
        }
    }

    pub async fn check_transaction(&self, tx: &TypedTransaction) -> Result<()> {
        // Check for suspicious function calls in transaction data
        if let Some(data) = tx.data() {
            if data.len() >= 4 {
                let function_selector = &data[0..4];
                
                for pattern in &self.suspicious_patterns {
                    if function_selector == pattern.as_slice() {
                        // Additional checks for potential reentrancy
                        self.analyze_call_pattern(data)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn analyze_call_pattern(&self, data: &[u8]) -> Result<()> {
        // Analyze the call data for reentrancy patterns
        // This is a simplified check - production would be more sophisticated
        
        if data.len() > 100 {
            // Check for multiple external calls in the same transaction
            let mut call_count = 0;
            for window in data.windows(4) {
                if window == [0x2e, 0x1a, 0x7d, 0x4d] { // call function selector
                    call_count += 1;
                }
            }
            
            if call_count > 3 {
                return Err(anyhow::anyhow!("Potential reentrancy detected: multiple calls"));
            }
        }

        Ok(())
    }

    pub async fn enter_transaction(&self, tx_hash: H256) -> Result<()> {
        let mut active = self.active_transactions.write().await;
        if active.contains(&tx_hash) {
            return Err(anyhow::anyhow!("Transaction already active"));
        }
        active.insert(tx_hash);
        Ok(())
    }

    pub async fn exit_transaction(&self, tx_hash: H256) -> Result<()> {
        let mut active = self.active_transactions.write().await;
        active.remove(&tx_hash);
        Ok(())
    }
}
