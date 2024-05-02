use std::cmp::{max, min};
use std::ops::Range;

use crate::block_height_ranges::BlockHeightRanges;
use crate::config::Config;
use crate::results::WorkerResult;
use crate::worker::WorkerMessage;
use crate::workers_pool::WorkersPool;

use tokio::sync::oneshot;
use tracing::info;

use super::ServerAPI;

pub async fn sync_blocks<S: ServerAPI + Send + Sync + 'static>(
    Config {
        total_block_height_range: Range { start, end },
        chunk_size,
        server_api_config,
    }: &Config<S>,
) -> Vec<WorkerResult> {
    let pool_size = min(100, max(1, (end - start) / chunk_size));
    let unsynced_block_heights = &mut BlockHeightRanges::new((*start)..(*end), *chunk_size);

    let mut workers_pool = WorkersPool::new(server_api_config, pool_size);
    let mut results_from_workers = Vec::new();

    info!("Starting workers in WorkerPool of size:{pool_size}");

    workers_pool.start_workers(unsynced_block_heights);

    while let Some(worker_message) = workers_pool.next_worker_message().await {
        match worker_message {
            WorkerMessage::AwaitingWork(_, worker) => {
                maybe_send_new_work(worker, unsynced_block_heights)
            }
            WorkerMessage::CompletedWork(_, _, worker_result) => {
                results_from_workers.push(worker_result);
            }
            WorkerMessage::FailedWork(_, worker, _, worker_result) => {
                results_from_workers.push(worker_result);
                maybe_send_new_work(worker, unsynced_block_heights);
            }
            WorkerMessage::FinishedShuttingDown(worker_id) => {
                workers_pool.shutdown_workers.push(worker_id);

                if workers_pool.all_workers_have_shutdown() {
                    break;
                }
            }
        };

        let sync_is_completed = !unsynced_block_heights.has_next();
        if sync_is_completed {
            workers_pool.start_shutdown();
        }
    }

    results_from_workers
}

fn maybe_send_new_work(
    worker: oneshot::Sender<Range<u32>>,
    unsynced_block_heights: &mut BlockHeightRanges,
) {
    if let Some(new_work) = unsynced_block_heights.next() {
        worker
            .send(new_work)
            .expect("Workers must be available till Pool is shut down");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tracing_test::traced_test;

    use crate::*;

    #[tokio::test]
    #[traced_test]
    async fn does_nothing_when_there_are_no_new_blocks() {
        let server_api = successful_server_api!(vec![], vec![]);

        let config = Config::new(server_api)
            .with_total_block_height_range(0..10)
            .with_chunk_size(10);

        let results = sync_blocks(&config).await;

        assert_eq!(results.len(), 1);
        assert!(results.first().unwrap().as_ref().unwrap().is_empty());
    }

    #[tokio::test]
    #[traced_test]
    async fn syncs_blocks_concurrently_using_chunk_size() {
        let server_api = successful_server_api!(vec![], vec![]);

        let config = Config::new(server_api)
            .with_total_block_height_range(0..10)
            .with_chunk_size(1);

        let results = sync_blocks(&config).await;

        assert_eq!(results.len(), 10);
    }

    #[tokio::test]
    #[traced_test]
    async fn creates_block_in_specified_height_range() {
        let server_api = successful_server_api!(
            vec![BlockHeader {
                block_height: 0,
                ..Default::default()
            }],
            vec![vec![Default::default()]]
        );

        let config = default_config(server_api).with_total_block_height_range(0..1);

        let results = sync_blocks(&config).await;

        assert_eq!(results.len(), 1);
        let result = results.first().unwrap();
        assert!(result.is_ok());
        let created_blocks = results.first().unwrap().clone().unwrap();
        assert_eq!(created_blocks.len(), 1);
        assert_eq!(created_blocks.first().unwrap().header.block_height, 0);
    }

    #[tokio::test]
    #[traced_test]
    async fn creates_multiple_blocks_in_specified_height_range_at_single_chunk_size() {
        let server_api = successful_server_api__headers_matching_range_start__no_block_txs!();

        let config = default_config(server_api)
            .with_total_block_height_range(0..4)
            .with_chunk_size(1);

        let results = sync_blocks(&config).await;

        assert_eq!(results.len(), 4);

        for result in results.into_iter() {
            assert!(result.is_ok());
            let created_blocks = result.unwrap();
            assert_eq!(created_blocks.len(), 1);
            let created_block = created_blocks.first().unwrap();
            assert!((0..4).contains(&created_block.header.block_height));
        }
    }

    fn default_config<S: ServerAPI + Send + Sync + 'static>(server_api: S) -> Config<S> {
        Config::new(server_api)
            .with_total_block_height_range(0..10)
            .with_chunk_size(1)
    }
}

#[macro_export]
macro_rules! successful_server_api {
    ($block_headers:expr, $block_transactions:expr) => {{
        use crate::{BlockHeader, ServerError, Transaction};

        struct TestServerAPI {
            block_headers: Vec<BlockHeader>,
            block_transactions: Vec<Vec<Transaction>>,
        }

        #[async_trait::async_trait]
        impl ServerAPI for TestServerAPI {
            async fn block_headers(&self, _: Range<u32>) -> Result<Vec<BlockHeader>, ServerError> {
                Ok(self.block_headers.clone())
            }

            async fn block_transactions(
                &self,
                _: Range<u32>,
            ) -> Result<Vec<Vec<Transaction>>, ServerError> {
                Ok(self.block_transactions.clone())
            }
        }

        TestServerAPI {
            block_headers: $block_headers,
            block_transactions: $block_transactions,
        }
    }};
}

#[macro_export]
macro_rules! successful_server_api__headers_matching_range_start__no_block_txs {
    () => {{
        use crate::{BlockHeader, ServerError, Transaction};

        #[derive(Clone)]
        struct TestServerAPI;

        #[async_trait::async_trait]
        impl ServerAPI for TestServerAPI {
            async fn block_headers(
                &self,
                Range { start, .. }: Range<u32>,
            ) -> Result<Vec<BlockHeader>, ServerError> {
                Ok(vec![BlockHeader {
                    block_height: start,
                    ..Default::default()
                }])
            }

            async fn block_transactions(
                &self,
                _: Range<u32>,
            ) -> Result<Vec<Vec<Transaction>>, ServerError> {
                Ok(vec![Default::default()])
            }
        }

        TestServerAPI
    }};
}
