#![feature(async_fn_in_trait)]

use ansi_term::Colour;
use anyhow::Result;
use ethers::types::Transaction;

pub mod dex;
pub mod factory;
pub mod router;

pub trait DecodableTransaction {
    async fn decode_tx(&self, tx: Transaction) -> Result<()>;
}

pub fn decode_debug(f: &ethers::abi::Function, tokens: Vec<ethers::abi::Token>) -> String {
    format!("{} ({})", Colour::White.bold().paint(f.name.clone()), {
        let args = f
            .outputs
            .iter()
            .zip(tokens.iter())
            .map(|(i, d)| {
                format!(
                    "{}: {}",
                    Colour::Green.paint(i.name.clone()),
                    Colour::Green.dimmed().paint(d.to_string())
                )
            })
            .collect::<Vec<String>>()
            .join(", ");
        args
    })
}
