use std::sync::Arc;

use anyhow::{anyhow, Result};

use super::gateways::{Images, Storage, Video};
use crate::{
    common::{format::Format, variant::Variant},
    entities::image::{self, Image},
};

pub struct UploadImage {
    storage: Arc<dyn Storage>,
    images: Arc<dyn Images>,
    video: Arc<dyn Video>,
}

pub fn new(
    storage: Arc<dyn Storage>,
    images: Arc<dyn Images>,
    video: Arc<dyn Video>,
) -> UploadImage {
    UploadImage {
        storage,
        images,
        video,
    }
}

impl UploadImage {
    pub async fn execute(&self, file_name: String, data: &[u8], variant: Variant) -> Result<Image> {
        let input_format =
            Format::infer(data).map_err(|e| anyhow!("could not infer format: {}", e))?;

        let result = match input_format {
            Format::Jpeg | Format::Png | Format::WebP => {
                self.images
                    .format(data, variant.clone(), input_format.clone())
                    .await
            }
            Format::Gif | Format::Mp4 => {
                self.video
                    .format(data, variant.clone(), input_format.clone())
                    .await
            }
        };

        let (thumbnail, output_format) =
            result.map_err(|e| anyhow!("could not resize image: {}", e))?;

        let (original_result, thumbnail_result) = tokio::join!(
            self.storage.upload(
                file_name.to_string(),
                Variant::Original,
                input_format.content_type(),
                data.to_vec(),
            ),
            self.storage.upload(
                file_name.to_string(),
                variant,
                output_format.content_type(),
                thumbnail
            ),
        );

        match (original_result, thumbnail_result) {
            (Ok(original_url), Ok(thumbnail_url)) => Ok(image::new(
                file_name.to_string(),
                original_url,
                input_format.content_type(),
                thumbnail_url,
                output_format.content_type(),
            )),
            (Err(e), _) => Err(anyhow!("could not upload original: {}", e)),
            (_, Err(e)) => Err(anyhow!("could not upload thumbnail: {}", e)),
        }
    }
}
