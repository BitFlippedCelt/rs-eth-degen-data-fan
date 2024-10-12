use anyhow::Result;
use ethers::{abi::Abi, addressbook::Chain, etherscan::Client, types::H160};

use crate::blockexplorerapi::BlockExplorerApi;

#[derive(Clone, Debug)]
pub struct EtherscanBlockExplorer {
    cache_path: std::path::PathBuf,
    cache_ttl: usize,
}

impl BlockExplorerApi for EtherscanBlockExplorer {
    fn new(cache_path: String, cache_age: Option<usize>) -> Self {
        let cache_path = std::path::PathBuf::from(cache_path);
        super::init_cache(cache_path.clone()).expect("Failed to initialize cache");

        Self {
            cache_path,
            cache_ttl: cache_age.unwrap_or(3600),
        }
    }

    async fn get_abi(&self, contract_address: H160) -> Result<Abi> {
        let abi = super::check_cache(self.cache_path.clone(), self.cache_ttl, contract_address);

        match abi {
            Ok(abi) => Ok(abi),
            Err(_) => {
                let etherscan_api_key =
                    dotenv::var("ETHERSCAN_API_KEY").expect("ETHERSCAN_API_KEY missing");
                // Fetch the ABI from Etherscan
                let etherscan = Client::new(Chain::Mainnet, etherscan_api_key)
                    .expect("Could not create etherscan client");

                // let rt = tokio::runtime::Runtime::new()?;
                // let abi = rt.block_on(etherscan.contract_abi(contract_address))?;
                let abi = etherscan.contract_abi(contract_address).await?;

                super::cache_abi(self.cache_path.clone(), contract_address, abi.clone())?;
                Ok(abi)
            }
        }
    }
}
