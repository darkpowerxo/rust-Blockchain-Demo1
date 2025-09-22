use anyhow::Result;
use std::sync::Arc;

use crate::chains::ChainManager;

pub mod aave;
pub mod compound;
pub mod flash_loans;

pub struct DefiManager {
    chain_manager: Arc<ChainManager>,
    aave: aave::AaveManager,
    compound: compound::CompoundManager,
    flash_loans: flash_loans::FlashLoanManager,
}

impl DefiManager {
    pub async fn new(chain_manager: Arc<ChainManager>) -> Result<Self> {
        let aave = aave::AaveManager::new(chain_manager.clone()).await?;
        let compound = compound::CompoundManager::new(chain_manager.clone()).await?;
        let flash_loans = flash_loans::FlashLoanManager::new(chain_manager.clone()).await?;

        Ok(Self {
            chain_manager,
            aave,
            compound,
            flash_loans,
        })
    }
}
