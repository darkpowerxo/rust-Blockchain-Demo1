use anyhow::Result;
use ethers::{
    prelude::*,
    types::{Address, Signature, H256},
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

pub struct MultiSigManager {
    multisig_wallets: Arc<RwLock<HashMap<Address, MultiSigWallet>>>,
}

#[derive(Clone)]
pub struct MultiSigWallet {
    pub address: Address,
    pub owners: Vec<Address>,
    pub threshold: u8,
    pub chain_id: u64,
    pub nonce: u64,
    pending_transactions: Arc<RwLock<HashMap<H256, PendingTransaction>>>,
}

#[derive(Clone)]
pub struct PendingTransaction {
    pub transaction_hash: H256,
    pub to: Address,
    pub value: U256,
    pub data: Vec<u8>,
    pub signatures: HashMap<Address, Signature>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub executed: bool,
}

impl MultiSigManager {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            multisig_wallets: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn create_multisig_wallet(
        &self,
        owners: Vec<Address>,
        threshold: u8,
        chain_id: u64,
    ) -> Result<MultiSigWallet> {
        if threshold == 0 || threshold as usize > owners.len() {
            return Err(anyhow::anyhow!(
                "Invalid threshold: {} for {} owners",
                threshold,
                owners.len()
            ));
        }

        // In production, this would deploy a multisig contract
        // For demo, we'll create a deterministic address based on owners and threshold
        let mut hasher = sha2::Sha256::new();
        use sha2::Digest;
        
        for owner in &owners {
            hasher.update(owner.as_bytes());
        }
        hasher.update(&[threshold]);
        hasher.update(&chain_id.to_le_bytes());
        
        let hash = hasher.finalize();
        let address = Address::from_slice(&hash[0..20]);

        let wallet = MultiSigWallet {
            address,
            owners: owners.clone(),
            threshold,
            chain_id,
            nonce: 0,
            pending_transactions: Arc::new(RwLock::new(HashMap::new())),
        };

        let mut wallets = self.multisig_wallets.write().await;
        wallets.insert(address, wallet.clone());

        info!(
            "Created MultiSig wallet {} with {} owners and threshold {}",
            address,
            owners.len(),
            threshold
        );

        Ok(wallet)
    }

    pub async fn get_wallet(&self, address: Address) -> Result<MultiSigWallet> {
        let wallets = self.multisig_wallets.read().await;
        wallets
            .get(&address)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("MultiSig wallet not found: {}", address))
    }
}

impl MultiSigWallet {
    pub fn get_address(&self) -> Address {
        self.address
    }

    pub async fn propose_transaction(
        &self,
        to: Address,
        value: U256,
        data: Vec<u8>,
        proposer: Address,
    ) -> Result<H256> {
        if !self.owners.contains(&proposer) {
            return Err(anyhow::anyhow!("Proposer is not an owner"));
        }

        // Create transaction hash
        let mut hasher = sha2::Sha256::new();
        use sha2::Digest;
        
        hasher.update(self.address.as_bytes());
        hasher.update(to.as_bytes());
        hasher.update(&value.to_string().as_bytes());  // Convert to string first
        hasher.update(&data);
        hasher.update(&self.nonce.to_le_bytes());
        
        let hash = H256::from_slice(&hasher.finalize());

        let pending_tx = PendingTransaction {
            transaction_hash: hash,
            to,
            value,
            data,
            signatures: HashMap::new(),
            created_at: chrono::Utc::now(),
            executed: false,
        };

        let mut pending_txs = self.pending_transactions.write().await;
        pending_txs.insert(hash, pending_tx);

        info!(
            "Proposed transaction {} for MultiSig wallet {}",
            hash, self.address
        );

        Ok(hash)
    }

    pub async fn sign_transaction(&self, tx_hash: H256, signer: Address) -> Result<()> {
        if !self.owners.contains(&signer) {
            return Err(anyhow::anyhow!("Signer is not an owner"));
        }

        let mut pending_txs = self.pending_transactions.write().await;
        let pending_tx = pending_txs
            .get_mut(&tx_hash)
            .ok_or_else(|| anyhow::anyhow!("Transaction not found"))?;

        if pending_tx.executed {
            return Err(anyhow::anyhow!("Transaction already executed"));
        }

        // In production, this would create a real signature
        // For demo, we'll create a mock signature
        let signature = Signature {
            r: U256::from(1),
            s: U256::from(1),
            v: 27,
        };

        pending_tx.signatures.insert(signer, signature);

        info!(
            "Signed transaction {} by owner {} ({}/{})",
            tx_hash,
            signer,
            pending_tx.signatures.len(),
            self.threshold
        );

        // Check if we have enough signatures to execute
        if pending_tx.signatures.len() >= self.threshold as usize {
            info!("Transaction {} ready for execution", tx_hash);
        }

        Ok(())
    }

    pub async fn execute_transaction(&self, tx_hash: H256, executor: Address) -> Result<H256> {
        if !self.owners.contains(&executor) {
            return Err(anyhow::anyhow!("Executor is not an owner"));
        }

        let mut pending_txs = self.pending_transactions.write().await;
        let pending_tx = pending_txs
            .get_mut(&tx_hash)
            .ok_or_else(|| anyhow::anyhow!("Transaction not found"))?;

        if pending_tx.executed {
            return Err(anyhow::anyhow!("Transaction already executed"));
        }

        if pending_tx.signatures.len() < self.threshold as usize {
            return Err(anyhow::anyhow!(
                "Not enough signatures: {}/{}",
                pending_tx.signatures.len(),
                self.threshold
            ));
        }

        pending_tx.executed = true;

        // In production, this would execute the transaction on-chain
        info!(
            "Executed MultiSig transaction {} from wallet {}",
            tx_hash, self.address
        );

        Ok(tx_hash)
    }

    pub async fn get_pending_transactions(&self) -> Vec<PendingTransaction> {
        let pending_txs = self.pending_transactions.read().await;
        pending_txs
            .values()
            .filter(|tx| !tx.executed)
            .cloned()
            .collect()
    }

    pub async fn revoke_signature(&self, tx_hash: H256, owner: Address) -> Result<()> {
        if !self.owners.contains(&owner) {
            return Err(anyhow::anyhow!("Not an owner"));
        }

        let mut pending_txs = self.pending_transactions.write().await;
        let pending_tx = pending_txs
            .get_mut(&tx_hash)
            .ok_or_else(|| anyhow::anyhow!("Transaction not found"))?;

        if pending_tx.executed {
            return Err(anyhow::anyhow!("Cannot revoke signature from executed transaction"));
        }

        pending_tx.signatures.remove(&owner);
        info!("Revoked signature for transaction {} by owner {}", tx_hash, owner);

        Ok(())
    }

    pub async fn sign_message(&self, message: &[u8]) -> Result<Signature> {
        // For demo purposes, return a mock signature
        // In production, this would require multi-sig consensus
        Ok(Signature {
            r: U256::from(1),
            s: U256::from(1),
            v: 27,
        })
    }
}
