use core::option::Option;

use anyhow::Result;
use ethers::{abi::Abi, types::H160};

pub trait BlockExplorerApi {
    /// Creates a new BlockExplorerApi
    fn new(cache_path: String, cache_ttl: Option<usize>) -> Self;

    /// Fetches the ABI for a contract address from Indexer
    async fn get_abi(&self, contract_address: H160) -> Result<Abi>;
}
