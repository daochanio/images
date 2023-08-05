use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Image {
    file_name: String,
    original_url: String,
    thumbnail_url: String,
}

pub fn new(file_name: String, original_url: String, thumbnail_url: String) -> Image {
    Image {
        file_name,
        original_url,
        thumbnail_url,
    }
}
