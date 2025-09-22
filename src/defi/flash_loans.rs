use anyhow::Result;
use std::sync::Arc;
use crate::chains::ChainManager;

pub struct FlashLoanManager {
    chain_manager: Arc<ChainManager>,
}

impl FlashLoanManager {
    pub async fn new(chain_manager: Arc<ChainManager>) -> Result<Self> {
        Ok(Self { chain_manager })
    }
}
