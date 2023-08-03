use dotenv::dotenv;
use std::env;
use tracing_subscriber::fmt;

pub struct Settings {
    env: String,
    api_key: String,
    region: String,
    bucket: String,
    endpoint: String,
}

pub fn new() -> Settings {
    dotenv().ok();

    let settings = Settings {
        env: env::var("ENV").unwrap(),
        api_key: env::var("API_KEY").unwrap(),
        region: env::var("REGION").unwrap(),
        bucket: env::var("BUCKET").unwrap(),
        endpoint: env::var("ENDPOINT").unwrap(),
    };

    let subscriber_builder = fmt().with_target(false);

    if settings.is_dev() {
        subscriber_builder
            .compact()
            .with_max_level(tracing::Level::INFO)
            .init();
    } else {
        subscriber_builder
            .json()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    return settings;
}

impl Settings {
    pub fn env(&self) -> String {
        self.env.clone()
    }

    pub fn is_dev(&self) -> bool {
        self.env() == "dev"
    }

    pub fn api_key(&self) -> String {
        self.api_key.clone()
    }

    pub fn region(&self) -> String {
        self.region.clone()
    }

    pub fn bucket(&self) -> String {
        self.bucket.clone()
    }

    pub fn endpoint(&self) -> String {
        self.endpoint.clone()
    }
}
