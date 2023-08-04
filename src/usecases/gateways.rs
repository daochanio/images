use async_trait::async_trait;

#[async_trait]
pub trait Storage: Send + Sync {
    async fn upload(
        &self,
        file_name: String,
        variant: ImageVariants,
        content_type: String,
        body: Vec<u8>,
    ) -> Result<(), String>;
    async fn exists(&self, file_name: String, variant: ImageVariants) -> Result<bool, String>;
}

#[derive(Debug, Clone)]
pub enum ImageVariants {
    Thumbnail,
    Original,
    Avatar,
}

#[async_trait]
pub trait Images: Send + Sync {
    async fn resize(
        &self,
        data: &[u8],
        variant: ImageVariants,
    ) -> Result<(Vec<u8>, String), String>;
}

#[async_trait]
pub trait Web: Send + Sync {
    async fn get_nft_image_url(&self, url: String) -> Result<String, String>;
    async fn get_image_data(&self, url: String) -> Result<Vec<u8>, String>;
}
