use std::sync::Arc;

use anyhow::Result;
#[cfg(feature = "redis")]
use cache::tx_caching::TxCaching;
use ethers::types::{Block, H256};
use tokio::sync::{broadcast::Receiver, Mutex};

pub struct BlockProcessor<C> {
    pub tx_cache: Arc<Mutex<C>>,
    // pub block_storage: Arc<Mutex<D>>,
    pub receiver: Arc<Mutex<Receiver<Block<H256>>>>,
}

impl<C> BlockProcessor<C> {
    pub fn new(
        tx_cache: Arc<Mutex<C>>,
        // block_storage: Arc<Mutex<D>>,
        receiver: Receiver<Block<H256>>,
    ) -> Self {
        Self {
            tx_cache,
            // block_storage,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    pub async fn process(&self) -> Result<()> {
        let mut receiver = self.receiver.lock().await;

        while let Ok(_block) = receiver.recv().await {
            // self.block_storage.lock().await.store(block).await?;
            // self.tx_cache.lock().await.store(block).await?;
            #[cfg(feature = "redis")]
            {
                let mut tx_cache = self.tx_cache.lock().await;
                let mut tx_caching = tx_cache.get_mut().unwrap();
                tx_caching.store(_block).await?;
            }
        }

        Ok(())
    }
}
