use controllers::http;
use std::sync::Arc;

mod container;
mod controllers;
mod gateways;
mod settings;
mod usecases;

#[tokio::main]
async fn main() {
    let container = container::new().await;

    http::start(Arc::new(container)).await;
}
