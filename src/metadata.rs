use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Metadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub publisher: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub isbn: Option<String>,
    pub publication_date: Option<String>,
    pub cover_image: Option<Vec<u8>>,
    pub cover_image_path: Option<String>,
    pub tags: Option<Vec<String>>,
    pub format: Option<String>,
    pub custom_fields: std::collections::HashMap<String, String>,
}

impl Metadata {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    pub fn add_custom_field(&mut self, key: String, value: String) {
        self.custom_fields.insert(key, value);
    }
}
