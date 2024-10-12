use anyhow::Result;
use ethers::types::Transaction;

pub trait TxCaching {
    /// Cache a transaction hash
    ///
    /// # Errors
    ///
    /// This function will return an error if the transaction hash could not be cached
    async fn cache(&mut self, tx: Transaction) -> Result<()>;

    /// Check if a transaction hash is cached
    ///
    /// # Errors
    ///
    /// This function will return an error if the transaction hash could not be checked
    async fn is_cached(&mut self, tx: Transaction) -> Result<bool>;

    /// Delete a transaction hash from the cache
    ///
    /// # Errors
    ///
    /// This function will return an error if the transaction hash could not be deleted
    async fn delete(&mut self, tx: Transaction) -> Result<()>;
}
