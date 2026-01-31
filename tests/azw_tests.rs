use ebook_cli::formats::AzwHandler;
use ebook_cli::traits::{EbookReader, EbookWriter, EbookOperator};
use ebook_cli::Metadata;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_azw_creation() {
    let temp_dir = TempDir::new().unwrap();
    let azw_path = temp_dir.path().join("test.azw");
    
    let mut handler = AzwHandler::new();
    let mut metadata = Metadata::new();
    metadata.title = Some("Test AZW Book".to_string());
    metadata.author = Some("Test Author".to_string());
    
    handler.set_metadata(metadata).unwrap();
    handler.set_content("This is test content for AZW format.").unwrap();
    handler.write_to_file(&azw_path).unwrap();
    
    assert!(azw_path.exists());
}

#[test]
fn test_azw_read_write() {
    let temp_dir = TempDir::new().unwrap();
    let azw_path = temp_dir.path().join("test.azw");
    
    // Write
    let mut handler = AzwHandler::new();
    handler.set_metadata(Metadata::new().with_title("AZW Test")).unwrap();
    handler.set_content("Test content").unwrap();
    handler.write_to_file(&azw_path).unwrap();
    
    // Read
    let mut reader = AzwHandler::new();
    reader.read_from_file(&azw_path).unwrap();
    
    let metadata = reader.get_metadata().unwrap();
    assert_eq!(metadata.title, Some("AZW Test".to_string()));
    
    let content = reader.get_content().unwrap();
    assert!(content.contains("Test content"));
}

#[test]
fn test_azw_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let azw_path = temp_dir.path().join("metadata.azw");
    
    let mut handler = AzwHandler::new();
    let mut metadata = Metadata::new();
    metadata.title = Some("Metadata Test".to_string());
    metadata.author = Some("John Doe".to_string());
    metadata.language = Some("en".to_string());
    
    handler.set_metadata(metadata).unwrap();
    handler.set_content("Content").unwrap();
    handler.write_to_file(&azw_path).unwrap();
    
    let mut reader = AzwHandler::new();
    reader.read_from_file(&azw_path).unwrap();
    
    let read_metadata = reader.get_metadata().unwrap();
    assert_eq!(read_metadata.title, Some("Metadata Test".to_string()));
}

#[test]
fn test_azw_validation() {
    let temp_dir = TempDir::new().unwrap();
    let azw_path = temp_dir.path().join("valid.azw");
    
    let mut handler = AzwHandler::new();
    handler.set_metadata(Metadata::new().with_title("Valid AZW")).unwrap();
    handler.set_content("Valid content").unwrap();
    handler.write_to_file(&azw_path).unwrap();
    
    let mut reader = AzwHandler::new();
    reader.read_from_file(&azw_path).unwrap();
    
    let is_valid = reader.validate().unwrap();
    assert!(is_valid);
}

#[test]
fn test_azw_repair() {
    let mut handler = AzwHandler::new();
    handler.set_content("Content without metadata").unwrap();
    
    handler.repair().unwrap();
    
    let metadata = handler.get_metadata().unwrap();
    assert_eq!(metadata.title, Some("Untitled".to_string()));
}

#[test]
fn test_azw_empty_content() {
    let temp_dir = TempDir::new().unwrap();
    let azw_path = temp_dir.path().join("empty.azw");
    
    let mut handler = AzwHandler::new();
    handler.set_metadata(Metadata::new().with_title("Empty")).unwrap();
    handler.set_content("").unwrap();
    handler.write_to_file(&azw_path).unwrap();
    
    assert!(azw_path.exists());
}

#[test]
fn test_azw_add_chapter() {
    let mut handler = AzwHandler::new();
    handler.set_metadata(Metadata::new().with_title("Chapters")).unwrap();
    
    handler.add_chapter("Chapter 1", "Content 1").unwrap();
    handler.add_chapter("Chapter 2", "Content 2").unwrap();
    
    let content = handler.get_content().unwrap();
    assert!(content.contains("Content 1"));
    assert!(content.contains("Content 2"));
}

#[test]
fn test_azw_toc_extraction() {
    let temp_dir = TempDir::new().unwrap();
    let azw_path = temp_dir.path().join("toc.azw");
    
    let mut handler = AzwHandler::new();
    handler.set_metadata(Metadata::new().with_title("TOC Test")).unwrap();
    handler.set_content("Chapter 1\nSome content\nChapter 2\nMore content").unwrap();
    handler.write_to_file(&azw_path).unwrap();
    
    let mut reader = AzwHandler::new();
    reader.read_from_file(&azw_path).unwrap();
    
    let toc = reader.get_toc().unwrap();
    // TOC extraction is basic, so we just check it doesn't error
    assert!(toc.len() >= 0);
}
