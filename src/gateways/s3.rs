use crate::{settings::Settings, usecases::gateways::Storage};
use async_trait::async_trait;
use aws_sdk_s3::{config::Region, primitives::ByteStream, Client};
use std::sync::Arc;

struct S3Impl {
    settings: Arc<Settings>,
    client: Client,
}

pub async fn new(settings: Arc<Settings>) -> impl Storage {
    let config = aws_config::from_env()
        .region(Region::new(settings.region()))
        .endpoint_url(settings.endpoint())
        .load()
        .await;
    let client = aws_sdk_s3::Client::new(&config);

    S3Impl { settings, client }
}

#[async_trait]
impl Storage for S3Impl {
    async fn upload(&self, key: String, content_type: String, body: Vec<u8>) -> Result<(), String> {
        return match self
            .client
            .put_object()
            .bucket(self.settings.bucket())
            .key(key)
            .content_type(content_type)
            .body(ByteStream::from(body))
            .send()
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("could not upload image: {}", e)),
        };
    }
}
