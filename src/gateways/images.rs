use std::io::Cursor;

use anyhow::{anyhow, Result};
use async_trait::async_trait;

use crate::{
    common::{format::Format, variant::Variant},
    usecases::gateways::Images,
};

struct ImagesImpl {}

pub fn new() -> impl Images {
    ImagesImpl {}
}

#[async_trait]
impl Images for ImagesImpl {
    // TODO:
    // - add avif support?
    async fn format(
        &self,
        data: &[u8],
        variant: Variant,
        input_format: Format,
    ) -> Result<(Vec<u8>, Format)> {
        let (nwidth, nheight) = match variant {
            Variant::Thumbnail => (300, 300),
            Variant::Avatar => (125, 125),
            Variant::Original => (800, 800),
        };

        let mut image = image::load_from_memory_with_format(data, get_image_format(input_format)?)
            .map_err(|e| anyhow!("could not load image: {}", e))?;

        image = image.resize(nwidth, nheight, image::imageops::FilterType::CatmullRom);

        let mut buffer = Cursor::new(Vec::new());
        image
            .write_to(&mut buffer, image::ImageFormat::WebP)
            .map_err(|e| anyhow!("could not write image: {}", e))?;

        return Ok((buffer.into_inner(), Format::WebP));
    }
}

fn get_image_format(format: Format) -> Result<image::ImageFormat> {
    match format {
        Format::Jpeg => Ok(image::ImageFormat::Jpeg),
        Format::Png => Ok(image::ImageFormat::Png),
        Format::WebP => Ok(image::ImageFormat::WebP),
        _ => return Err(anyhow!("unsupported image format: {:?}", format)),
    }
}
