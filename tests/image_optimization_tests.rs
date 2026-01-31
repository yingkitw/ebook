use ebook_cli::formats::{CbzHandler, EpubHandler};
use ebook_cli::traits::{EbookReader, EbookWriter};
use ebook_cli::image_optimizer::OptimizationOptions;
use ebook_cli::Metadata;
use tempfile::TempDir;

fn create_large_test_image() -> Vec<u8> {
    // Create a larger test image (100x100 red square)
    use image::{RgbImage, DynamicImage, ImageFormat};
    use std::io::Cursor;
    
    let img = DynamicImage::ImageRgb8(RgbImage::from_pixel(
        100,
        100,
        image::Rgb([255, 0, 0]),
    ));
    
    let mut buffer = Cursor::new(Vec::new());
    img.write_to(&mut buffer, ImageFormat::Png).unwrap();
    buffer.into_inner()
}

#[test]
fn test_cbz_image_optimization() {
    let temp_dir = TempDir::new().unwrap();
    let cbz_path = temp_dir.path().join("optimize.cbz");

    let mut handler = CbzHandler::new();
    handler.set_metadata(Metadata::new().with_title("Optimization Test")).unwrap();
    
    // Add multiple images
    for i in 1..=5 {
        handler.add_image(&format!("page{:02}.png", i), create_large_test_image()).unwrap();
    }
    
    handler.write_to_file(&cbz_path).unwrap();

    // Read it back and optimize
    let mut reader = CbzHandler::new();
    reader.read_from_file(&cbz_path).unwrap();

    let options = OptimizationOptions::default()
        .with_max_dimensions(800, 800)
        .with_quality(80);
    
    let savings = reader.optimize_images(options);
    assert!(savings.is_ok());
    
    // Verify images are still present
    let images = reader.extract_images().unwrap();
    assert_eq!(images.len(), 5);
}

#[test]
fn test_epub_image_optimization() {
    let temp_dir = TempDir::new().unwrap();
    let epub_path = temp_dir.path().join("optimize.epub");

    let mut handler = EpubHandler::new();
    handler.set_metadata(Metadata::new().with_title("EPUB Optimization")).unwrap();
    handler.add_chapter("Chapter 1", "<h1>Chapter 1</h1><p>Content</p>").unwrap();
    
    // Add images
    handler.add_image("image1.png", create_large_test_image()).unwrap();
    handler.add_image("image2.png", create_large_test_image()).unwrap();
    
    handler.write_to_file(&epub_path).unwrap();

    // Read and optimize
    let mut reader = EpubHandler::new();
    reader.read_from_file(&epub_path).unwrap();

    let options = OptimizationOptions::default()
        .with_max_dimensions(1024, 1024)
        .with_quality(85);
    
    let savings = reader.optimize_images(options);
    assert!(savings.is_ok());
    
    let images = reader.extract_images().unwrap();
    assert_eq!(images.len(), 2);
}

#[test]
fn test_optimization_quality_settings() {
    let temp_dir = TempDir::new().unwrap();
    let cbz_path = temp_dir.path().join("quality.cbz");

    let mut handler = CbzHandler::new();
    handler.set_metadata(Metadata::new().with_title("Quality Test")).unwrap();
    handler.add_image("test.png", create_large_test_image()).unwrap();
    handler.write_to_file(&cbz_path).unwrap();

    let mut reader = CbzHandler::new();
    reader.read_from_file(&cbz_path).unwrap();

    // Test different quality settings
    let options_high = OptimizationOptions::default().with_quality(95);
    let savings_high = reader.optimize_images(options_high).unwrap();
    
    // High quality should still provide some savings
    assert!(savings_high >= 0);
}

#[test]
fn test_optimization_no_resize() {
    let temp_dir = TempDir::new().unwrap();
    let cbz_path = temp_dir.path().join("no_resize.cbz");

    let mut handler = CbzHandler::new();
    handler.set_metadata(Metadata::new().with_title("No Resize Test")).unwrap();
    handler.add_image("original.png", create_large_test_image()).unwrap();
    handler.write_to_file(&cbz_path).unwrap();

    let mut reader = CbzHandler::new();
    reader.read_from_file(&cbz_path).unwrap();

    // Optimize without resizing
    let options = OptimizationOptions::default().no_resize().with_quality(85);
    let result = reader.optimize_images(options);
    assert!(result.is_ok());
}

#[test]
fn test_optimization_preserves_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let cbz_path = temp_dir.path().join("preserve_meta.cbz");

    let mut handler = CbzHandler::new();
    let mut metadata = Metadata::new();
    metadata.title = Some("Metadata Preservation".to_string());
    metadata.author = Some("Test Author".to_string());
    
    handler.set_metadata(metadata).unwrap();
    handler.add_image("page.png", create_large_test_image()).unwrap();
    handler.write_to_file(&cbz_path).unwrap();

    let mut reader = CbzHandler::new();
    reader.read_from_file(&cbz_path).unwrap();

    // Optimize
    let options = OptimizationOptions::default();
    reader.optimize_images(options).unwrap();

    // Verify metadata is still intact
    let read_metadata = reader.get_metadata().unwrap();
    assert_eq!(read_metadata.title, Some("Metadata Preservation".to_string()));
    assert_eq!(read_metadata.author, Some("Test Author".to_string()));
}

#[test]
fn test_optimization_with_write() {
    let temp_dir = TempDir::new().unwrap();
    let original_path = temp_dir.path().join("original.cbz");
    let optimized_path = temp_dir.path().join("optimized.cbz");

    // Create original
    let mut handler = CbzHandler::new();
    handler.set_metadata(Metadata::new().with_title("Write Test")).unwrap();
    handler.add_image("page1.png", create_large_test_image()).unwrap();
    handler.add_image("page2.png", create_large_test_image()).unwrap();
    handler.write_to_file(&original_path).unwrap();

    // Read, optimize, and write
    let mut reader = CbzHandler::new();
    reader.read_from_file(&original_path).unwrap();
    
    let options = OptimizationOptions::default().with_quality(75);
    reader.optimize_images(options).unwrap();
    reader.write_to_file(&optimized_path).unwrap();

    // Verify optimized file is valid
    let mut final_reader = CbzHandler::new();
    final_reader.read_from_file(&optimized_path).unwrap();
    
    let images = final_reader.extract_images().unwrap();
    assert_eq!(images.len(), 2);
}
