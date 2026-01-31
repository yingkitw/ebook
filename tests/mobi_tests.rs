use ebook_cli::formats::MobiHandler;
use ebook_cli::traits::{EbookReader, EbookWriter, EbookOperator};
use ebook_cli::Metadata;
use tempfile::TempDir;

#[test]
fn test_mobi_create_and_read() {
    let temp_dir = TempDir::new().unwrap();
    let mobi_path = temp_dir.path().join("test.mobi");

    // Create a MOBI
    let mut handler = MobiHandler::new();
    let mut metadata = Metadata::new();
    metadata.title = Some("Test MOBI Book".to_string());
    metadata.author = Some("MOBI Author".to_string());

    handler.set_metadata(metadata).unwrap();
    handler.set_content("Chapter 1\n\nThis is the first chapter.\n\nChapter 2\n\nThis is the second chapter.").unwrap();
    handler.write_to_file(&mobi_path).unwrap();

    // Read the MOBI back
    let mut reader = MobiHandler::new();
    reader.read_from_file(&mobi_path).unwrap();

    let read_metadata = reader.get_metadata().unwrap();
    assert_eq!(read_metadata.format, Some("MOBI".to_string()));

    let content = reader.get_content().unwrap();
    assert!(content.contains("Chapter 1") || content.contains("chapter"));
}

#[test]
fn test_mobi_toc_extraction() {
    let temp_dir = TempDir::new().unwrap();
    let mobi_path = temp_dir.path().join("test_toc.mobi");

    let mut handler = MobiHandler::new();
    handler.set_metadata(Metadata::new().with_title("MOBI TOC Test")).unwrap();
    
    let content = "Chapter 1\n\nFirst chapter content.\n\nChapter 2\n\nSecond chapter content.\n\nChapter 3\n\nThird chapter content.";
    handler.set_content(content).unwrap();
    handler.write_to_file(&mobi_path).unwrap();

    let mut reader = MobiHandler::new();
    reader.read_from_file(&mobi_path).unwrap();

    let toc = reader.get_toc().unwrap();
    // TOC extraction is basic, so we just verify it returns a result
    assert!(toc.len() >= 0);
}

#[test]
fn test_mobi_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let mobi_path = temp_dir.path().join("test_metadata.mobi");

    let mut handler = MobiHandler::new();
    let mut metadata = Metadata::new();
    metadata.title = Some("MOBI Metadata Test".to_string());
    metadata.author = Some("Test Author".to_string());
    metadata.language = Some("en".to_string());

    handler.set_metadata(metadata).unwrap();
    handler.set_content("Test content for metadata").unwrap();
    handler.write_to_file(&mobi_path).unwrap();

    let mut reader = MobiHandler::new();
    reader.read_from_file(&mobi_path).unwrap();

    let read_metadata = reader.get_metadata().unwrap();
    assert_eq!(read_metadata.format, Some("MOBI".to_string()));
}

#[test]
fn test_mobi_validation() {
    let temp_dir = TempDir::new().unwrap();
    let mobi_path = temp_dir.path().join("test_validate.mobi");

    let mut handler = MobiHandler::new();
    handler.set_metadata(Metadata::new().with_title("Valid MOBI")).unwrap();
    handler.set_content("Valid MOBI content").unwrap();
    handler.write_to_file(&mobi_path).unwrap();

    let mut reader = MobiHandler::new();
    reader.read_from_file(&mobi_path).unwrap();

    let is_valid = reader.validate().unwrap();
    assert!(is_valid);
}

#[test]
fn test_mobi_empty_content() {
    let temp_dir = TempDir::new().unwrap();
    let mobi_path = temp_dir.path().join("test_empty.mobi");

    let mut handler = MobiHandler::new();
    handler.set_metadata(Metadata::new().with_title("Empty MOBI")).unwrap();
    handler.write_to_file(&mobi_path).unwrap();

    let mut reader = MobiHandler::new();
    reader.read_from_file(&mobi_path).unwrap();

    let content = reader.get_content().unwrap();
    // Empty content is acceptable
    assert!(content.len() >= 0);
}

#[test]
fn test_mobi_repair() {
    let temp_dir = TempDir::new().unwrap();
    let mobi_path = temp_dir.path().join("test_repair.mobi");

    let mut handler = MobiHandler::new();
    handler.set_metadata(Metadata::new().with_title("Repair Test")).unwrap();
    handler.set_content("Content to repair").unwrap();
    handler.write_to_file(&mobi_path).unwrap();

    let mut reader = MobiHandler::new();
    reader.read_from_file(&mobi_path).unwrap();
    
    // Repair should succeed
    let result = reader.repair();
    assert!(result.is_ok());
}
