use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Image {
    file_name: String,
    original: Header,
    formatted: Header,
}

#[derive(Debug, Clone, Serialize)]
struct Header {
    url: String,
    content_type: String,
}

pub fn new(
    file_name: String,
    original_url: String,
    original_content_type: String,
    formatted_url: String,
    formatted_content_type: String,
) -> Image {
    Image {
        file_name,
        original: Header {
            url: original_url,
            content_type: original_content_type,
        },
        formatted: Header {
            url: formatted_url,
            content_type: formatted_content_type,
        },
    }
}
