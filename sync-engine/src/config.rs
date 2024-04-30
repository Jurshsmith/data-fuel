use std::{ops::Range, sync::Arc, time::Duration};

use super::rate_limiting::RateLimit;

use crate::ServerAPI;

#[derive(Clone, Debug)]
pub struct Config<S: ServerAPI + Send + Sync + 'static> {
    pub total_block_height_range: Range<u32>,
    pub chunk_size: u32,
    pub server_api_config: ServerAPIConfig<S>,
}

#[derive(Clone, Debug)]
pub struct ServerAPIConfig<S: ServerAPI + Send + Sync + 'static> {
    pub server_api: Arc<S>,
    pub block_headers_rate_limit: RateLimit,
    pub block_transactions_rate_limit: RateLimit,
}

impl<S: ServerAPI + Send + Sync + 'static> Config<S> {
    pub fn new(server_api: S) -> Self {
        Self {
            total_block_height_range: 0..10_000_000,
            chunk_size: 10_000,
            server_api_config: ServerAPIConfig {
                server_api: Arc::new(server_api),
                block_headers_rate_limit: RateLimit::new(100, Duration::from_secs(1)),
                block_transactions_rate_limit: RateLimit::new(100, Duration::from_secs(1)),
            },
        }
    }

    pub fn with_total_block_height_range(mut self, total_block_height_range: Range<u32>) -> Self {
        self.total_block_height_range = total_block_height_range;
        self
    }
    pub fn with_chunk_size(mut self, chunk_size: u32) -> Self {
        self.chunk_size = chunk_size;
        self
    }
    pub fn with_block_headers_rate_limit(mut self, rate_limit: RateLimit) -> Self {
        self.server_api_config.block_headers_rate_limit = rate_limit;
        self
    }
    pub fn with_block_transactions_rate_limit(mut self, rate_limit: RateLimit) -> Self {
        self.server_api_config.block_transactions_rate_limit = rate_limit;
        self
    }
}
