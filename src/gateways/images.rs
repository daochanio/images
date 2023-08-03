use std::io::Cursor;

use async_trait::async_trait;

use crate::usecases::gateways::{ImageVariants, Images};

struct ImagesImpl {}

pub fn new() -> impl Images {
    ImagesImpl {}
}

// TODO:
// - add avif support?
// - convert gif to webp?
// - only scale down if image is larger than 250x250
// - scale up if image is smaller than 250x250?
#[async_trait]
impl Images for ImagesImpl {
    async fn resize(
        &self,
        data: &[u8],
        variant: ImageVariants,
    ) -> Result<(Vec<u8>, String), String> {
        let format = match image::guess_format(data) {
            Ok(f) => match f {
                image::ImageFormat::Jpeg => image::ImageFormat::Jpeg,
                image::ImageFormat::Png => image::ImageFormat::Png,
                image::ImageFormat::Gif => image::ImageFormat::Gif,
                image::ImageFormat::WebP => image::ImageFormat::WebP,
                // image::ImageFormat::Avif => image::ImageFormat::Avif,
                _ => return Err(format!("unsupported image format: {:?}", f)),
            },
            Err(e) => return Err(format!("could not derive image format: {}", e)),
        };

        match image::load_from_memory_with_format(data, format) {
            Ok(image) => {
                let resized_image = match variant {
                    ImageVariants::Thumbnail => image.thumbnail(250, 250),
                    ImageVariants::Original => image,
                };

                // Generally, we want to keep the original in its existing format and convert thumbnails to webp for optimized size
                let output_format = match (format, variant) {
                    (image::ImageFormat::Jpeg, ImageVariants::Original) => image::ImageFormat::Jpeg,
                    (image::ImageFormat::Png, ImageVariants::Original) => image::ImageFormat::Png,
                    (image::ImageFormat::Gif, ImageVariants::Original) => image::ImageFormat::Gif,
                    (image::ImageFormat::WebP, ImageVariants::Original) => image::ImageFormat::WebP,
                    // (image::ImageFormat::Avif, ImageVariants::Original) => image::ImageFormat::Avif,
                    (image::ImageFormat::Jpeg, ImageVariants::Thumbnail) => {
                        image::ImageFormat::WebP
                    }
                    (image::ImageFormat::Png, ImageVariants::Thumbnail) => image::ImageFormat::WebP,
                    (image::ImageFormat::Gif, ImageVariants::Thumbnail) => image::ImageFormat::Gif,
                    (image::ImageFormat::WebP, ImageVariants::Thumbnail) => {
                        image::ImageFormat::WebP
                    }
                    // (image::ImageFormat::Avif, ImageVariants::Original) => image::ImageFormat::Avif,
                    _ => return Err("unsupported output format".to_string()),
                };

                let mut buffer = Cursor::new(Vec::new());
                match resized_image.write_to(&mut buffer, output_format) {
                    Ok(_) => (),
                    Err(e) => return Err(format!("could not write image: {}", e)),
                }

                return Ok((buffer.into_inner(), format!("{:?}", output_format)));
            }
            Err(e) => return Err(format!("could not load image: {}", e)),
        }
    }
}
