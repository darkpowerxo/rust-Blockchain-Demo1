use anyhow::Result;
use std::sync::Arc;
use crate::chains::ChainManager;

pub struct SushiSwapManager {
    chain_manager: Arc<ChainManager>,
}

impl SushiSwapManager {
    pub async fn new(chain_manager: Arc<ChainManager>) -> Result<Self> {
        Ok(Self { chain_manager })
    }
}
