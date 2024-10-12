use ethers::types::{NameOrAddress, TransactionRequest};

pub mod block_processor;
pub mod block_watcher;
pub mod tx_pool;
pub mod tx_processor;

pub fn check_contract_creation(
    to: ethers::types::H160,
) -> Result<Option<TransactionRequest>, anyhow::Error> {
    let opt_to = Some(to);
    let tx_request = TransactionRequest {
        to: opt_to.map(NameOrAddress::from),
        ..Default::default()
    };
    Ok(if tx_request.to.is_none() {
        None
    } else {
        Some(tx_request)
    })
}
