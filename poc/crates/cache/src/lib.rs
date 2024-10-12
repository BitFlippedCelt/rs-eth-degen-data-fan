#![feature(async_fn_in_trait)]

use std::sync::Arc;

use anyhow::Result;
use ethers::types::Transaction;
use log::info;
use tokio::sync::{broadcast::Receiver, Mutex};
use tx_caching::TxCaching;

#[cfg(feature = "redis")]
pub mod redis;
pub mod tx_caching;

pub async fn tx_cache_updates<C: TxCaching>(
    tx_cache: Arc<Mutex<C>>,
    mut receiver: Arc<Receiver<Transaction>>,
) -> Result<()> {
    let receiver = Arc::get_mut(&mut receiver).unwrap();
    let mut tx_cache = tx_cache.lock().await;

    info!("Starting tx cache updates...");

    while let Ok(tx) = receiver.recv().await {
        info!("Updating tx cache for tx {}", tx.hash());
        tx_cache.cache(tx).await?;
    }

    Ok(())
}
