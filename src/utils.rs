use crate::{EbookError, Result};
use std::path::Path;

pub fn detect_format(path: &Path) -> Result<String> {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| EbookError::UnsupportedFormat("No file extension".to_string()))?;

    match extension.to_lowercase().as_str() {
        "epub" => Ok("epub".to_string()),
        "mobi" => Ok("mobi".to_string()),
        "azw" | "azw3" => Ok("azw".to_string()),
        "fb2" => Ok("fb2".to_string()),
        "cbz" => Ok("cbz".to_string()),
        "txt" => Ok("txt".to_string()),
        "pdf" => Ok("pdf".to_string()),
        ext => Err(EbookError::UnsupportedFormat(format!(
            "Unsupported extension: {ext}"
        ))),
    }
}

pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

pub fn guess_mime_type(filename: &str) -> String {
    let extension = Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match extension.to_lowercase().as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "html" | "htm" => "application/xhtml+xml",
        "css" => "text/css",
        "js" => "application/javascript",
        _ => "application/octet-stream",
    }
    .to_string()
}
