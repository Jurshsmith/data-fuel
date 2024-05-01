mod block_height_ranges;
pub mod config;
pub mod rate_limiting;
pub mod results;
mod sync_engine;
mod worker;
mod workers_pool;

pub use sync_engine::*;
use tracing::info;

use std::{ops::Range, time::Instant};

/// Fields required by the consensus to validate the block.
///
/// For simplicity, it is a dummy structure. We don't need to verify the block header validity.
#[derive(Clone, Debug, Default)]
pub struct ConsensusFields;

/// Fields of the transaction that cause some state transition of the blockchain.
///
/// For simplicity, it is a dummy structure. We don't need to implement state transition.
#[derive(Clone, Debug, Default)]
pub struct TransactionFields;

/// The identifier of the transaction in the database and the network.
pub type TransactionId = [u8; 32];

/// The header of the block that describes the final state of the blockchain at `block_height`.
#[derive(Clone, Debug, Default)]
pub struct BlockHeader {
    pub block_height: u32,
    pub consensus_fields: ConsensusFields,
}

impl BlockHeader {
    /// The function that verifies the block header validity.
    pub fn verify(&self) -> bool {
        true
    }
}

/// The error that describe failed state transition.
pub struct StateTransitionError;

/// The transaction causes a state transition on the blockchain.
#[derive(Clone, Debug, Default)]
pub struct Transaction {
    pub tx_id: TransactionId,
    pub transaction_fields: TransactionFields,
}

impl Transaction {
    /// The function executes transaction and performance state transition.
    ///
    /// Note: This can be a very heavy operation and may take up to `1` second to execute.
    pub fn execute(self) -> Result<(), StateTransitionError> {
        simulate_sync_work();
        Ok(())
    }
}

fn simulate_sync_work() {
    let now = Instant::now();
    let mut clock_seconds = 0;
    loop {
        clock_seconds += 1;
        if clock_seconds > 100 {
            break;
        }
    }
    let elapsed = now.elapsed();
    info!("Transaction execution finished in: {:.2?}", elapsed);
}

/// The block that contains transactions and the header of the blockchain state.
#[derive(Clone, Debug)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}
/// The error that describes failure on the server side.
#[derive(Clone, Debug)]
pub struct ServerError;

/// The API is supported by the server.
#[async_trait::async_trait]
pub trait ServerAPI {
    /// Return the list of headers for provided `block_height_range`.
    ///
    /// The endpoint guarantees that the ordering of the header is right and headers are connected to each other.
    /// The maximum length of the range is `10000`; otherwise, the request will be rejected.
    /// The service has a limit of `100` requests per second.
    async fn block_headers(
        &self,
        block_height_range: Range<u32>,
    ) -> Result<Vec<BlockHeader>, ServerError>;

    /// Return the list of transactions per each block height from `block_height_range`.
    ///
    /// Each element is a `Vec<Transaction>` that belongs to the corresponding block height.
    /// The endpoint guarantees that the ordering of transactions is right according to the `block_height_range` .
    /// The maximum length of the range is `10000`; otherwise, the request will be rejected.
    /// The service has a limit of `100` requests per second.
    async fn block_transactions(
        &self,
        block_height_range: Range<u32>,
    ) -> Result<Vec<Vec<Transaction>>, ServerError>;
}
