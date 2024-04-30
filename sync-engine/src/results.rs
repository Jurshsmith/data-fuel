use std::ops::Range;

use crate::{Block, ServerError};

#[derive(Clone, Debug)]
pub enum WorkerSkippedError {
    StateTransitionError(u32),
    UnverifiedBlockHeight(u32),
}

#[derive(Clone, Debug)]
pub enum WorkerError {
    BlockHeaders(Range<u32>, ServerError),
    BlockTransactions(Range<u32>, ServerError),
    FinishedWithSkippedErrors(Range<u32>, Vec<WorkerSkippedError>),
}

pub type WorkerResult = Result<Vec<Block>, WorkerError>;

pub fn to_worker_result(
    results: Vec<Result<Block, WorkerSkippedError>>,
    batch: Range<u32>,
) -> Result<Vec<Block>, WorkerError> {
    let mut created_blocks = Vec::new();
    let mut skipped_errors = Vec::new();

    for result in results {
        match result {
            Ok(created_block) => created_blocks.push(created_block),
            Err(skipped_error) => skipped_errors.push(skipped_error),
        }
    }

    if skipped_errors.is_empty() {
        Ok(created_blocks)
    } else {
        let error = WorkerError::FinishedWithSkippedErrors(batch, skipped_errors);
        Err(error)
    }
}
