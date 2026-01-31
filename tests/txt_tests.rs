use ebook_cli::formats::TxtHandler;
use ebook_cli::traits::{EbookReader, EbookWriter, EbookOperator};
use ebook_cli::Metadata;
use tempfile::NamedTempFile;
use std::io::Write;

#[test]
fn test_txt_read_write() {
    let mut temp_file = NamedTempFile::new().unwrap();
    let content = "This is a test ebook.\nChapter 1\nSome content here.";
    temp_file.write_all(content.as_bytes()).unwrap();
    
    let mut handler = TxtHandler::new();
    handler.read_from_file(temp_file.path()).unwrap();
    
    let read_content = handler.get_content().unwrap();
    assert_eq!(read_content, content);
}

#[test]
fn test_txt_metadata() {
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(b"Test content").unwrap();
    
    let mut handler = TxtHandler::new();
    handler.read_from_file(temp_file.path()).unwrap();
    
    let metadata = handler.get_metadata().unwrap();
    assert!(metadata.title.is_some());
    assert_eq!(metadata.format, Some("TXT".to_string()));
}

#[test]
fn test_txt_write() {
    let temp_file = NamedTempFile::new().unwrap();
    
    let mut handler = TxtHandler::new();
    let mut metadata = Metadata::new();
    metadata.title = Some("Test Book".to_string());
    metadata.author = Some("Test Author".to_string());
    
    handler.set_metadata(metadata).unwrap();
    handler.set_content("This is test content").unwrap();
    handler.write_to_file(temp_file.path()).unwrap();
    
    let mut read_handler = TxtHandler::new();
    read_handler.read_from_file(temp_file.path()).unwrap();
    let content = read_handler.get_content().unwrap();
    
    assert_eq!(content, "This is test content");
}

#[test]
fn test_txt_validate() {
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(b"Some content").unwrap();
    
    let mut handler = TxtHandler::new();
    handler.read_from_file(temp_file.path()).unwrap();
    
    assert!(handler.validate().unwrap());
}

#[test]
fn test_txt_repair() {
    let mut handler = TxtHandler::new();
    handler.set_content("  \n  Some content with whitespace  \n  ").unwrap();
    
    handler.repair().unwrap();
    
    let metadata = handler.get_metadata().unwrap();
    assert!(metadata.title.is_some());
}

#[test]
fn test_txt_toc_detection() {
    let mut temp_file = NamedTempFile::new().unwrap();
    let content = "Introduction\n\nChapter 1\nFirst chapter content\n\nChapter 2\nSecond chapter content";
    temp_file.write_all(content.as_bytes()).unwrap();
    
    let mut handler = TxtHandler::new();
    handler.read_from_file(temp_file.path()).unwrap();
    
    let toc = handler.get_toc().unwrap();
    assert_eq!(toc.len(), 2);
    assert_eq!(toc[0].title, "Chapter 1");
    assert_eq!(toc[1].title, "Chapter 2");
}
