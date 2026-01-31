use ebook_cli::formats::CbzHandler;
use ebook_cli::traits::{EbookReader, EbookWriter};
use ebook_cli::Metadata;
use tempfile::TempDir;

fn create_test_image() -> Vec<u8> {
    // Create a minimal valid PNG (1x1 red pixel)
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
        0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
        0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41,
        0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
        0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00,
        0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
        0x42, 0x60, 0x82,
    ]
}

#[test]
fn test_cbz_with_comic_info() {
    let temp_dir = TempDir::new().unwrap();
    let cbz_path = temp_dir.path().join("test_comic.cbz");

    // Create a CBZ with metadata
    let mut handler = CbzHandler::new();
    let mut metadata = Metadata::new();
    metadata.title = Some("Test Comic".to_string());
    metadata.author = Some("Test Writer".to_string());
    metadata.publisher = Some("Test Publisher".to_string());
    metadata.description = Some("A test comic book".to_string());
    metadata.language = Some("en".to_string());
    metadata.tags = Some(vec!["Action".to_string(), "Adventure".to_string()]);

    handler.set_metadata(metadata).unwrap();
    handler.add_image("page01.png", create_test_image()).unwrap();
    handler.add_image("page02.png", create_test_image()).unwrap();
    handler.write_to_file(&cbz_path).unwrap();

    // Read it back
    let mut reader = CbzHandler::new();
    reader.read_from_file(&cbz_path).unwrap();

    let read_metadata = reader.get_metadata().unwrap();
    assert_eq!(read_metadata.title, Some("Test Comic".to_string()));
    assert_eq!(read_metadata.author, Some("Test Writer".to_string()));
    assert_eq!(read_metadata.publisher, Some("Test Publisher".to_string()));
    assert_eq!(read_metadata.description, Some("A test comic book".to_string()));
    assert_eq!(read_metadata.language, Some("en".to_string()));
}

#[test]
fn test_cbz_comic_info_tags() {
    let temp_dir = TempDir::new().unwrap();
    let cbz_path = temp_dir.path().join("test_tags.cbz");

    let mut handler = CbzHandler::new();
    let mut metadata = Metadata::new();
    metadata.title = Some("Tagged Comic".to_string());
    metadata.tags = Some(vec![
        "Superhero".to_string(),
        "Sci-Fi".to_string(),
        "Drama".to_string(),
    ]);

    handler.set_metadata(metadata).unwrap();
    handler.add_image("cover.png", create_test_image()).unwrap();
    handler.write_to_file(&cbz_path).unwrap();

    let mut reader = CbzHandler::new();
    reader.read_from_file(&cbz_path).unwrap();

    let read_metadata = reader.get_metadata().unwrap();
    assert!(read_metadata.tags.is_some());
    let tags = read_metadata.tags.unwrap();
    assert_eq!(tags.len(), 3);
    assert!(tags.contains(&"Superhero".to_string()));
}

#[test]
fn test_cbz_page_count() {
    let temp_dir = TempDir::new().unwrap();
    let cbz_path = temp_dir.path().join("test_pages.cbz");

    let mut handler = CbzHandler::new();
    handler.set_metadata(Metadata::new().with_title("Page Count Test")).unwrap();
    
    for i in 1..=10 {
        handler.add_image(&format!("page{:02}.png", i), create_test_image()).unwrap();
    }
    
    handler.write_to_file(&cbz_path).unwrap();

    let mut reader = CbzHandler::new();
    reader.read_from_file(&cbz_path).unwrap();

    let images = reader.extract_images().unwrap();
    assert_eq!(images.len(), 10);
}

#[test]
fn test_cbz_without_comic_info() {
    let temp_dir = TempDir::new().unwrap();
    let cbz_path = temp_dir.path().join("no_metadata.cbz");

    // Create a minimal CBZ
    let mut handler = CbzHandler::new();
    handler.add_image("page01.png", create_test_image()).unwrap();
    handler.write_to_file(&cbz_path).unwrap();

    // Read it back - should still work
    let mut reader = CbzHandler::new();
    reader.read_from_file(&cbz_path).unwrap();

    let images = reader.extract_images().unwrap();
    assert_eq!(images.len(), 1);
}

#[test]
fn test_cbz_metadata_preservation() {
    let temp_dir = TempDir::new().unwrap();
    let cbz_path = temp_dir.path().join("preserve.cbz");

    let mut handler = CbzHandler::new();
    let mut metadata = Metadata::new();
    metadata.title = Some("Preservation Test".to_string());
    metadata.author = Some("Original Author".to_string());
    metadata.publisher = Some("Original Publisher".to_string());

    handler.set_metadata(metadata).unwrap();
    handler.add_image("page.png", create_test_image()).unwrap();
    handler.write_to_file(&cbz_path).unwrap();

    // Read and verify all metadata is preserved
    let mut reader = CbzHandler::new();
    reader.read_from_file(&cbz_path).unwrap();

    let read_metadata = reader.get_metadata().unwrap();
    assert_eq!(read_metadata.title, Some("Preservation Test".to_string()));
    assert_eq!(read_metadata.author, Some("Original Author".to_string()));
    assert_eq!(read_metadata.publisher, Some("Original Publisher".to_string()));
    assert_eq!(read_metadata.format, Some("CBZ".to_string()));
}
