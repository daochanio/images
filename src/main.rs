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
    let container = container::new().await;

    http::start(Arc::new(container)).await;
}
