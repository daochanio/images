use crate::{common::enums::ImageVariants, settings::Settings, usecases::gateways::Storage};
use async_trait::async_trait;
use aws_sdk_s3::{config::Region, error::SdkError, primitives::ByteStream, Client};
use std::sync::Arc;

struct S3 {
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

    S3 { settings, client }
}

#[async_trait]
impl Storage for S3 {
    async fn upload(
        &self,
        file_name: String,
        variant: ImageVariants,
        content_type: String,
        body: Vec<u8>,
    ) -> Result<String, String> {
        let bucket = self.settings.bucket();
        let key = self.get_key(file_name.clone(), variant);
        return match self
            .client
            .put_object()
            .bucket(bucket)
            .key(key.clone())
            .content_type(content_type)
            .cache_control("max-age=31536000") // 1yr
            .body(ByteStream::from(body))
            .send()
            .await
        {
            Ok(_) => Ok(self.get_external_url(key)),
            Err(e) => Err(format!("could not upload image: {}", e)),
        };
    }

    async fn get(
        &self,
        variant: ImageVariants,
        file_name: String,
    ) -> Result<Option<String>, String> {
        let bucket = self.settings.bucket();
        let key = self.get_key(file_name.clone(), variant);
        return match self
            .client
            .head_object()
            .bucket(bucket)
            .key(key.clone())
            .send()
            .await
        {
            Ok(_) => Ok(Some(self.get_external_url(key))),
            Err(e) => match e {
                SdkError::ServiceError(err) => {
                    if err.err().is_not_found() {
                        Ok(None)
                    } else {
                        Err(format!("could not check if image exists: {}", err.err()))
                    }
                }
                _ => Err(format!("could not check if image exists: {}", e)),
            },
        };
    }
}

impl S3 {
    fn get_key(&self, id: String, variant: ImageVariants) -> String {
        return match variant {
            ImageVariants::Thumbnail => format!("images/thumbnails/{}", id),
            ImageVariants::Original => format!("images/originals/{}", id),
            ImageVariants::Avatar => format!("images/avatars/{}", id),
        };
    }

    fn get_external_url(&self, key: String) -> String {
        return format!("{}/{}", self.settings.storage_external_url(), key);
    }
}
