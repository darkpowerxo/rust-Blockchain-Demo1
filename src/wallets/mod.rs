use anyhow::Result;
use ethers::{
    prelude::*,
    signers::{LocalWallet, Signer, Wallet, coins_bip39::English},
    types::{Address, Signature, H256, transaction::eip2718::TypedTransaction},
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

pub mod metamask;
pub mod walletconnect;
pub mod ledger;
pub mod multisig;

use crate::security::SecurityManager;

#[derive(Debug, Clone)]
pub enum WalletType {
    MetaMask,
    WalletConnect,
    Ledger,
    LocalWallet,
    MultiSig,
}

#[derive(Debug, Clone)]
pub struct WalletInfo {
    pub address: Address,
    pub wallet_type: WalletType,
    pub chain_id: u64,
    pub is_connected: bool,
    pub balance: Option<U256>,
}

pub struct WalletManager {
    wallets: Arc<RwLock<HashMap<Address, WalletProvider>>>,
    security: Arc<SecurityManager>,
    multisig_manager: multisig::MultiSigManager,
}

pub enum WalletProvider {
    MetaMask(metamask::MetaMaskWallet),
    WalletConnect(walletconnect::WalletConnectProvider),
    Ledger(ledger::LedgerWallet),
    Local(LocalWallet),
    MultiSig(multisig::MultiSigWallet),
}

impl WalletManager {
    pub async fn new(_config: Option<&crate::app_config::Config>) -> Result<Self> {
        let security = Arc::new(SecurityManager::new().await?);
        let multisig_manager = multisig::MultiSigManager::new().await?;

        info!("Initialized WalletManager");

        Ok(Self {
            wallets: Arc::new(RwLock::new(HashMap::new())),
            security,
            multisig_manager,
        })
    }

    pub async fn connect_metamask(&self, chain_id: u64) -> Result<Address> {
        let wallet = metamask::MetaMaskWallet::connect(chain_id).await?;
        let address = wallet.get_address();
        
        let mut wallets = self.wallets.write().await;
        wallets.insert(address, WalletProvider::MetaMask(wallet));
        
        info!("Connected MetaMask wallet: {}", address);
        Ok(address)
    }

    pub async fn connect_walletconnect(&self, project_id: &str) -> Result<Address> {
        let wallet = walletconnect::WalletConnectProvider::connect(project_id).await?;
        let address = wallet.get_address();
        
        let mut wallets = self.wallets.write().await;
        wallets.insert(address, WalletProvider::WalletConnect(wallet));
        
        info!("Connected WalletConnect wallet: {}", address);
        Ok(address)
    }

    pub async fn connect_ledger(&self, _derivation_path: &str) -> Result<Address> {
        let wallet = ledger::LedgerWallet::connect().await?;
        let address = wallet.get_address().unwrap_or_default();
        
        let mut wallets = self.wallets.write().await;
        wallets.insert(address, WalletProvider::Ledger(wallet));
        
        info!("Connected Ledger wallet: {:?}", address);
        Ok(address)
    }

    pub async fn create_local_wallet(&self, private_key: Option<String>) -> Result<Address> {
        let wallet = if let Some(pk) = private_key {
            pk.parse::<LocalWallet>()?
        } else {
            LocalWallet::new(&mut rand::thread_rng())
        };

        let address = wallet.address();
        
        let mut wallets = self.wallets.write().await;
        wallets.insert(address, WalletProvider::Local(wallet));
        
        info!("Created local wallet: {}", address);
        Ok(address)
    }

    pub async fn create_multisig_wallet(
        &self,
        owners: Vec<Address>,
        threshold: u8,
        chain_id: u64,
    ) -> Result<Address> {
        let multisig_wallet = self.multisig_manager
            .create_multisig_wallet(owners, threshold, chain_id)
            .await?;
        
        let address = multisig_wallet.get_address();
        
        let mut wallets = self.wallets.write().await;
        wallets.insert(address, WalletProvider::MultiSig(multisig_wallet));
        
        info!("Created MultiSig wallet: {} with threshold {}", address, threshold);
        Ok(address)
    }

    pub async fn sign_message(&self, address: Address, message: &[u8]) -> Result<Signature> {
        let wallets = self.wallets.read().await;
        let wallet = wallets
            .get(&address)
            .ok_or_else(|| anyhow::anyhow!("Wallet not found: {}", address))?;

        // Validate message before signing
        self.security.validate_message(message).await?;

        match wallet {
            WalletProvider::MetaMask(w) => w.sign_message(message).await,
            WalletProvider::WalletConnect(w) => w.sign_message(message).await,
            WalletProvider::Ledger(w) => w.sign_message(message).await,
            WalletProvider::Local(_w) => {
                // For demo purposes, return a mock signature
                // In production, you'd properly sign the message hash
                Ok(Signature {
                    r: U256::from(1),
                    s: U256::from(1),
                    v: 27,
                })
            }
            WalletProvider::MultiSig(w) => w.sign_message(message).await,
        }
    }

    pub async fn sign_transaction(&self, address: Address, tx: TypedTransaction) -> Result<Signature> {
        let wallets = self.wallets.read().await;
        let wallet = wallets
            .get(&address)
            .ok_or_else(|| anyhow::anyhow!("Wallet not found: {}", address))?;

        // Security validation
        self.security.validate_transaction(&tx).await?;

        match wallet {
            WalletProvider::MetaMask(w) => w.sign_transaction(tx).await,
            WalletProvider::WalletConnect(w) => w.sign_transaction(tx).await,
            WalletProvider::Ledger(w) => w.sign_transaction(tx).await,
            WalletProvider::Local(_w) => {
                // For local wallet, we need to handle the transaction differently
                // This is a simplified version - in production you'd use the proper signing method
                Ok(Signature {
                    r: U256::from(1),
                    s: U256::from(1),
                    v: 27,
                })
            }
            WalletProvider::MultiSig(_w) => {
                // MultiSig transactions require multiple signatures
                // Return a mock signature for demo
                Ok(Signature {
                    r: U256::from(1),
                    s: U256::from(1),
                    v: 27,
                })
            }
        }
    }

    pub async fn get_wallet_info(&self, address: Address) -> Result<WalletInfo> {
        let wallets = self.wallets.read().await;
        let wallet = wallets
            .get(&address)
            .ok_or_else(|| anyhow::anyhow!("Wallet not found: {}", address))?;

        let wallet_type = match wallet {
            WalletProvider::MetaMask(_) => WalletType::MetaMask,
            WalletProvider::WalletConnect(_) => WalletType::WalletConnect,
            WalletProvider::Ledger(_) => WalletType::Ledger,
            WalletProvider::Local(_) => WalletType::LocalWallet,
            WalletProvider::MultiSig(_) => WalletType::MultiSig,
        };

        // In production, would fetch actual balance and chain info
        Ok(WalletInfo {
            address,
            wallet_type,
            chain_id: 1, // Default to mainnet, should be fetched from wallet
            is_connected: true,
            balance: None, // Would be fetched from chain
        })
    }

    pub async fn disconnect_wallet(&self, address: Address) -> Result<()> {
        let mut wallets = self.wallets.write().await;
        
        if let Some(wallet) = wallets.remove(&address) {
            match wallet {
                WalletProvider::MetaMask(mut w) => w.disconnect().await?,
                WalletProvider::WalletConnect(mut w) => w.disconnect().await?,
                WalletProvider::Ledger(mut w) => w.disconnect().await?,
                WalletProvider::Local(_) => {} // Nothing to disconnect
                WalletProvider::MultiSig(_) => {} // Nothing to disconnect
            }
            info!("Disconnected wallet: {}", address);
        }

        Ok(())
    }

    pub async fn list_wallets(&self) -> Vec<WalletInfo> {
        let wallets = self.wallets.read().await;
        let mut wallet_infos = Vec::new();

        for (address, _wallet) in wallets.iter() {
            if let Ok(info) = self.get_wallet_info(*address).await {
                wallet_infos.push(info);
            }
        }

        wallet_infos
    }

    pub async fn batch_sign_transactions(
        &self,
        address: Address,
        transactions: Vec<TypedTransaction>,
    ) -> Result<Vec<Signature>> {
        let mut signatures = Vec::new();

        // Validate all transactions first
        for tx in &transactions {
            self.security.validate_transaction(tx).await?;
        }

        // Sign all transactions
        for tx in transactions {
            let signature = self.sign_transaction(address, tx).await?;
            signatures.push(signature);
        }

        Ok(signatures)
    }
}
