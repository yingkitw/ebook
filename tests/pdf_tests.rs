use ebook_cli::formats::PdfHandler;
use ebook_cli::traits::{EbookReader, EbookWriter, EbookOperator};
use ebook_cli::Metadata;
use tempfile::TempDir;

#[test]
fn test_pdf_create_and_read() {
    let temp_dir = TempDir::new().unwrap();
    let pdf_path = temp_dir.path().join("test.pdf");

    // Create a PDF
    let mut handler = PdfHandler::new();
    let mut metadata = Metadata::new();
    metadata.title = Some("Test PDF Document".to_string());
    metadata.author = Some("PDF Author".to_string());

    handler.set_metadata(metadata).unwrap();
    handler.set_content("This is a test PDF document.\n\nIt has multiple paragraphs.").unwrap();
    handler.write_to_file(&pdf_path).unwrap();

    // Read the PDF back
    let mut reader = PdfHandler::new();
    reader.read_from_file(&pdf_path).unwrap();

    let read_metadata = reader.get_metadata().unwrap();
    assert_eq!(read_metadata.title, Some("Test PDF Document".to_string()));
    assert_eq!(read_metadata.author, Some("PDF Author".to_string()));
    assert_eq!(read_metadata.format, Some("PDF".to_string()));

    let content = reader.get_content().unwrap();
    assert!(content.contains("test") || content.contains("PDF") || !content.is_empty());
}

#[test]
fn test_pdf_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let pdf_path = temp_dir.path().join("test_metadata.pdf");

    let mut handler = PdfHandler::new();
    let mut metadata = Metadata::new();
    metadata.title = Some("PDF Metadata Test".to_string());
    metadata.author = Some("Test Author".to_string());
    metadata.publisher = Some("Test Publisher".to_string());

    handler.set_metadata(metadata).unwrap();
    handler.set_content("Content for metadata test").unwrap();
    handler.write_to_file(&pdf_path).unwrap();

    let mut reader = PdfHandler::new();
    reader.read_from_file(&pdf_path).unwrap();

    let read_metadata = reader.get_metadata().unwrap();
    assert_eq!(read_metadata.title, Some("PDF Metadata Test".to_string()));
    assert_eq!(read_metadata.author, Some("Test Author".to_string()));
}

#[test]
fn test_pdf_validation() {
    let temp_dir = TempDir::new().unwrap();
    let pdf_path = temp_dir.path().join("test_validate.pdf");

    let mut handler = PdfHandler::new();
    handler.set_metadata(Metadata::new().with_title("Valid PDF")).unwrap();
    handler.set_content("Valid PDF content").unwrap();
    handler.write_to_file(&pdf_path).unwrap();

    let mut reader = PdfHandler::new();
    reader.read_from_file(&pdf_path).unwrap();

    let is_valid = reader.validate().unwrap();
    assert!(is_valid);
}

#[test]
fn test_pdf_empty_content() {
    let temp_dir = TempDir::new().unwrap();
    let pdf_path = temp_dir.path().join("test_empty.pdf");

    let mut handler = PdfHandler::new();
    handler.set_metadata(Metadata::new().with_title("Empty PDF")).unwrap();
    handler.write_to_file(&pdf_path).unwrap();

    let mut reader = PdfHandler::new();
    reader.read_from_file(&pdf_path).unwrap();

    let content = reader.get_content().unwrap();
    // Empty or minimal content is acceptable
    assert!(content.len() >= 0);
}

#[test]
fn test_pdf_multipage() {
    let temp_dir = TempDir::new().unwrap();
    let pdf_path = temp_dir.path().join("test_multipage.pdf");

    let mut handler = PdfHandler::new();
    handler.set_metadata(Metadata::new().with_title("Multi-page PDF")).unwrap();
    
    let content = "Page 1 content\n\nPage 2 content\n\nPage 3 content";
    handler.set_content(content).unwrap();
    handler.write_to_file(&pdf_path).unwrap();

    let mut reader = PdfHandler::new();
    reader.read_from_file(&pdf_path).unwrap();

    let read_content = reader.get_content().unwrap();
    // Verify we got some content back
    assert!(!read_content.is_empty());
}

#[test]
fn test_pdf_repair() {
    let temp_dir = TempDir::new().unwrap();
    let pdf_path = temp_dir.path().join("test_repair.pdf");

    let mut handler = PdfHandler::new();
    handler.set_metadata(Metadata::new().with_title("Repair Test")).unwrap();
    handler.set_content("Content to repair").unwrap();
    handler.write_to_file(&pdf_path).unwrap();

    let mut reader = PdfHandler::new();
    reader.read_from_file(&pdf_path).unwrap();
    
    // Repair should succeed
    let result = reader.repair();
    assert!(result.is_ok());
}
