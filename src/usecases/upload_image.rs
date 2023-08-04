use std::sync::Arc;

use uuid::Uuid;

use super::gateways::{ImageVariants, Images, Storage};

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
    pub async fn execute(&self, data: &[u8]) -> Result<String, String> {
        let file_name = Uuid::new_v4();

        let (thumbnail_result, original_result) = tokio::join!(
            self.execute_internal(file_name.to_string(), data, ImageVariants::Thumbnail),
            self.execute_internal(file_name.to_string(), data, ImageVariants::Original)
        );

        if let Err(e) = thumbnail_result {
            return Err(format!("could not put thumbnail: {}", e));
        }

        if let Err(e) = original_result {
            return Err(format!("could not put original: {}", e));
        }

        return Ok(file_name.to_string());
    }

    async fn execute_internal(
        &self,
        file_name: String,
        data: &[u8],
        variant: ImageVariants,
    ) -> Result<(), String> {
        let (image, format) = match self.images.resize(data, variant.clone()).await {
            Ok((image, format)) => (image, format),
            Err(e) => return Err(format!("could not format image: {}", e)),
        };

        let content_type = format!("image/{format}").to_lowercase();

        return self
            .storage
            .upload(file_name, variant, content_type, image)
            .await;
    }
}
