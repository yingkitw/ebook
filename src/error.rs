use thiserror::Error;

pub type Result<T> = std::result::Result<T, EbookError>;

#[derive(Error, Debug)]
pub enum EbookError {
    #[error("IO error: {0}\nHint: Check if the file exists and you have read permissions")]
    Io(#[from] std::io::Error),

    #[error("ZIP error: {0}\nHint: The archive may be corrupted or not a valid ZIP file")]
    Zip(#[from] zip::result::ZipError),

    #[error("XML parsing error: {0}\nHint: The file may be corrupted or have invalid XML structure")]
    Xml(String),

    #[error("PDF error: {0}\nHint: Try repairing the PDF with a dedicated PDF repair tool")]
    Pdf(#[from] lopdf::Error),

    #[error("Unsupported format: {0}\nHint: Supported formats are EPUB, MOBI, PDF, FB2, CBZ, and TXT")]
    UnsupportedFormat(String),

    #[error("Invalid metadata: {0}\nHint: Ensure all required metadata fields (title, author) are provided")]
    InvalidMetadata(String),

    #[error("Parse error: {0}\nHint: The file structure may be corrupted or in an unexpected format")]
    Parse(String),

    #[error("Encoding error: {0}\nHint: The file may use a text encoding that is not UTF-8 compatible")]
    Encoding(String),

    #[error("Not found: {0}\nHint: Verify the required file or component exists in the ebook")]
    NotFound(String),

    #[error("Invalid file structure: {0}\nHint: The file may not be a valid ebook or is corrupted. Try using the 'repair' command")]
    InvalidStructure(String),

    #[error("Operation not supported: {0}\nHint: This feature is not yet implemented for this format")]
    NotSupported(String),

    #[error("Image processing error: {0}\nHint: Ensure the image is in a supported format (JPEG, PNG, GIF, WebP)")]
    ImageError(String),

    #[error("Conversion error: {0}\nHint: Not all format conversions are supported. Check documentation for supported conversions")]
    ConversionError(String),

    #[error("Validation error: {0}\nHint: Use the 'repair' command to fix common issues")]
    ValidationError(String),
}

impl From<xml::reader::Error> for EbookError {
    fn from(err: xml::reader::Error) -> Self {
        EbookError::Xml(err.to_string())
    }
}

impl From<quick_xml::Error> for EbookError {
    fn from(err: quick_xml::Error) -> Self {
        EbookError::Xml(err.to_string())
    }
}
