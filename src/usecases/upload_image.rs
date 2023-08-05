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

// TODO:
// We can run the two tasks in parallel threads but we need to consider the implications of spawning threads from requests.
// We would likely need to use some kind of thread pool to ensure we don't exhaust the system resources under high request load.
// Naive tests with threads definitely shows a significant improvement in performance.
impl UploadImage {
    pub async fn execute(&self, data: &[u8]) -> Result<Image, String> {
        let file_name = Uuid::new_v4();

        let (original_result, thumbnail_result) = tokio::join!(
            self.execute_internal(file_name.to_string(), data, ImageVariants::Original),
            self.execute_internal(file_name.to_string(), data, ImageVariants::Thumbnail),
        );

        match (original_result, thumbnail_result) {
            (Ok(original_url), Ok(thumbnail_url)) => Ok(image::new(
                file_name.to_string(),
                original_url,
                thumbnail_url,
            )),
            (Err(e), _) => Err(format!("could not put original: {}", e)),
            (_, Err(e)) => Err(format!("could not put thumbnail: {}", e)),
        }
    }

    async fn execute_internal(
        &self,
        file_name: String,
        data: &[u8],
        variant: ImageVariants,
    ) -> Result<String, String> {
        let (image, content_type) = match self.images.resize(data, variant.clone()).await {
            Ok((image, content_type)) => (image, content_type),
            Err(e) => return Err(format!("could not format image: {}", e)),
        };

        self.storage
            .upload(file_name, variant, content_type, image)
            .await
    }
}
