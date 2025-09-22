use anyhow::Result;
use std::sync::Arc;
use crate::chains::ChainManager;

pub struct CompoundManager {
    chain_manager: Arc<ChainManager>,
}

impl CompoundManager {
    pub async fn new(chain_manager: Arc<ChainManager>) -> Result<Self> {
        Ok(Self { chain_manager })
    }
}
