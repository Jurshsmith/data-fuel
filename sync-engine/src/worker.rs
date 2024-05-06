use super::rate_limiting::RateLimiter;
use super::results::{self, WorkerError, WorkerResult, WorkerSkippedError};

use futures::Future;
use rayon::prelude::*;
use std::{collections::HashMap, ops::Range, sync::Arc};
use tokio_util::task::TaskTracker;

use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;

use tracing::{error, info};

use crate::{Block, BlockHeader, Transaction};

use super::ServerAPI;

type Master = mpsc::Sender<WorkerMessage>;

type WorkerId = u32;

#[derive(Debug)]
pub enum WorkerMessage {
    AwaitingWork(WorkerId, oneshot::Sender<Range<u32>>),
    CompletedWork(WorkerId, Range<u32>, WorkerResult),
    FailedWork(
        WorkerId,
        oneshot::Sender<Range<u32>>,
        Range<u32>,
        WorkerResult,
    ),
    FinishedShuttingDown(WorkerId),
}

pub struct Worker<S: ServerAPI + Send + Sync + 'static> {
    id: WorkerId,
    server_api: Arc<S>,
    master: Master,
    block_headers_rate_limiter: RateLimiter,
    block_transactions_rate_limiter: RateLimiter,
    pub cancel_token: CancellationToken,
}

impl<S: ServerAPI + Send + Sync + 'static> Worker<S> {
    pub fn new(
        id: WorkerId,
        server_api: Arc<S>,
        master: Master,
        block_headers_rate_limiter: RateLimiter,
        block_transactions_rate_limiter: RateLimiter,
        cancel_token: CancellationToken,
    ) -> Self {
        Self {
            id,
            server_api,
            master,
            cancel_token,
            block_headers_rate_limiter,
            block_transactions_rate_limiter,
        }
    }

    pub fn start(self, mut block_height_range: Range<u32>) -> impl Future<Output = ()> {
        async move {
            let worker_id = self.id;
            let server_api = self.server_api;
            let master = self.master;
            let cancel_token = self.cancel_token;
            let mut block_headers_rate_limiter = self.block_headers_rate_limiter;
            let mut block_transactions_rate_limiter = self.block_transactions_rate_limiter;

            loop {
                let (worker, worker_messages) = oneshot::channel::<Range<u32>>();
                let (result_agent, result_agent_tracker) = Self::start_result_agent(&master);

                let Range { start, end } = block_height_range;
                info!("Worker:{worker_id} working on BlockHeightRange:{start}..{end}");

                let mut worker_error: Option<WorkerError> = None;

                block_headers_rate_limiter.throttle().await;
                let (mut verified_block_headers, unverified_block_headers) =
                    match server_api.block_headers(start..end).await {
                        Ok(block_headers) => verify_block_headers(block_headers),
                        Err(err) => {
                            worker_error = Some(WorkerError::BlockHeaders(start..end, err));
                            Default::default()
                        }
                    };
                if let Some(worker_error) = worker_error {
                    let message =
                        WorkerMessage::FailedWork(worker_id, worker, start..end, Err(worker_error));
                    return Self::send_result_to_master(result_agent, message);
                }

                block_transactions_rate_limiter.throttle().await;
                let block_transactions = match server_api.block_transactions(start..end).await {
                    Ok(transactions) => transactions,
                    Err(err) => {
                        worker_error = Some(WorkerError::BlockTransactions(start..end, err));
                        Default::default()
                    }
                };
                if let Some(worker_error) = worker_error {
                    let message =
                        WorkerMessage::FailedWork(worker_id, worker, start..end, Err(worker_error));
                    return Self::send_result_to_master(result_agent, message);
                }

                // Delegate computing-intensive task to a rayon's dedicated thread
                rayon::spawn(move || {
                    let created_blocks_results: Vec<_> = execute_block_transactions(
                        block_transactions,
                        start..end,
                        &unverified_block_headers,
                    )
                    .into_iter()
                    .map(|block_txs| maybe_create_block(block_txs, &mut verified_block_headers))
                    .collect();

                    let result_message = WorkerMessage::CompletedWork(
                        worker_id,
                        start..end,
                        results::to_worker_result(created_blocks_results, start..end),
                    );
                    Self::send_result_to_master(result_agent, result_message);
                });

                let message = WorkerMessage::AwaitingWork(worker_id, worker);
                Self::send_master_message(&master, message).await;

                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        info!("Worker:{worker_id} shutdown started...");
                        result_agent_tracker.wait().await;
                        let message = WorkerMessage::FinishedShuttingDown(worker_id);
                        Self::send_master_message(&master, message).await;
                        info!("Worker:{worker_id} finished shutting down");
                        break;
                    }
                    Ok(new_block_height_range) = worker_messages => {
                        block_height_range = new_block_height_range
                    }
                }
            }
        }
    }

    // Result agent stores the result from a worker to re-direct to the master on its behalf
    fn start_result_agent(
        master: &Master,
    ) -> (tokio::sync::oneshot::Sender<WorkerMessage>, TaskTracker) {
        let (result_agent, result_agent_message) = oneshot::channel::<WorkerMessage>();
        let result_agent_tracker = TaskTracker::new();
        result_agent_tracker.spawn({
            let master = master.clone();

            async move {
                let worker_message = result_agent_message
                    .await
                    .expect("Worker's result agent must receive result for master");

                Self::send_master_message(&master, worker_message).await;
            }
        });
        result_agent_tracker.close();

        (result_agent, result_agent_tracker)
    }

    fn send_result_to_master(
        result_agent: oneshot::Sender<WorkerMessage>,
        worker_message: WorkerMessage,
    ) {
        result_agent
            .send(worker_message)
            .expect("Worker's result agent must receive result to redirect to Master")
    }

    async fn send_master_message(master: &Master, worker_message: WorkerMessage) {
        master
            .send(worker_message)
            .await
            .expect("Master must exist as long as I do")
    }
}

fn verify_block_headers(
    block_headers: Vec<BlockHeader>,
) -> (HashMap<u32, BlockHeader>, HashMap<u32, BlockHeader>) {
    block_headers
        .into_iter()
        .map(|block_header| (block_header.block_height, block_header))
        .partition(|(_, block_header)| block_header.verify())
}

fn execute_block_transactions(
    block_transactions: Vec<Vec<Transaction>>,
    Range { start, .. }: Range<u32>,
    unverified_block_headers: &HashMap<u32, BlockHeader>,
) -> Vec<Result<(u32, Vec<Transaction>), WorkerSkippedError>> {
    block_transactions
        // Order matters here to map txs to block_headers correctly
        .into_iter()
        .enumerate()
        .map(move |(current_index, transactions)| {
            let block_height = start + (current_index as u32);

            if unverified_block_headers.contains_key(&block_height) {
                error!("Skipping txs in unverified block:{block_height}");

                Err(WorkerSkippedError::UnverifiedBlockHeight(block_height))
            } else {
                let executed_transactions = transactions
                    .par_iter()
                    .take_any_while(move |tx| match ((*tx).clone()).execute() {
                        Ok(()) => true,
                        Err(_err) => {
                            error!("StateTransitionError: skipping txs for block:{block_height}");

                            false
                        }
                    })
                    .collect::<Vec<_>>();

                if executed_transactions.len() == transactions.len() {
                    Ok((block_height, transactions))
                } else {
                    Err(WorkerSkippedError::StateTransitionError(block_height))
                }
            }
        })
        .collect()
}

fn maybe_create_block(
    block_transactions_results: Result<(u32, Vec<Transaction>), WorkerSkippedError>,
    verified_block_headers: &mut HashMap<u32, BlockHeader>,
) -> Result<Block, WorkerSkippedError> {
    match block_transactions_results {
        Ok((block_height, transactions)) => {
            let new_block = Block {
                header: verified_block_headers
                    .remove(&block_height)
                    .expect("Block transactions cannot skip verified block headers"),
                transactions,
            };

            Ok(new_block)
        }
        Err(skipped_error) => Err(skipped_error),
    }
}
