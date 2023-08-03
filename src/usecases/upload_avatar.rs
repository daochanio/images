use std::sync::Arc;

use super::gateways::{Images, Storage};

pub struct UploadAvatar {
    storage: Arc<dyn Storage>,
    images: Arc<dyn Images>,
}

pub fn new(storage: Arc<dyn Storage>, images: Arc<dyn Images>) -> UploadAvatar {
    UploadAvatar { storage, images }
}

impl UploadAvatar {
    pub async fn execute(&self) -> Result<(), String> {
        Ok(())
    }
}
