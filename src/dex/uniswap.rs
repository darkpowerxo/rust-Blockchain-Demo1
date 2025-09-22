use anyhow::Result;
use std::sync::Arc;
use crate::chains::ChainManager;

pub struct UniswapV3Manager {
    chain_manager: Arc<ChainManager>,
}

impl UniswapV3Manager {
    pub async fn new(chain_manager: Arc<ChainManager>) -> Result<Self> {
        Ok(Self { chain_manager })
    }
}
