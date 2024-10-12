#![feature(async_fn_in_trait)]

use anyhow::Result;
use ethers::{abi::Abi, types::H160};
use log::debug;
use std::path::PathBuf;

pub mod blockexplorerapi;
pub mod etherscan;

pub fn init_cache(cache_path: PathBuf) -> Result<()> {
    if !cache_path.exists() {
        debug!("Creating cache directory");
        std::fs::create_dir(cache_path)?;
    }

    Ok(())
}

pub fn check_cache(cache_path: PathBuf, cache_ttl: usize, contract_address: H160) -> Result<Abi> {
    let cache_file = cache_path.join(format!("{:?}.json", contract_address));
    if cache_file.exists() {
        let metadata = std::fs::metadata(cache_file.clone())?;
        let modified = metadata.modified()?;
        let elapsed = modified.elapsed()?;

        if elapsed.as_secs() < cache_ttl as u64 {
            debug!("Using cached ABI for {}", contract_address);
            let abi = std::fs::read_to_string(cache_file)?;

            Ok(serde_json::from_str(&abi)?)
        } else {
            Err(anyhow::anyhow!("Cache for {} is too old", contract_address))
        }
    } else {
        Err(anyhow::anyhow!("ABI not found in cache"))
    }
}

pub fn cache_abi(cache_path: PathBuf, contract_address: H160, abi: Abi) -> Result<()> {
    debug!("Caching ABI for {}", contract_address.to_string());
    std::fs::write(
        cache_path.join(format!("{:?}.json", contract_address)),
        serde_json::to_string(&abi)?,
    )?;

    Ok(())
}
