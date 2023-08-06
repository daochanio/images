use std::io::Cursor;

use async_trait::async_trait;
use image::codecs::gif::{GifDecoder, GifEncoder, Repeat};
use image::{AnimationDecoder, DynamicImage, Frame};

use crate::{common::enums::ImageVariants, usecases::gateways::Images};

struct ImagesImpl {}

pub fn new() -> impl Images {
    ImagesImpl {}
}

#[async_trait]
impl Images for ImagesImpl {
    // TODO:
    // - add avif support?
    async fn resize(
        &self,
        data: &[u8],
        variant: ImageVariants,
    ) -> Result<(Vec<u8>, String), String> {
        let (nwidth, nheight) = match variant {
            ImageVariants::Thumbnail => (300, 300),
            ImageVariants::Avatar => (125, 125),
            ImageVariants::Original => (800, 800),
        };

        return match image::guess_format(data) {
            Ok(format) => match format {
                image::ImageFormat::Jpeg | image::ImageFormat::Png | image::ImageFormat::WebP => {
                    self.resize_image(data, nwidth, nheight, format)
                }
                image::ImageFormat::Gif => self.resize_gif(data, nwidth, nheight),
                _ => Err(format!("unsupported image format: {:?}", format)),
            },
            Err(e) => Err(format!("could not derive image format: {}", e)),
        };
    }

    fn get_content_type(&self, data: &[u8]) -> Result<String, String> {
        return match image::guess_format(data) {
            Ok(format) => match format {
                image::ImageFormat::Jpeg => Ok(String::from("image/jpeg")),
                image::ImageFormat::Png => Ok(String::from("image/png")),
                image::ImageFormat::WebP => Ok(String::from("image/webp")),
                image::ImageFormat::Gif => Ok(String::from("image/gif")),
                _ => Err(format!("unsupported image format: {:?}", format)),
            },
            Err(e) => Err(format!("could not derive image format: {}", e)),
        };
    }
}

impl ImagesImpl {
    fn resize_image(
        &self,
        data: &[u8],
        nwidth: u32,
        nheight: u32,
        format: image::ImageFormat,
    ) -> Result<(Vec<u8>, String), String> {
        let mut image = match image::load_from_memory_with_format(data, format) {
            Ok(image) => image,
            Err(e) => return Err(format!("could not load image: {}", e)),
        };

        image = image.resize(nwidth, nheight, image::imageops::FilterType::CatmullRom);

        let mut buffer = Cursor::new(Vec::new());
        match image.write_to(&mut buffer, image::ImageFormat::WebP) {
            Ok(_) => (),
            Err(e) => return Err(format!("could not write image: {}", e)),
        }

        return Ok((buffer.into_inner(), String::from("image/webp")));
    }

    fn resize_gif(
        &self,
        data: &[u8],
        nwidth: u32,
        nheight: u32,
    ) -> Result<(Vec<u8>, String), String> {
        let mut output = Vec::new();
        let mut encoder = GifEncoder::new_with_speed(&mut output, 20);
        encoder.set_repeat(Repeat::Infinite).unwrap();

        let decoder = GifDecoder::new(data).unwrap();
        let frames = decoder.into_frames();

        for frame in frames {
            let image = DynamicImage::from(frame.unwrap().into_buffer());
            let resized = image.resize(nwidth, nheight, image::imageops::FilterType::CatmullRom);
            let resized_frame = Frame::new(resized.into_rgba8());
            encoder.encode_frame(resized_frame).unwrap();
        }

        drop(encoder);

        return Ok((output, String::from("image/gif")));
    }
}
