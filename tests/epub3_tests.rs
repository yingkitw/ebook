use ebook_cli::formats::{EpubHandler, EpubVersion};
use ebook_cli::traits::{EbookReader, EbookWriter};
use ebook_cli::Metadata;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_epub3_creation() {
    let temp_dir = TempDir::new().unwrap();
    let epub_path = temp_dir.path().join("test_v3.epub");
    
    let mut handler = EpubHandler::new();
    handler.set_epub_version(EpubVersion::V3);
    
    let mut metadata = Metadata::new();
    metadata.title = Some("EPUB 3.0 Test".to_string());
    metadata.author = Some("Test Author".to_string());
    
    handler.set_metadata(metadata).unwrap();
    handler.add_chapter("Chapter 1", "<h1>Chapter 1</h1><p>Content</p>").unwrap();
    handler.add_chapter("Chapter 2", "<h1>Chapter 2</h1><p>More content</p>").unwrap();
    handler.write_to_file(&epub_path).unwrap();
    
    assert!(epub_path.exists());
    
    // Verify it's a valid EPUB by reading it back
    let mut reader = EpubHandler::new();
    reader.read_from_file(&epub_path).unwrap();
    
    let read_metadata = reader.get_metadata().unwrap();
    assert_eq!(read_metadata.title, Some("EPUB 3.0 Test".to_string()));
}

#[test]
fn test_epub3_with_nav() {
    let temp_dir = TempDir::new().unwrap();
    let epub_path = temp_dir.path().join("test_nav.epub");
    
    let mut handler = EpubHandler::new();
    handler.set_epub_version(EpubVersion::V3);
    
    handler.set_metadata(Metadata::new().with_title("Nav Test")).unwrap();
    handler.add_chapter("Introduction", "<h1>Introduction</h1>").unwrap();
    handler.add_chapter("Main Content", "<h1>Main Content</h1>").unwrap();
    handler.add_chapter("Conclusion", "<h1>Conclusion</h1>").unwrap();
    handler.write_to_file(&epub_path).unwrap();
    
    // Verify nav.xhtml exists in the EPUB
    use zip::ZipArchive;
    let file = fs::File::open(&epub_path).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();
    
    let mut nav_found = false;
    for i in 0..archive.len() {
        let file = archive.by_index(i).unwrap();
        if file.name() == "OEBPS/nav.xhtml" {
            nav_found = true;
            break;
        }
    }
    
    assert!(nav_found, "nav.xhtml should exist in EPUB 3.0");
}

#[test]
fn test_epub2_creation() {
    let temp_dir = TempDir::new().unwrap();
    let epub_path = temp_dir.path().join("test_v2.epub");
    
    let mut handler = EpubHandler::new();
    handler.set_epub_version(EpubVersion::V2);
    
    handler.set_metadata(Metadata::new().with_title("EPUB 2.0 Test")).unwrap();
    handler.add_chapter("Chapter 1", "<h1>Chapter 1</h1>").unwrap();
    handler.write_to_file(&epub_path).unwrap();
    
    assert!(epub_path.exists());
}

#[test]
fn test_epub_version_default() {
    let handler = EpubHandler::new();
    assert_eq!(handler.get_epub_version(), EpubVersion::V3);
}

#[test]
fn test_epub_version_switching() {
    let mut handler = EpubHandler::new();
    assert_eq!(handler.get_epub_version(), EpubVersion::V3);
    
    handler.set_epub_version(EpubVersion::V2);
    assert_eq!(handler.get_epub_version(), EpubVersion::V2);
    
    handler.set_epub_version(EpubVersion::V3);
    assert_eq!(handler.get_epub_version(), EpubVersion::V3);
}

#[test]
fn test_epub3_with_images() {
    let temp_dir = TempDir::new().unwrap();
    let epub_path = temp_dir.path().join("test_images.epub");
    
    let mut handler = EpubHandler::new();
    handler.set_epub_version(EpubVersion::V3);
    
    handler.set_metadata(Metadata::new().with_title("Images Test")).unwrap();
    handler.add_chapter("Chapter 1", "<h1>Chapter 1</h1><img src=\"image.png\"/>").unwrap();
    
    let test_image = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    handler.add_image("image.png", test_image).unwrap();
    
    handler.write_to_file(&epub_path).unwrap();
    
    let mut reader = EpubHandler::new();
    reader.read_from_file(&epub_path).unwrap();
    
    let images = reader.extract_images().unwrap();
    assert_eq!(images.len(), 1);
}

#[test]
fn test_epub3_metadata_preservation() {
    let temp_dir = TempDir::new().unwrap();
    let epub_path = temp_dir.path().join("test_metadata.epub");
    
    let mut handler = EpubHandler::new();
    handler.set_epub_version(EpubVersion::V3);
    
    let mut metadata = Metadata::new();
    metadata.title = Some("Metadata Test".to_string());
    metadata.author = Some("John Doe".to_string());
    metadata.language = Some("en-US".to_string());
    
    handler.set_metadata(metadata).unwrap();
    handler.add_chapter("Test", "<h1>Test</h1>").unwrap();
    handler.write_to_file(&epub_path).unwrap();
    
    let mut reader = EpubHandler::new();
    reader.read_from_file(&epub_path).unwrap();
    
    let read_metadata = reader.get_metadata().unwrap();
    assert_eq!(read_metadata.title, Some("Metadata Test".to_string()));
    assert_eq!(read_metadata.author, Some("John Doe".to_string()));
}
