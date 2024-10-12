use std::sync::Arc;

use anyhow::Result;
use ethers::types::Transaction;
use log::info;
use tokio::sync::{broadcast::Receiver, Mutex};

pub trait TxStorage<T> {
    /// Store a transaction
    ///
    /// # Errors
    ///
    /// This function will return an error if the transaction could not be stored
    async fn store(&mut self, tx: T) -> Result<()>;

    /// Check if a transaction is stored
    ///
    /// # Errors
    ///
    /// This function will return an error if the transaction could not be checked
    async fn is_stored(&mut self, tx: T) -> Result<bool>;

    /// Delete a transaction from the store
    ///
    /// # Errors
    ///
    /// This function will return an error if the transaction could not be deleted
    async fn delete(&mut self, tx: T) -> Result<()>;
}

pub async fn tx_store<C: TxStorage<Transaction>>(
    tx_storage: Arc<Mutex<C>>,
    mut receiver: Arc<Receiver<Transaction>>,
) -> Result<()> {
    let receiver = Arc::get_mut(&mut receiver).unwrap();
    let mut tx_storage = tx_storage.lock().await;

    info!("Starting tx storage updates...");

    while let Ok(tx) = receiver.recv().await {
        tx_storage.store(tx).await?;
    }

    Ok(())
}
