use ebook_cli::EbookError;

#[test]
fn test_error_io_includes_hint() {
    let err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
    let e = EbookError::Io(err);
    let msg = e.to_string();
    assert!(msg.contains("Hint:"));
    assert!(msg.contains("Check if the file exists"));
}

#[test]
fn test_error_unsupported_format_includes_hint() {
    let e = EbookError::UnsupportedFormat(".xyz".to_string());
    let msg = e.to_string();
    assert!(msg.contains("Hint:"));
    assert!(msg.contains("Supported formats"));
}

#[test]
fn test_error_not_supported_includes_hint() {
    let e = EbookError::NotSupported("nope".to_string());
    let msg = e.to_string();
    assert!(msg.contains("Hint:"));
    assert!(msg.contains("not yet implemented"));
}

#[test]
fn test_error_invalid_structure_includes_hint() {
    let e = EbookError::InvalidStructure("broken".to_string());
    let msg = e.to_string();
    assert!(msg.contains("Hint:"));
    assert!(msg.contains("repair"));
}

#[test]
fn test_error_validation_includes_hint() {
    let e = EbookError::ValidationError("bad".to_string());
    let msg = e.to_string();
    assert!(msg.contains("Hint:"));
    assert!(msg.contains("repair"));
}
