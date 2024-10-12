use anyhow::Result;
use ethers::types::{Block, Transaction, H256};
use scylla::{Session, SessionBuilder};

//pub mod engine;

pub async fn create_database(session: &Session) -> Result<()> {
    // Create eth keyspace
    session
        .query(
            "CREATE KEYSPACE IF NOT EXISTS eth WITH REPLICATION = { 'class' : 'SimpleStrategy', 'replication_factor' : 1 };",
            ()
        )
        .await?;

    Ok(())
}

pub async fn init_blocks_session() -> Result<Session> {
    // Get the SCYLLA_URI from the environment
    let scylla_uri = std::env::var("SCYLLA_URI")
        .expect("Please set the `SCYLLA_URI` environment variable to point to a Scylla server");
    let session: Session = SessionBuilder::new().known_node(scylla_uri).build().await?;

    // Create eth keyspace
    create_database(&session).await?;

    // Create blocks table based on ethers-rs Block type
    session
        .query(
            "CREATE TABLE IF NOT EXISTS eth.blocks (
                number bigint,
                hash text,
                parent_hash text,
                nonce text,
                sha3_uncles text,
                logs_bloom text,
                transactions_root text,
                state_root text,
                receipts_root text,
                miner text,
                difficulty text,
                total_difficulty text,
                size bigint,
                extra_data text,
                gas_limit bigint,
                gas_used bigint,
                timestamp bigint,
                transactions list<text>,
                uncles list<text>,
                PRIMARY KEY (number)
            );",
            (),
        )
        .await?;

    Ok(session)
}

pub async fn store_block(session: &Session, block: Block<H256>) -> Result<()> {
    // Store the block in the database using the ethers-rs Block type
    session
        .query(
            "INSERT INTO eth.blocks (
                number,
                hash,
                parent_hash,
                nonce,
                sha3_uncles,
                logs_bloom,
                miner,
                difficulty,
                total_difficulty,
                size,
                extra_data,
                gas_limit,
                gas_used,
                timestamp,
                transactions,
                uncles
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            (
                block.number.unwrap_or_default().as_usize() as i64,
                block.hash.unwrap().to_string(),
                block.parent_hash.to_string(),
                block.nonce.unwrap().to_string(),
                block.uncles_hash.to_string(),
                block.logs_bloom.unwrap().to_string(),
                block.author.unwrap_or_default().to_string(),
                block.difficulty.to_string(),
                block.total_difficulty.unwrap_or_default().to_string(),
                block.size.unwrap_or_default().as_usize() as i64,
                block.extra_data.to_string(),
                block.gas_limit.as_usize() as i64,
                block.gas_used.as_usize() as i64,
                block.timestamp.as_usize() as i64,
                block
                    .transactions
                    .iter()
                    .map(|tx| tx.to_string())
                    .collect::<Vec<String>>(),
                block
                    .uncles
                    .iter()
                    .map(|uncle| uncle.to_string())
                    .collect::<Vec<String>>(),
            ),
        )
        .await?;

    Ok(())
}

pub async fn init_tx_session() -> Result<Session> {
    // Get the SCYLLA_URI from the environment
    let scylla_uri = std::env::var("SCYLLA_URI")
        .expect("Please set the `SCYLLA_URI` environment variable to point to a Scylla server");
    let session: Session = SessionBuilder::new().known_node(scylla_uri).build().await?;

    // Create eth keyspace
    create_database(&session).await?;

    // Create transactions table
    session
        .query(
            "CREATE TABLE IF NOT EXISTS eth.transactions (
                hash text,
                nonce bigint,
                block_hash text,
                block_number bigint,
                transaction_index bigint,
                from_address text,
                to_address text,
                value text,
                gas_price text,
                gas bigint,
                input text,
                PRIMARY KEY (hash)
            );",
            (),
        )
        .await?;

    Ok(session)
}

pub async fn store_tx(session: &Session, tx: Transaction) -> Result<()> {
    // Store the transaction in the database
    session
        .query(
            "INSERT INTO eth.transactions (
                hash,
                nonce,
                block_hash,
                block_number,
                transaction_index,
                from_address,
                to_address,
                value,
                gas_price,
                gas,
                input
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            (
                tx.hash.to_string(),
                tx.nonce.as_usize() as i64,
                tx.block_hash.unwrap_or_default().to_string(),
                tx.block_number.unwrap_or_default().as_usize() as i64,
                tx.transaction_index.unwrap_or_default().as_usize() as i64,
                tx.from.to_string(),
                tx.to.unwrap_or_default().to_string(),
                tx.value.to_string(),
                tx.gas_price.unwrap_or_default().to_string(),
                tx.gas.as_usize() as i64,
                tx.input.to_string(),
            ),
        )
        .await?;

    Ok(())
}
