use anyhow::Result;
use std::sync::Arc;

use crate::chains::ChainManager;

pub mod uniswap;
pub mod sushiswap;
pub mod aggregator;

pub struct DexManager {
    chain_manager: Arc<ChainManager>,
    uniswap: uniswap::UniswapV3Manager,
    sushiswap: sushiswap::SushiSwapManager,
    aggregator: aggregator::DexAggregator,
}

impl DexManager {
    pub async fn new(chain_manager: Arc<ChainManager>) -> Result<Self> {
        let uniswap = uniswap::UniswapV3Manager::new(chain_manager.clone()).await?;
        let sushiswap = sushiswap::SushiSwapManager::new(chain_manager.clone()).await?;
        let aggregator = aggregator::DexAggregator::new().await?;

        Ok(Self {
            chain_manager,
            uniswap,
            sushiswap,
            aggregator,
        })
    }
}
