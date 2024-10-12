use anyhow::Result;
use config::{Config, Environment, File};
use serde::Deserialize;

use dex::router::RouterSettings;

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Ethereum {
    pub node_http: String,
    pub node_ws: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct BlockExplorer {
    pub url: String,
    pub api_key: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Token {
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

// #[derive(Debug, Deserialize, Clone)]
// #[allow(unused)]
// pub struct Router {
//     pub name: String,
//     pub version: u8,
//     pub factory: String,
//     pub addresses: Vec<String>,
// }

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Dex {
    pub tokens: Vec<Token>,
    pub routers: Vec<RouterSettings>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Nats {
    pub url: String,
    pub subject_prefix: String,
    pub queue_group: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Scylla {
    pub url: String,
    pub keyspace: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Redis {
    pub url: String,
    // pub cluster: Option<bool>,
    pub db: i64,
    pub insecure: Option<bool>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Log {
    pub level: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub cache_path: String,
    pub ethereum: Ethereum,
    pub block_explorer: BlockExplorer,
    pub dex: Dex,
    pub nats: Nats,
    pub scylla: Scylla,

    pub redis: Redis,
    pub log: Log,
}

impl Settings {
    pub fn new(config_file_name: String) -> Result<Self> {
        let s = Config::builder()
            .add_source(File::with_name(config_file_name.as_str()))
            .add_source(Environment::with_prefix("SNIPER"))
            .build()?;

        Ok(s.try_deserialize()?)
    }
}
