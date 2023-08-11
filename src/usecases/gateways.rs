use async_trait::async_trait;

use crate::common::{format::Format, variant::Variant};

#[async_trait]
pub trait Storage: Send + Sync {
    async fn upload(
        &self,
        file_name: String,
        variant: Variant,
        content_type: String,
        body: Vec<u8>,
    ) -> Result<String, String>;
    async fn get(
        &self,
        variant: Variant,
        file_name: String,
    ) -> Result<Option<(String, String)>, String>;
}

#[async_trait]
pub trait Images: Send + Sync {
    async fn format(
        &self,
        data: &[u8],
        variant: Variant,
        input_format: Format,
    ) -> Result<(Vec<u8>, Format), String>;
}

#[async_trait]
pub trait Web: Send + Sync {
    async fn get_nft_image_url(&self, url: String) -> Result<String, String>;
    async fn get_image_data(&self, url: String) -> Result<Vec<u8>, String>;
}

#[async_trait]
pub trait Video: Send + Sync {
    async fn format(
        &self,
        data: &[u8],
        variant: Variant,
        input_format: Format,
    ) -> Result<(Vec<u8>, Format), String>;
    async fn clean(&self, stale_seconds: u64) -> Result<(), String>;
}
