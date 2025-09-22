use anyhow::Result;
use std::sync::Arc;
use ethers::providers::{Provider, Http};
use tracing::info;

pub mod chains;
pub mod defi;
pub mod dex;
pub mod docs;
pub mod health;
pub mod models;
pub mod portfolio;
pub mod security;
pub mod wallets;

use crate::chains::ChainManager;
use crate::dex::DexManager;
use crate::wallets::WalletManager;
use crate::defi::DefiManager;
use crate::analytics::AnalyticsService;
use crate::security::SecurityManager;

/// Central application state containing all managers and services
#[derive(Clone)]
pub struct ApiState {
    pub chain_manager: Arc<ChainManager>,
    pub dex_manager: Arc<DexManager>,
    pub wallet_manager: Arc<WalletManager>,
    pub defi_manager: Arc<DefiManager>,
    pub analytics: Arc<AnalyticsService>,
    pub security: Arc<SecurityManager>,
}

impl ApiState {
    pub async fn new(config: config::Config) -> Result<Self> {
        info!("Initializing API state with configuration");
        
        // Initialize all managers with error tolerance for demo mode
        let wallet_manager = Arc::new(WalletManager::new(None).await?);
        let analytics = Arc::new(AnalyticsService::new(&config).await?);
        
        // Create demo/empty managers to avoid RPC connection issues
        let chain_manager = Arc::new(ChainManager::new_demo().await?);
        let dex_manager = Arc::new(DexManager::new_demo().await?);
        let defi_manager = Arc::new(DefiManager::new_demo().await?);
        let security = Arc::new(SecurityManager::new_demo().await?);

        Ok(Self {
            chain_manager,
            dex_manager,
            wallet_manager,
            defi_manager,
            analytics,
            security,
        })
    }
}

pub fn routes() -> axum::Router<Arc<ApiState>> {
    axum::Router::new()
        .nest("/docs", docs::routes())
        .nest("/health", health::routes())
        .nest("/portfolio", portfolio::routes())
        .nest("/dex", dex::routes())
        .nest("/defi", defi::routes())
        .nest("/security", security::routes())
        .nest("/wallets", wallets::routes())
        .nest("/chains", chains::routes())
}
