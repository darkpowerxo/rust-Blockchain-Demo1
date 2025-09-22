use anyhow::Result;
use std::sync::Arc;

pub mod health;
pub mod portfolio;
pub mod dex;
pub mod defi;
pub mod models;

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
        // Initialize all managers and services
        let chain_manager = Arc::new(ChainManager::new(&config).await?);
        let wallet_manager = Arc::new(WalletManager::new(&config).await?);
        let dex_manager = Arc::new(DexManager::new(chain_manager.clone()).await?);
        let defi_manager = Arc::new(DefiManager::new(chain_manager.clone()).await?);
        let analytics = Arc::new(AnalyticsService::new(&config).await?);
        let security = Arc::new(SecurityManager::new().await?);

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
        .nest("/health", health::routes())
        .nest("/portfolio", portfolio::routes())
        .nest("/dex", dex::routes())
        .nest("/defi", defi::routes())
}
