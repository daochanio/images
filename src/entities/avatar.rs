use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Avatar {
    file_name: String,
    url: String,
}

pub fn new(file_name: String, url: String) -> Avatar {
    Avatar { file_name, url }
}
