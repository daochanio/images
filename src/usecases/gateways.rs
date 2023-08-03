use async_trait::async_trait;

#[async_trait]
pub trait Storage: Send + Sync {
    async fn upload(&self, key: String, content_type: String, body: Vec<u8>) -> Result<(), String>;
}

#[async_trait]
pub trait Images: Send + Sync {
    async fn resize(
        &self,
        data: &[u8],
        variant: ImageVariants,
    ) -> Result<(Vec<u8>, String), String>;
}

#[derive(Debug, Clone)]
pub enum ImageVariants {
    Thumbnail,
    Original,
}
