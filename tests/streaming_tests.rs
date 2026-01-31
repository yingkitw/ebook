use ebook_cli::formats::{TxtHandler, EpubHandler};
use ebook_cli::traits::{EbookReader, EbookWriter};
use ebook_cli::Metadata;
use tempfile::TempDir;
use std::fs::File;
use std::io::Write;

#[test]
fn test_txt_streaming_read() {
    let temp_dir = TempDir::new().unwrap();
    let txt_path = temp_dir.path().join("large.txt");
    
    // Create a large text file (15 MB)
    let mut file = File::create(&txt_path).unwrap();
    let chunk = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(100);
    for _ in 0..30000 {
        writeln!(file, "{}", chunk).unwrap();
    }
    drop(file);
    
    // Read using streaming
    let mut handler = TxtHandler::new();
    handler.read_from_file_streaming(&txt_path).unwrap();
    
    let content = handler.get_content().unwrap();
    assert!(content.len() > 10_000_000); // Should be > 10 MB
    assert!(content.contains("Lorem ipsum"));
}

#[test]
fn test_txt_streaming_write() {
    let temp_dir = TempDir::new().unwrap();
    let txt_path = temp_dir.path().join("large_out.txt");
    
    // Create large content
    let large_content = "Test line\n".repeat(1_000_000); // ~10 MB
    
    let mut handler = TxtHandler::new();
    handler.set_metadata(Metadata::new().with_title("Large File")).unwrap();
    handler.set_content(&large_content).unwrap();
    
    // Write using streaming
    handler.write_to_file_streaming(&txt_path).unwrap();
    
    assert!(txt_path.exists());
    let metadata = std::fs::metadata(&txt_path).unwrap();
    assert!(metadata.len() > 5_000_000); // Should be > 5 MB
}

#[test]
fn test_txt_streaming_threshold() {
    let temp_dir = TempDir::new().unwrap();
    
    // Small file - should use regular read
    let small_path = temp_dir.path().join("small.txt");
    let mut file = File::create(&small_path).unwrap();
    writeln!(file, "Small content").unwrap();
    drop(file);
    
    let mut handler = TxtHandler::new();
    handler.read_from_file_streaming(&small_path).unwrap();
    
    let content = handler.get_content().unwrap();
    assert!(content.contains("Small content"));
}

#[test]
fn test_epub_streaming_check() {
    let temp_dir = TempDir::new().unwrap();
    let small_path = temp_dir.path().join("small.epub");
    
    // Create a small EPUB
    let mut handler = EpubHandler::new();
    handler.set_metadata(Metadata::new().with_title("Small")).unwrap();
    handler.add_chapter("Ch1", "<h1>Chapter 1</h1>").unwrap();
    handler.write_to_file(&small_path).unwrap();
    
    // Should not need streaming for small file
    let should_stream = EpubHandler::should_use_streaming(&small_path).unwrap();
    assert!(!should_stream);
}

#[test]
fn test_streaming_preserves_content() {
    let temp_dir = TempDir::new().unwrap();
    let txt_path = temp_dir.path().join("preserve.txt");
    
    let original_content = "Line 1\nLine 2\nLine 3\n".repeat(100000);
    
    // Write with streaming
    let mut writer = TxtHandler::new();
    writer.set_metadata(Metadata::new().with_title("Preserve")).unwrap();
    writer.set_content(&original_content).unwrap();
    writer.write_to_file_streaming(&txt_path).unwrap();
    
    // Read with streaming
    let mut reader = TxtHandler::new();
    reader.read_from_file_streaming(&txt_path).unwrap();
    
    let read_content = reader.get_content().unwrap();
    assert_eq!(read_content.trim(), original_content.trim());
}

#[test]
fn test_streaming_metadata_preservation() {
    let temp_dir = TempDir::new().unwrap();
    let txt_path = temp_dir.path().join("metadata.txt");
    
    let mut handler = TxtHandler::new();
    handler.set_metadata(Metadata::new().with_title("Streaming Test")).unwrap();
    let content = "Content\n".repeat(100000);
    handler.set_content(&content).unwrap();
    handler.write_to_file_streaming(&txt_path).unwrap();
    
    let mut reader = TxtHandler::new();
    reader.read_from_file_streaming(&txt_path).unwrap();
    
    let metadata = reader.get_metadata().unwrap();
    assert_eq!(metadata.title, Some("metadata".to_string())); // Filename-based
}

#[test]
fn test_streaming_empty_file() {
    let temp_dir = TempDir::new().unwrap();
    let txt_path = temp_dir.path().join("empty.txt");
    
    let mut handler = TxtHandler::new();
    handler.set_metadata(Metadata::new().with_title("Empty")).unwrap();
    handler.set_content("").unwrap();
    handler.write_to_file_streaming(&txt_path).unwrap();
    
    assert!(txt_path.exists());
}

#[test]
fn test_streaming_unicode_content() {
    let temp_dir = TempDir::new().unwrap();
    let txt_path = temp_dir.path().join("unicode.txt");
    
    let unicode_content = "Hello 世界 🌍 Привет مرحبا\n".repeat(50000);
    
    let mut writer = TxtHandler::new();
    writer.set_content(&unicode_content).unwrap();
    writer.write_to_file_streaming(&txt_path).unwrap();
    
    let mut reader = TxtHandler::new();
    reader.read_from_file_streaming(&txt_path).unwrap();
    
    let content = reader.get_content().unwrap();
    assert!(content.contains("世界"));
    assert!(content.contains("🌍"));
    assert!(content.contains("Привет"));
}
