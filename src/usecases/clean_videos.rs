use std::sync::Arc;

use anyhow::{Context, Result};

use super::gateways::Video;

pub struct CleanVideos {
    video: Arc<dyn Video>,
}

pub fn new(video: Arc<dyn Video>) -> CleanVideos {
    CleanVideos { video }
}

const STALE_SECONDS: u64 = 2 * 60; // 2 mins

impl CleanVideos {
    pub async fn execute(&self) -> Result<()> {
        tracing::info!("cleaning videos");

        self.video
            .clean(STALE_SECONDS)
            .await
            .context("could not clean videos")?;

        tokio::time::sleep(tokio::time::Duration::from_secs(STALE_SECONDS)).await;

        Ok(())
    }
}
