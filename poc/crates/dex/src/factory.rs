use ansi_term::Colour;
use anyhow::Result;
use block_explorer::blockexplorerapi::BlockExplorerApi;
use ethers::types::Transaction;
use ethers::{abi::Abi, types::Address};
use log::debug;
use log::trace;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

use crate::{decode_debug, DecodableTransaction};

#[derive(Clone, Debug)]
pub struct Factory {
    pub address: Address,
    pub abi: Abi,
    pub name: String,
    pub version: u8,
}

impl Factory {
    pub fn new(address: Address, abi: Abi, name: String, version: u8) -> Self {
        Self {
            address,
            abi,
            name,
            version,
        }
    }
}

impl DecodableTransaction for Factory {
    async fn decode_tx(&self, tx: Transaction) -> Result<()> {
        let data = tx.input.0.to_vec();

        if tx.to.is_none() {
            return Ok(());
        }

        let abi = self.abi.clone();
        trace!("Decoding TX: {}", tx.hash);
        abi.functions.iter().for_each(|(name, functions)| {
            functions.iter().for_each(|f| {
                if let Ok(tokens) = f.decode_input(&data) {
                    debug!(
                        "{} {}",
                        Colour::Yellow.bold().paint("Input: "),
                        decode_debug(f, tokens)
                    );
                } else {
                    trace!("Failed to decode TX output with function: {}", name);
                }

                if let Ok(tokens) = f.decode_output(&data) {
                    debug!(
                        "{} {}",
                        Colour::Purple.bold().paint("Output: "),
                        decode_debug(f, tokens)
                    );
                } else {
                    trace!("Failed to decode TX output with function: {}", name);
                }
            });
        });

        Ok(())
    }
}

///
impl Display for Factory {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Factory {{ address: {}, name: {}, version: {} }}",
            self.address, self.name, self.version
        )
    }
}

/// Get a factory from an indexer
///
/// # Arguments
///
/// * `indexer` - An indexer that implements the IndexerAPI trait
/// * `address` - The address of the factory
/// * `name` - The name of the factory
/// * `version` - The version of the factory
///
/// # Returns
///
/// A factory
pub async fn get_factory<T: BlockExplorerApi>(
    indexer: &T,
    address: String,
    name: String,
    version: u8,
) -> Result<Factory> {
    let address = address.parse::<Address>()?;
    let abi = indexer.get_abi(address).await?;

    Ok(Factory {
        address,
        abi,
        name,
        version,
    })
}

/// Get a vector of factories from an indexer
///
/// # Arguments
///
/// * `indexer` - An indexer that implements the IndexerAPI trait
/// * `addresses` - A vector of addresses of the factories
/// * `name` - The name of the factory
/// * `version` - The version of the factory
///
/// # Returns
///
/// A vector of factories
pub async fn get_factories<T: BlockExplorerApi>(
    indexer: T,
    addresses: Vec<String>,
    name: String,
    version: u8,
) -> Result<Vec<Factory>> {
    let mut factories: Vec<Factory> = Vec::new();

    for address in addresses {
        let factory = get_factory(&indexer, address, name.clone(), version).await?;
        factories.push(factory);
    }

    Ok(factories)
}
