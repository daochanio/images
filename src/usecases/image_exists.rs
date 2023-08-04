use std::sync::Arc;

use super::gateways::{ImageVariants, Storage};

pub struct ImageExists {
    storage: Arc<dyn Storage>,
}

pub fn new(storage: Arc<dyn Storage>) -> ImageExists {
    ImageExists { storage }
}

impl ImageExists {
    // TODO:
    // - support checking existence of other variants in the future?
    pub async fn execute(&self, file_name: String) -> Result<bool, String> {
        match self
            .storage
            .exists(file_name, ImageVariants::Original)
            .await
        {
            Ok(exists) => Ok(exists),
            Err(e) => Err(format!("could not check if image exists: {}", e)),
        }
    }
}
