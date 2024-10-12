use ansi_term::Colour;
use anyhow::Result;
use block_explorer::blockexplorerapi::BlockExplorerApi;
use ethers::types::{Transaction, H160};
use ethers::{abi::Abi, types::Address};
use log::{debug, trace};
use serde::Deserialize;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

use crate::factory::get_factory;
use crate::factory::Factory;
use crate::{decode_debug, DecodableTransaction};

#[derive(Clone, Debug, PartialEq)]
pub struct RouterAddress {
    pub address: Address,
    pub abi: Abi,
}

impl PartialEq<H160> for RouterAddress {
    fn eq(&self, other: &H160) -> bool {
        self.address == *other
    }
}

impl Display for RouterAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "RouterAddress {{ address: {} }}", self.address)
    }
}

#[derive(Clone, Debug)]
pub struct Router {
    pub addresses: Vec<RouterAddress>,
    pub factory: Factory,
    pub name: String,
    pub version: u8,
}

impl Display for Router {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Router {{ addresses: [{}], name: {}, version: {} }}",
            self.addresses
                .iter()
                .map(|address| format!("{}", address.address))
                .collect::<Vec<String>>()
                .join(", "),
            self.name,
            self.version
        )
    }
}

impl Router {
    pub fn get_address(&self, address: Address) -> Option<&RouterAddress> {
        self.addresses.iter().find(|a| **a == address)
    }
}

impl DecodableTransaction for Router {
    async fn decode_tx(&self, tx: Transaction) -> Result<()> {
        let data = tx.input.0.to_vec();

        if tx.to.is_none() {
            return Ok(());
        }

        let abi = self
            .addresses
            .iter()
            .find(|a| *a == &tx.to.unwrap())
            .unwrap()
            .abi
            .clone();

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

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct RouterSettings {
    pub name: String,
    pub version: u8,
    pub factory: String,
    pub addresses: Vec<String>,
}

/// Get a router from an indexer
///
/// # Arguments
///
/// * `indexer` - An indexer that implements the IndexerAPI trait
/// * `addresses` - A vector of addresses of the routers
/// * `name` - The name of the router
/// * `version` - The version of the router
///
/// # Returns
///
/// A router
pub async fn get_router<T: BlockExplorerApi>(
    indexer: &T,
    addresses: Vec<String>,
    factory: String,
    name: String,
    version: u8,
) -> Result<Router> {
    let factory = get_factory(indexer, factory, name.clone(), version).await?;

    let router_addresses = {
        let mut ra = Vec::<RouterAddress>::new();

        for address in addresses.clone().iter() {
            let address = address.parse::<Address>()?;
            let abi = indexer.get_abi(address).await?;

            ra.push(RouterAddress { address, abi });
        }

        ra
    };

    Ok(Router {
        addresses: router_addresses,
        factory,
        name,
        version,
    })
}
