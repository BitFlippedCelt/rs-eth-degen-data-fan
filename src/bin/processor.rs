use anyhow::Result;
use dotenv::dotenv;
use ethers::{
    abi::Abi,
    contract::{abigen, ContractFactory},
    providers::{Http, Provider, ProviderExt},
    types::Address,
};
use ethers::{
    providers::StreamExt,
    types::{Block, Transaction, H256},
};
use log::{debug, error, info};

#[tokio::main]
async fn main() -> Result<()> {
    // Load the .env file
    dotenv().ok();

    // Configure the logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Spawn a task to process blocks
    let blocks = tokio::spawn(process_blocks());

    // Spawn a task to process transactions
    let txs = tokio::spawn(process_txs());

    // Wait for the tasks to finish
    blocks.await??;
    txs.await??;

    Ok(())
}

async fn process_blocks() -> Result<()> {
    // Connect to the NATS server and subscribe to the `eth_blocks` subject
    let nc = async_nats::connect(
        dotenv::var("NATS_URL")
            .expect("Please set the `NATS_URL` environment variable to point to a NATS server"),
    )
    .await?;
    let mut sub = nc.subscribe("eth_blocks".into()).await?;

    info!("Waiting for blocks...");

    // Create a new Scylla storage engine
    let storage = bfc_degen::storage::scylla::init_blocks_session().await?;

    // Process messages
    while let Some(msg) = sub.next().await {
        // Decode the message
        let block: Block<H256> = serde_json::from_slice(&msg.payload)?;

        // Store the block in the database
        let block = block.clone();
        match bfc_degen::storage::scylla::store_block(&storage, block.clone()).await {
            Ok(_) => debug!("Stored block: {}", block.number.unwrap()),
            Err(e) => error!("Error storing block: {}", e),
        };
    }

    Ok(())
}

async fn process_txs() -> Result<()> {
    // Connect to the NATS server and subscribe to the `eth_txs` subject
    let nc = async_nats::connect(
        dotenv::var("NATS_URL")
            .expect("Please set the `NATS_URL` environment variable to point to a NATS server"),
    )
    .await?;
    let mut sub = nc.subscribe("eth_txs".into()).await?;

    info!("Waiting for transactions...");

    // Create a new Scylla storage engine
    let storage = bfc_degen::storage::scylla::init_tx_session().await?;

    let provider = Provider::<Http>::connect(dotenv::var("ETH_WS_URL")).await;

    // Process messages
    while let Some(msg) = sub.next().await {
        // Decode the message
        let tx: Transaction = serde_json::from_slice(&msg.payload)?;

        let provider = provider.clone();
        let abi = provider.get_contract_abi(tx.to.unwrap()).await?;
        // Store the transaction in the database
        // match bfc_degen::storage::scylla::store_tx(&storage, tx.clone()).await {
        //     Ok(_) => debug!("Stored tx: {}", tx.hash),
        //     Err(e) => error!("Error storing tx: {}", e),
        // };
    }

    Ok(())
}
