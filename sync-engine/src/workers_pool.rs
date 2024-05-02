use std::ops::Range;

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::info;

use super::config::ServerAPIConfig;
use super::rate_limiting::RateLimiter;
use super::worker::{Worker, WorkerMessage};
use crate::ServerAPI;

type WorkerId = u32;
pub struct WorkersPool<'a, S: ServerAPI + Send + Sync + 'static> {
    size: u32,
    master_messages: mpsc::Receiver<WorkerMessage>,
    workers: Option<Box<dyn Iterator<Item = Worker<S>> + 'a>>,
    workers_tracker: TaskTracker,
    pub(crate) shutdown_workers: Vec<WorkerId>,
    cancel_tokens: Vec<CancellationToken>,
}

impl<'a, S: ServerAPI + Send + Sync + 'static> WorkersPool<'a, S> {
    pub fn new(
        ServerAPIConfig {
            server_api,
            block_headers_rate_limit,
            block_transactions_rate_limit,
        }: &'a ServerAPIConfig<S>,
        size: u32,
    ) -> Self {
        let (master, master_messages) = mpsc::channel::<WorkerMessage>(size as usize);

        let workers = (1..=size).map({
            let master = master.clone();

            move |id| {
                Worker::new(
                    id,
                    server_api.clone(),
                    master.clone(),
                    RateLimiter::new(
                        block_headers_rate_limit
                            .get_min_elapsed(size)
                            .expect("Size exceeds rate limit per worker"),
                    ),
                    RateLimiter::new(
                        block_transactions_rate_limit
                            .get_min_elapsed(size)
                            .expect("Size exceeds rate limit per worker"),
                    ),
                    CancellationToken::new(),
                )
            }
        });

        Self {
            size,
            master_messages,
            workers: Some(Box::new(workers)),
            cancel_tokens: Vec::new(),
            workers_tracker: TaskTracker::new(),
            shutdown_workers: Vec::new(),
        }
    }

    pub async fn next_worker_message(&mut self) -> Option<WorkerMessage> {
        self.master_messages.recv().await
    }

    pub fn start_workers(&mut self, block_height_ranges: &mut impl Iterator<Item = Range<u32>>) {
        self.cancel_tokens = self
            .workers
            .as_mut()
            .expect("Workers cannot be restarted. Create a new pool!")
            .map_while(|worker| {
                if let Some(block_height_range) = block_height_ranges.take(1).next() {
                    let cancel_token = worker.cancel_token.clone();

                    self.workers_tracker.spawn(worker.start(block_height_range));

                    Some(cancel_token)
                } else {
                    None
                }
            })
            .collect();

        self.workers_tracker.close();
    }

    pub fn has_started_shutdown(&self) -> bool {
        self.workers.is_none()
    }

    pub fn all_workers_have_shutdown(&self) -> bool {
        self.shutdown_workers.len() as u32 == self.size
    }

    /// Starts shutdown for workers' pool idempotently
    pub fn start_shutdown(&mut self) {
        if !self.has_started_shutdown() {
            info!("WorkerPool shutdown started...");
            self.cancel_tokens.iter_mut().for_each(|c| c.cancel());
            self.workers = None;
        }
    }
}
