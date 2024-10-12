use anyhow::Result;
use ethers::{prelude::*, providers::Ws};
use log::{info, trace, warn};

use std::sync::Arc;
use tokio::sync::broadcast::Sender;

pub struct BlockWatcher {
    pub ws_url: Arc<String>,
    pub sender: Arc<Sender<ethers::types::Block<H256>>>,
}

impl BlockWatcher {
    pub fn new(ws_url: String, channel: Sender<ethers::types::Block<H256>>) -> Self {
        Self {
            ws_url: Arc::new(ws_url),
            sender: Arc::new(channel),
        }
    }

    pub async fn watch(&self) -> Result<()> {
        let provider = Provider::<Ws>::connect(self.ws_url.as_ref()).await?;
        let mut block_stream = provider.subscribe_blocks().await?;

        info!("Connected to {}, listening for blocks", self.ws_url);

        while let Some(block) = block_stream.next().await {
            if let Some(hash) = block.hash {
                if let Some(block) = provider.get_block(hash).await? {
                    info!(
                        "{} ({:?})",
                        ansi_term::Colour::Cyan.paint("Block"),
                        hash.clone()
                    );

                    self.send(block).await?;
                }
            } else {
                warn!("Block has no hash")
            }
        }

        Ok(())
    }

    async fn send(&self, block: ethers::types::Block<H256>) -> Result<()> {
        trace!("Sending block: {:?}", block.hash);
        self.sender.send(block)?;
        Ok(())
    }
}
