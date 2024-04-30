use std::ops::Range;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::ServerAPI;

use super::config::ServerAPIConfig;
use super::rate_limiting::RateLimiter;

use super::worker::{Worker, WorkerMessage};

pub struct WorkersPool<'a, S: ServerAPI + Send + Sync + 'static> {
    master: mpsc::Sender<WorkerMessage>,
    master_messages: mpsc::Receiver<WorkerMessage>,
    workers: Option<Box<dyn Iterator<Item = Worker<S>> + 'a>>,
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
            master,
            master_messages,
            workers: Some(Box::new(workers)),
            cancel_tokens: Vec::new(),
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

                    tokio::spawn(worker.start(block_height_range));

                    Some(cancel_token)
                } else {
                    None
                }
            })
            .collect();
    }

    pub fn has_shutdown(&self) -> bool {
        self.workers.is_none()
    }

    /// Shutdown workers' pool idempotently
    pub async fn shutdown(&mut self) {
        if !self.has_shutdown() {
            info!("WorkerPool shutting down...");
            self.cancel_tokens.iter_mut().for_each(|c| c.cancel());
            self.workers = None;
            sleep(Duration::from_millis(100)).await;
            self.master
                .send(WorkerMessage::AllDone)
                .await
                .expect("Master did the shutdown afterall!");
        }
    }
}
