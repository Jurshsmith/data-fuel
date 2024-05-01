use std::{ops::Range, time::Instant};

use sync_engine::config::Config;
use sync_engine::*;

use tracing::info;

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).expect("Tracing setup failed");

    let now = Instant::now();

    let config = Config::new(PlayGroundServerAPI);
    sync_blocks(&config).await;

    let elapsed = now.elapsed();
    info!("Elapsed: {:.2?}", elapsed);
}

struct PlayGroundServerAPI;

#[async_trait::async_trait]
impl ServerAPI for PlayGroundServerAPI {
    async fn block_headers(
        &self,
        block_height_range: Range<u32>,
    ) -> Result<Vec<BlockHeader>, ServerError> {
        Ok(block_height_range
            .map(|height| BlockHeader {
                block_height: height,
                ..Default::default()
            })
            .collect())
    }

    async fn block_transactions(
        &self,
        block_height_range: Range<u32>,
    ) -> Result<Vec<Vec<Transaction>>, ServerError> {
        Ok(block_height_range
            .map(|_| vec![Default::default()])
            .collect())
    }
}
