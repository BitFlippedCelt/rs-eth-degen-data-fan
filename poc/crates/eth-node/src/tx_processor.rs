use std::sync::Arc;

use ansi_term::Colour;
use anyhow::Result;
use dex::router::Router;
use ethers::types::Transaction;
use log::{info, trace};
use tokio::sync::{
    broadcast::{Receiver, Sender},
    Mutex,
};

use crate::check_contract_creation;

pub struct TxProcessor {
    pub receiver: Arc<Mutex<Receiver<Transaction>>>,
    pub sender: Arc<Mutex<Sender<Transaction>>>,
    pub routers: Vec<Router>,
}

impl TxProcessor {
    pub fn new(
        receiver: Receiver<Transaction>,
        sender: Sender<Transaction>,
        routers: Vec<Router>,
    ) -> Self {
        Self {
            receiver: Arc::new(Mutex::new(receiver)),
            sender: Arc::new(Mutex::new(sender)),
            routers,
        }
    }

    pub async fn process(&self) -> Result<()> {
        let mut receiver = self.receiver.lock().await;
        let sender = self.sender.lock().await;

        while let Ok(tx) = receiver.recv().await {
            trace!("Received tx: {}", tx.hash);
            trace!("{}", serde_json::to_string_pretty(&tx)?);

            if let Some(to) = tx.to {
                trace!("TX to: {:?}", to);

                if let Some(router) = self
                    .routers
                    .iter()
                    .find(|r| r.addresses.iter().any(|a| *a == to))
                {
                    info!(
                        "TX Pool ({}) to: {}",
                        Colour::White.bold().paint(format!("{:?}", tx.hash)),
                        Colour::Green.paint(router.to_string())
                    );

                    sender.send(tx.clone())?;
                } else if let Some(router) = self.routers.iter().find(|r| r.factory.address == to) {
                    info!(
                        "TX Pool ({}) to: {}",
                        Colour::White.bold().paint(format!("{:#?}", tx.hash)),
                        Colour::Blue.paint(router.factory.to_string())
                    );

                    sender.send(tx.clone())?;
                } else if check_contract_creation(to)?.is_none() {
                    info!(
                        "TX Pool ({}) Create Contract",
                        Colour::Red.bold().paint(format!("{:?}", tx.hash))
                    );
                    sender.send(tx)?;
                } else {
                    trace!("TX to: Unknown");
                }
            } else {
                trace!("TX to: None");
            }
        }

        Ok(())
    }
}
