use block_explorer::blockexplorerapi::BlockExplorerApi;

use cache::redis::TxCacheRedis;

use cache::tx_cache_updates;
use eth_node::{block_watcher::BlockWatcher, tx_pool::TxPool, tx_processor::TxProcessor};
use ethers::types::{Block, Transaction, H256};
use lazy_static::lazy_static;
use log::{debug, info, warn};
use poc_eth::settings::Settings;

use redis::IntoConnectionInfo;
use std::sync::Arc;
use storage::tx_store;
use tokio::sync::broadcast;

use tokio::sync::Mutex;

lazy_static! {
    static ref SETTINGS: Settings =
        Settings::new(String::from("sniper")).expect("Failed to load settings");
}

#[tokio::main]
async fn main() {
    let settings = SETTINGS.clone();
    // Setup logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(settings.log.level))
        .init();

    // Create indexer instance
    let indexer = block_explorer::etherscan::EtherscanBlockExplorer::new(settings.cache_path, None);

    // Get routers
    let routers = dex::dex::load_dex_routers(indexer, settings.dex.routers)
        .await
        .expect("Failed to load dex routers");

    // Create channels
    let (block_sender, _block_receiver) = broadcast::channel::<Block<H256>>(100);
    let (tx_pool_sender, tx_pool_receiver) = broadcast::channel::<Transaction>(100);
    let (tx_processor_sender, _tx_processor_receiver) = broadcast::channel::<Transaction>(100);

    // TX Pool monitor
    let tx_pool = Arc::new(TxPool::new(
        settings.ethereum.node_ws.clone(),
        tx_pool_sender.clone(),
    ));

    // TX Pool processor
    let tx_pool_processor = Arc::new(TxProcessor::new(
        tx_pool_receiver,
        tx_processor_sender.clone(),
        routers.clone(),
    ));

    // Block Creation Watcher
    let block_watcher = Arc::new(BlockWatcher {
        ws_url: Arc::new(settings.ethereum.node_ws.clone()),
        sender: Arc::new(block_sender),
    });

    // Spawn a task to process txs

    let tx_cache_receiver = Arc::new(tx_processor_sender.subscribe());
    let tx_store_receiver = Arc::new(tx_processor_sender.subscribe());

    // Create Redis client

    let redis_connection_info: redis::ConnectionInfo =
        settings.redis.into_connection_info().unwrap();

    let redis_client = Arc::new(Mutex::new(
        redis::Client::open(redis_connection_info).expect("Failed to create Redis client"),
    ));

    let cache = Arc::new(Mutex::new(TxCacheRedis::new(redis_client.clone())));

    info!("Starting Sniper Bot...");

    // Spawn tasks

    let tx_cache_handle = tokio::spawn(tx_cache_updates(cache, tx_cache_receiver));
    let tx_store_handle = tokio::spawn(tx_store(tx_store_receiver));
    let tx_processor_handle = tokio::spawn(async move { tx_pool_processor.process().await });
    let tx_pool_handle = tokio::spawn(async move { tx_pool.watch().await });
    let block_watcher_handle = tokio::spawn(async move { block_watcher.watch().await });

    // Wait for ctrl-c and abort all tasks
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C");

    info!("Received Ctrl+C, aborting tasks...");

    block_watcher_handle.abort();
    tx_pool_handle.abort();
    tx_processor_handle.abort();
    tx_store_handle.abort();

    tx_cache_handle.abort();

    // Join threads and log errors
    tokio::select! {
        // res = tx_cache_handle => {
        //     match res {
        //         Ok(_) => debug!("Tx cache updates exited"),
        //         Err(e) => warn!("Tx cache updates exited with error: {}", e),
        //     }
        // },
        res = tx_store_handle => {
            match res {
                Ok(_) => debug!("Tx cache updates exited"),
                Err(e) => warn!("Tx cache updates exited with error: {}", e),
            }
        },
        res = tx_processor_handle => {
            match res {
                Ok(_) => debug!("Tx processor exited"),
                Err(e) => warn!("Tx processor exited with error: {}", e),
            }
        },
        res = tx_pool_handle => {
            match res {
                Ok(_) => debug!("Tx pool monitor exited"),
                Err(e) => warn!("Tx pool monitor exited with error: {}", e),
            }
        },
    }
}
