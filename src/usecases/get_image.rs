use std::sync::Arc;

use anyhow::{anyhow, Result};

use crate::{
    common::variant::Variant,
    entities::image::{self, Image},
};

use super::gateways::Storage;

pub struct GetImage {
    storage: Arc<dyn Storage>,
}

pub fn new(storage: Arc<dyn Storage>) -> GetImage {
    GetImage { storage }
}

impl GetImage {
    pub async fn execute(&self, file_name: String, variant: Variant) -> Result<Option<Image>> {
        let (original_result, thumbnail_result) = tokio::join!(
            self.storage.get(Variant::Original, file_name.to_string()),
            self.storage.get(variant, file_name.to_string()),
        );

        match (original_result, thumbnail_result) {
            (
                Ok(Some((original_url, original_content_type))),
                Ok(Some((thumbnail_url, thumbnail_content_type))),
            ) => Ok(Some(image::new(
                file_name,
                original_url,
                original_content_type,
                thumbnail_url,
                thumbnail_content_type,
            ))),
            (Ok(Some(_)), Ok(None)) => {
                tracing::warn!("original exists but thumbnail does not");
                Ok(None)
            }
            (Ok(None), Ok(Some(_))) => {
                tracing::warn!("thumbnail exists but original does not");
                Ok(None)
            }
            (Ok(None), Ok(None)) => {
                tracing::warn!("neither original nor thumbnail exists");
                Ok(None)
            }
            (Err(e), _) => Err(anyhow!("could not check if original exists: {}", e)),
            (_, Err(e)) => Err(anyhow!("could not check if thumbnail exists: {}", e)),
        }
    }
}
