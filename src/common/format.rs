use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub enum Format {
    Jpeg,
    Png,
    WebP,
    Gif,
    Mp4,
}

impl Format {
    pub fn infer(data: &[u8]) -> Result<Format> {
        match infer::get(data) {
            Some(kind) => match kind.extension() {
                "jpeg" => Ok(Format::Jpeg),
                "jpg" => Ok(Format::Jpeg),
                "png" => Ok(Format::Png),
                "webp" => Ok(Format::WebP),
                "gif" => Ok(Format::Gif),
                "mp4" => Ok(Format::Mp4),
                _ => Err(anyhow!("unsupported format")),
            },
            None => Err(anyhow!("could not get format")),
        }
    }

    pub fn content_type(&self) -> String {
        match self {
            Format::Jpeg => String::from("image/jpeg"),
            Format::Png => String::from("image/png"),
            Format::WebP => String::from("image/webp"),
            Format::Gif => String::from("image/gif"),
            Format::Mp4 => String::from("video/mp4"),
        }
    }

    pub fn extension(&self) -> String {
        match self {
            Format::Jpeg => String::from("jpg"),
            Format::Png => String::from("png"),
            Format::WebP => String::from("webp"),
            Format::Gif => String::from("gif"),
            Format::Mp4 => String::from("mp4"),
        }
    }
}
