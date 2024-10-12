use anyhow::Result;
use ethers::types::Transaction;
use log::debug;
use scylla::{Session, SessionBuilder};

use crate::tx_storage::TxStorage;

pub struct TXScyllaStorage {
    pub url: String,
    pub keyspace: String,
    session: Session,
}

impl TXScyllaStorage {
    pub async fn new(
        url: String,
        keyspace: String,
        username: Option<String>,
        password: Option<String>,
    ) -> Result<Self> {
        let session = {
            if let (Some(username), Some(password)) = (username, password) {
                log::info!("Connecting to ScyllaDB with username: {}", username);
                SessionBuilder::new()
                    .known_node(url.as_str())
                    .user(username, password)
                    // .use_keyspace(keyspace.as_str(), true)
                    .build()
                    .await?
            } else {
                log::info!("Connecting to ScyllaDB without username");
                SessionBuilder::new()
                    .known_node(url.as_str())
                    // .use_keyspace(keyspace.as_str(), true)
                    .build()
                    .await?
            }
        };

        Ok(Self {
            url,
            keyspace,
            session,
        })
    }
}

impl TxStorage<Transaction> for TXScyllaStorage {
    async fn store(&mut self, tx: Transaction) -> Result<()> {
        debug!("Storing tx: {:#?}", tx.hash);

        let prepared = self
            .session
            .prepare(
                "INSERT INTO transactions (
                hash,
                nonce,
                blockHash,
                blockNumber,
                transactionIndex,
                from,
                to,
                value,
                gasPrice,
                gas,
                input,
                type,
                maxPriorityFee,
                maxFeePerGas,
                chainId
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .await?;

        self.session
            .execute(
                &prepared,
                (
                    tx.hash.to_string(),
                    tx.nonce.as_u64() as i64,
                    tx.block_hash.unwrap_or_default().to_string(),
                    tx.block_number.unwrap_or_default().as_u64() as i64,
                    tx.transaction_index.unwrap_or_default().as_u64() as i64,
                    tx.from.to_string(),
                    tx.to.unwrap_or_default().to_string(),
                    tx.value.as_u64() as i64,
                    tx.gas_price.unwrap_or_default().as_u64() as i64,
                    tx.gas.as_u64() as i64,
                    tx.input.0.to_vec(),
                    // tx.v,
                    // tx.r,
                    // tx.s,
                    tx.transaction_type.unwrap_or_default().as_u64() as i64,
                    tx.max_priority_fee_per_gas.unwrap_or_default().to_string(),
                    tx.max_fee_per_gas.unwrap_or_default().as_u64() as i64,
                    tx.chain_id.unwrap_or_default().as_u64() as i64,
                ),
            )
            .await?;

        Ok(())
    }

    async fn is_stored(&mut self, tx: Transaction) -> Result<bool> {
        debug!("Checking if tx is stored: {:#?}", tx.hash);

        Ok(false)
    }

    async fn delete(&mut self, tx: Transaction) -> Result<()> {
        debug!("Deleting tx: {:#?}", tx.hash);

        Ok(())
    }
}
