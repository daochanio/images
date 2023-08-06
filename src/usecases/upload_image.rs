use std::sync::Arc;

use uuid::Uuid;

use super::gateways::{Images, Storage};
use crate::{
    common::enums::ImageVariants,
    entities::image::{self, Image},
};

pub struct UploadImage {
    storage: Arc<dyn Storage>,
    images: Arc<dyn Images>,
}

pub fn new(storage: Arc<dyn Storage>, images: Arc<dyn Images>) -> UploadImage {
    UploadImage { storage, images }
}

impl UploadImage {
    pub async fn execute(&self, data: &[u8]) -> Result<Image, String> {
        let file_name = Uuid::new_v4();

        let original_content_type = match self.images.get_content_type(data) {
            Ok(content_type) => content_type,
            Err(e) => return Err(format!("could not get content type: {}", e)),
        };

        let (thumbnail, thumbnail_content_type) =
            match self.images.resize(data, ImageVariants::Thumbnail).await {
                Ok((image, content_type)) => (image, content_type),
                Err(e) => return Err(format!("could not resize image: {}", e)),
            };

        let (original_result, thumbnail_result) = tokio::join!(
            self.storage.upload(
                file_name.to_string(),
                ImageVariants::Original,
                original_content_type,
                data.to_vec(),
            ),
            self.storage.upload(
                file_name.to_string(),
                ImageVariants::Thumbnail,
                thumbnail_content_type,
                thumbnail
            ),
        );

        match (original_result, thumbnail_result) {
            (Ok(original_url), Ok(thumbnail_url)) => Ok(image::new(
                file_name.to_string(),
                original_url,
                thumbnail_url,
            )),
            (Err(e), _) => Err(format!("could not upload original: {}", e)),
            (_, Err(e)) => Err(format!("could not upload thumbnail: {}", e)),
        }
    }
}
