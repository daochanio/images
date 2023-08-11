use std::io::Cursor;

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
    ) -> Result<(Vec<u8>, Format), String> {
        let (nwidth, nheight) = match variant {
            Variant::Thumbnail => (300, 300),
            Variant::Avatar => (125, 125),
            Variant::Original => (800, 800),
        };

        let mut image =
            match image::load_from_memory_with_format(data, get_image_format(input_format)?) {
                Ok(image) => image,
                Err(e) => return Err(format!("could not load image: {}", e)),
            };

        image = image.resize(nwidth, nheight, image::imageops::FilterType::CatmullRom);

        let mut buffer = Cursor::new(Vec::new());
        match image.write_to(&mut buffer, image::ImageFormat::WebP) {
            Ok(_) => (),
            Err(e) => return Err(format!("could not write image: {}", e)),
        }

        return Ok((buffer.into_inner(), Format::WebP));
    }
}

fn get_image_format(format: Format) -> Result<image::ImageFormat, String> {
    match format {
        Format::Jpeg => Ok(image::ImageFormat::Jpeg),
        Format::Png => Ok(image::ImageFormat::Png),
        Format::WebP => Ok(image::ImageFormat::WebP),
        _ => return Err(format!("unsupported image format: {:?}", format)),
    }
}
