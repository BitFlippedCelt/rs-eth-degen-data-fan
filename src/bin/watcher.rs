use ansi_term::Colour;
use anyhow::Result;
use dotenv::dotenv;
use ethers::prelude::*;
use log::{debug, info};

#[tokio::main]
async fn main() -> Result<()> {
    // Load the .env file
    dotenv().ok();

    // Configure the logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // ENV variables
    let http_url = dotenv::var("ETH_HTTP_URL")
        .expect("Please set the `ETH_HTTP_URL` environment variable to point to an Ethereum node");
    let wss_url = dotenv::var("ETH_WS_URL")
        .expect("Please set the `ETH_WS_URL` environment variable to point to an Ethereum node");
    let nats_url = dotenv::var("NATS_URL")
        .expect("Please set the `NATS_URL` environment variable to point to a NATS server");

    // Connect to the NATS server
    info!("Connecting to NATS server at {}", nats_url);
    let nc = async_nats::connect(&nats_url)
        .await
        .expect("Could not connect to NATS server");

    // Check the sync state of the node
    // check_sync_state(http_url.to_owned()).await?;

    // Spawn a task to process blocks
    let blocks = tokio::spawn(process_blocks(wss_url.to_owned(), nc.to_owned()));

    // Spawn a task to process transactions
    let txs = tokio::spawn(process_txs(wss_url.to_owned(), nc.to_owned()));

    // Wait for the tasks to finish
    blocks.await??;
    txs.await??;

    Ok(())
}

async fn create_provider(wss_url: String) -> Result<Provider<Ws>> {
    let provider = Provider::<Ws>::connect(wss_url.to_owned()).await?;

    info!("Connected to {}", wss_url);
    Ok(provider)
}

async fn check_sync_state(http_url: String) -> Result<()> {
    let provider = Provider::<Http>::connect(&http_url).await;

    let mut syncing = true;
    while syncing {
        let sync_status = provider.syncing().await.expect("Could not get sync state");
        match sync_status {
            SyncingStatus::IsFalse => syncing = false,
            SyncingStatus::IsSyncing(status) => {
                let current = status.current_block.to_string().parse::<f64>().unwrap();
                let highest = status.highest_block.to_string().parse::<f64>().unwrap();
                let pct = (current / highest) * 100.0;

                log::info!(
                    "Node syncing: {current} / {highest}, {pct:.2}% complete",
                    current = status.current_block,
                    highest = status.highest_block,
                    pct = pct
                );

                let storage_gb = status
                    .synced_storage_bytes
                    .unwrap()
                    .to_string()
                    .parse::<f64>()
                    .unwrap()
                    / 1_000_000_000.0;
                log::info!("Storage: {:.2} GB", storage_gb);

                let bytecode_bytes = status
                    .synced_bytecode_bytes
                    .unwrap()
                    .to_string()
                    .parse::<f64>()
                    .unwrap()
                    / 1_000_000_000.0;
                log::info!("Bytecode: {:.2} GB", bytecode_bytes);

                let account_bytes = status
                    .synced_account_bytes
                    .unwrap()
                    .to_string()
                    .parse::<f64>()
                    .unwrap()
                    / 1_000_000_000.0;
                log::info!("Accounts: {:.2} GB", account_bytes);

                // Sleep 30 seconds
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            }
        }
    }

    Ok(())
}

async fn process_blocks(wss_url: String, nats_client: async_nats::Client) -> Result<()> {
    info!("Connecting to {}", wss_url);
    let provider = create_provider(wss_url)
        .await
        .expect("Could not connect to Ethereum node");

    info!("Waiting for blocks...");
    let mut block_stream = provider.subscribe_blocks().await?;
    while let Some(block) = block_stream.next().await {
        if let Ok(json) = serde_json::to_string(&block) {
            debug!(
                "{} {}",
                Colour::Green.bold().paint("Block:"),
                block.number.unwrap()
            );
            nats_client
                .publish("eth_blocks".into(), json.into())
                .await?;
        }
    }
    Ok(())
}

async fn process_txs(wss_url: String, nats_client: async_nats::Client) -> Result<()> {
    let provider = create_provider(wss_url).await?;

    info!("Waiting for transactions...");
    let mut tx_stream = provider.subscribe_pending_txs().await?;
    while let Some(hash) = tx_stream.next().await {
        if let Some(tx) = provider.get_transaction(hash).await? {
            if let Ok(json) = serde_json::to_string(&tx) {
                debug!("{} {}", Colour::Blue.bold().paint("Transaction:"), tx.hash);
                nats_client.publish("eth_txs".into(), json.into()).await?;
            }
        }
    }
    Ok(())
}
