use hex;
use sha2::{Digest, Sha256};
use std::sync::Arc;

use super::gateways::{ImageVariants, Images, Storage, Web};

pub struct UploadAvatar {
    storage: Arc<dyn Storage>,
    images: Arc<dyn Images>,
    web: Arc<dyn Web>,
}

pub fn new(storage: Arc<dyn Storage>, images: Arc<dyn Images>, web: Arc<dyn Web>) -> UploadAvatar {
    UploadAvatar {
        storage,
        images,
        web,
    }
}

// Logic:
// - hash the uri to get filename
// - check if filename hash already exists in storage
//  - if exists, return hashed filename without uploading
// - if nft, get image uri from metadata
// - download image and upload to bucket
impl UploadAvatar {
    pub async fn execute(&self, url: String, is_nft: bool) -> Result<String, String> {
        let file_name = self.hash(url.clone());

        match self
            .storage
            .exists(file_name.clone(), ImageVariants::Avatar)
            .await
        {
            Ok(exists) => {
                if exists {
                    return Ok(file_name);
                }
            }
            Err(e) => return Err(format!("could not check if image exists: {}", e)),
        };

        let image_url = match is_nft {
            true => match self.web.get_nft_image_url(url).await {
                Ok(nft_image_uri) => nft_image_uri,
                Err(e) => return Err(format!("could not get nft uri: {}", e)),
            },
            false => url,
        };

        let image_data = match self.web.get_image_data(image_url).await {
            Ok(image_data) => image_data,
            Err(e) => return Err(format!("could not get image data: {}", e)),
        };

        let (image, content_type) = match self
            .images
            .resize(image_data.as_ref(), ImageVariants::Avatar)
            .await
        {
            Ok((thumbnail, content_type)) => (thumbnail, content_type),
            Err(e) => return Err(format!("could not resize image: {}", e)),
        };

        return match self
            .storage
            .upload(
                file_name.clone(),
                ImageVariants::Avatar,
                content_type,
                image,
            )
            .await
        {
            Ok(_) => Ok(file_name),
            Err(e) => Err(format!("could not upload image: {}", e)),
        };
    }

    fn hash(&self, input: String) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input);
        let result = hasher.finalize();
        hex::encode(result)
    }
}
