use controllers::http;
use std::sync::Arc;

mod common;
mod container;
mod controllers;
mod entities;
mod gateways;
mod settings;
mod usecases;

#[tokio::main]
async fn main() {
    let container = Arc::new(container::new().await);

    let cloned_container = container.clone();
    tokio::spawn(async move {
        controllers::cleaner::start(cloned_container).await;
    });

    http::start(container.clone()).await;
}
