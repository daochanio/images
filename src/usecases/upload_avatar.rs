use hex;
use sha2::{Digest, Sha256};
use std::sync::Arc;

use crate::{common::variant::Variant, entities::image::Image};

use super::{gateways::Web, get_image::GetImage, upload_image::UploadImage};

pub struct UploadAvatar {
    web: Arc<dyn Web>,
    upload_image: Arc<UploadImage>,
    get_image: Arc<GetImage>,
}

pub fn new(
    web: Arc<dyn Web>,
    upload_image: Arc<UploadImage>,
    get_image: Arc<GetImage>,
) -> UploadAvatar {
    UploadAvatar {
        web,
        upload_image,
        get_image,
    }
}

// General idea:
// - hash the uri to get filename
// - check if the avatar already exists
//  - if exists, return existing image
// - if nft, get image url from metadata url
// - download image from url and format/upload
impl UploadAvatar {
    pub async fn execute(&self, url: String, is_nft: bool) -> Result<Image, String> {
        let file_name = self.hash(url.clone());

        match self
            .get_image
            .execute(file_name.clone(), Variant::Avatar)
            .await
        {
            Ok(Some(image)) => {
                tracing::info!("avatar already exists");
                return Ok(image);
            }
            Ok(None) => {
                tracing::info!("avatar does not exist, hydrating...");
            }
            Err(e) => return Err(format!("could not check if avatar exists: {}", e)),
        };

        let image_url = match is_nft {
            true => match self.web.get_nft_image_url(url).await {
                Ok(nft_image_uri) => nft_image_uri,
                Err(e) => return Err(format!("could not get nft uri: {}", e)),
            },
            false => url,
        };

        let data = match self.web.get_image_data(image_url).await {
            Ok(data) => data,
            Err(e) => return Err(format!("could not get image data: {}", e)),
        };

        self.upload_image
            .execute(file_name, &data, Variant::Avatar)
            .await
    }

    fn hash(&self, input: String) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input);
        let result = hasher.finalize();
        hex::encode(result)
    }
}
