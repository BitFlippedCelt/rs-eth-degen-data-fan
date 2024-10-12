use anyhow::Result;
use ethers::{prelude::*, providers::Ws, types::Transaction};
use log::{debug, info, trace};
use std::sync::Arc;
use tokio::sync::broadcast::Sender;

pub struct TxPool {
    pub ws_url: Arc<String>,
    pub sender: Arc<Sender<Transaction>>,
}

impl TxPool {
    pub fn new(ws_url: String, channel: Sender<Transaction>) -> Self {
        Self {
            ws_url: Arc::new(ws_url),
            sender: Arc::new(channel),
        }
    }

    pub async fn watch(&self) -> Result<()> {
        let provider = Provider::<Ws>::connect(self.ws_url.as_ref()).await?;
        let mut tx_stream = provider.subscribe_pending_txs().await?;

        info!("Connected to {}, listening for transactions", self.ws_url);

        while let Some(hash) = tx_stream.next().await {
            if let Some(tx) = provider.get_transaction(hash).await? {
                trace!("Transaction: {:?}", tx.hash.clone());

                self.send(tx).await?;
            } else {
                debug!("Failed to get transaction {:?}", hash);
            }
        }

        Ok(())
    }

    async fn send(&self, tx: Transaction) -> Result<()> {
        trace!("Sending tx: {:?}", tx.hash);
        self.sender.send(tx)?;
        Ok(())
    }
}
