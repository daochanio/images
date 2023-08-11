use std::sync::Arc;

use tokio::signal::unix::{signal, SignalKind};

use crate::container::Container;

// clean videos and then sleep for 1 minute in a forever loop
pub async fn start(container: Arc<Container>) {
    let mut interrupt_signal =
        signal(SignalKind::interrupt()).expect("Failed to register interrupt signal handler");
    let mut terminate_signal =
        signal(SignalKind::terminate()).expect("Failed to register terminate signal handler");

    loop {
        tokio::select! {
            _ = interrupt_signal.recv() => {
                break;
            },
            _ = terminate_signal.recv() => {
                break;
            },
            result = container.clean_videos.execute() => {
                if let Err(e) = result {
                    tracing::error!("Could not clean videos: {}", e);
                }
            },
        }
    }

    tracing::info!("received shutdown signal, exiting cleaner");
}
