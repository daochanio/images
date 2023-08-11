use crate::{common::variant::Variant, settings::Settings, usecases::gateways::Storage};
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
        variant: Variant,
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
        variant: Variant,
        file_name: String,
    ) -> Result<Option<(String, String)>, String> {
        let bucket = self.settings.bucket();
        let key = self.get_key(file_name.clone(), variant);
        let header = match self
            .client
            .head_object()
            .bucket(bucket)
            .key(key.clone())
            .send()
            .await
        {
            Ok(header) => header,
            Err(e) => match e {
                SdkError::ServiceError(err) => {
                    if err.err().is_not_found() {
                        return Ok(None);
                    } else {
                        return Err(format!("could not check if image exists: {}", err.err()));
                    }
                }
                _ => return Err(format!("could not check if image exists: {}", e)),
            },
        };

        let content_type = match header.content_type() {
            Some(content_type) => content_type.to_string(),
            None => return Err(format!("could not get content type for {}", key)),
        };
        let url = self.get_external_url(key);

        Ok(Some((url, content_type)))
    }
}

impl S3 {
    fn get_key(&self, id: String, variant: Variant) -> String {
        return match variant {
            Variant::Thumbnail => format!("images/thumbnails/{}", id),
            Variant::Original => format!("images/originals/{}", id),
            Variant::Avatar => format!("images/avatars/{}", id),
        };
    }

    fn get_external_url(&self, key: String) -> String {
        return format!("{}/{}", self.settings.storage_external_url(), key);
    }
}
