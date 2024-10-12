use regex::Regex;

use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::Mutex;

use anyhow::{Error, Result};
use ethers::types::Transaction;
use log::{debug, warn};
use redis::{Commands, ConnectionAddr};
use redis::{ConnectionInfo, RedisConnectionInfo};

use crate::tx_caching::TxCaching;

pub struct TxCacheRedis {
    redis_connection: Arc<Mutex<redis::Client>>,
}

impl TxCacheRedis {
    pub fn new(config: settings::Redis) -> Self {
        let connection = get_connection(&config);

        log::info!("Connecting to Redis at {}", config.url);

        Self {
            redis_connection: Arc::new(Mutex::new(connection)),
        }
    }
}

fn get_connection(config: &settings::Redis) -> redis::Client {
    let re = Regex::new(r"^((?P<scheme>[^:]+)://)?(?P<host>[^:]+):(?P<port>\d+)$").unwrap();
    let caps = re.captures(&config.url).unwrap();
    let scheme = caps.name("scheme").unwrap().as_str();
    let host = caps.name("host").unwrap().as_str().to_string();
    let port = caps.name("port").unwrap().as_str().parse::<u16>().unwrap();

    let connection_info = {
        if scheme.to_lowercase() == "rediss" {
            ConnectionInfo {
                addr: ConnectionAddr::TcpTls {
                    host,
                    port,
                    insecure: config.insecure.unwrap_or(false),
                },
                redis: RedisConnectionInfo {
                    db: config.db,
                    username: config.username.clone(),
                    password: config.password.clone(),
                },
            }
        } else {
            ConnectionInfo {
                addr: ConnectionAddr::Tcp(host, port),
                redis: RedisConnectionInfo {
                    db: config.db,
                    username: config.username.clone(),
                    password: config.password.clone(),
                },
            }
        }
    };

    redis::Client::open(connection_info).expect("Failed to connect to Redis")
}

impl TxCaching for TxCacheRedis {
    async fn cache(&mut self, tx: Transaction) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed to get current time")
            .as_secs();

        let tx_hash = format!("{:#?}", tx.hash).to_string();
        let mut conn = self.redis_connection.lock().await;

        if let Err(e) = conn.set_ex::<String, u64, bool>(tx_hash.clone(), now, 60 * 60 * 24) {
            warn!("Failed to cache tx: {}", e);
            return Err(Error::msg(e.to_string()));
        } else {
            debug!("Cached tx: {}", tx_hash.clone());
        }

        Ok(())
    }

    async fn is_cached(&mut self, tx: Transaction) -> Result<bool> {
        let tx_hash = format!("{:#?}", tx.hash).to_string();
        let mut conn = self.redis_connection.lock().await;

        if let Ok(res) = conn.exists::<String, bool>(tx_hash) {
            return Ok(res);
        }

        Ok(false)
    }

    async fn delete(&mut self, tx: Transaction) -> Result<()> {
        let tx_hash = format!("{:#?}", tx.hash).to_string();
        let mut conn = self.redis_connection.lock().await;

        if let Err(e) = conn.del::<String, bool>(tx_hash) {
            warn!("Failed to delete tx: {}", e);
            return Err(Error::msg(e.to_string()));
        }

        Ok(())
    }
}
