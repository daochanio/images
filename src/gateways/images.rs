use std::io::Cursor;

use anyhow::{bail, Context, Result};
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

        let image_format = match input_format {
            Format::Jpeg => image::ImageFormat::Jpeg,
            Format::Png => image::ImageFormat::Png,
            Format::WebP => image::ImageFormat::WebP,
            _ => bail!("unsupported image format: {:?}", input_format),
        };

        let mut image = image::load_from_memory_with_format(data, image_format)
            .context("could not load image")?;

        image = image.resize(nwidth, nheight, image::imageops::FilterType::CatmullRom);

        let mut buffer = Cursor::new(Vec::new());
        image
            .write_to(&mut buffer, image::ImageFormat::WebP)
            .context("could not write image")?;

        Ok((buffer.into_inner(), Format::WebP))
    }
}
