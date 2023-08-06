use async_trait::async_trait;

use crate::common::enums::ImageVariants;

#[async_trait]
pub trait Storage: Send + Sync {
    async fn upload(
        &self,
        file_name: String,
        variant: ImageVariants,
        content_type: String,
        body: Vec<u8>,
    ) -> Result<String, String>;
    async fn get(
        &self,
        variant: ImageVariants,
        file_name: String,
    ) -> Result<Option<String>, String>;
}

#[async_trait]
pub trait Images: Send + Sync {
    async fn resize(
        &self,
        data: &[u8],
        variant: ImageVariants,
    ) -> Result<(Vec<u8>, String), String>;
    fn get_content_type(&self, data: &[u8]) -> Result<String, String>;
}

#[async_trait]
pub trait Web: Send + Sync {
    async fn get_nft_image_url(&self, url: String) -> Result<String, String>;
    async fn get_image_data(&self, url: String) -> Result<Vec<u8>, String>;
}
