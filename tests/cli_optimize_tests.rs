use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::fs;

fn create_test_cbz(path: &std::path::Path) {
    use ebook_cli::formats::CbzHandler;
    use ebook_cli::traits::EbookWriter;
    use ebook_cli::Metadata;
    
    let mut handler = CbzHandler::new();
    handler.set_metadata(Metadata::new().with_title("Test Comic")).unwrap();
    
    // Create a simple test image
    let test_image = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
        0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
        0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41,
        0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
        0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00,
        0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
        0x42, 0x60, 0x82,
    ];
    
    handler.add_image("page01.png", test_image.clone()).unwrap();
    handler.add_image("page02.png", test_image).unwrap();
    handler.write_to_file(path).unwrap();
}

fn create_test_epub(path: &std::path::Path) {
    use ebook_cli::formats::EpubHandler;
    use ebook_cli::traits::EbookWriter;
    use ebook_cli::Metadata;
    
    let mut handler = EpubHandler::new();
    handler.set_metadata(Metadata::new().with_title("Test Book")).unwrap();
    handler.add_chapter("Chapter 1", "<h1>Chapter 1</h1><p>Content</p>").unwrap();
    
    let test_image = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
        0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
        0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41,
        0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
        0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00,
        0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
        0x42, 0x60, 0x82,
    ];
    
    handler.add_image("cover.png", test_image).unwrap();
    handler.write_to_file(path).unwrap();
}

#[test]
fn test_cli_optimize_cbz() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.cbz");
    let output_path = temp_dir.path().join("optimized.cbz");
    
    create_test_cbz(&input_path);
    
    let mut cmd = Command::cargo_bin("ebook").unwrap();
    cmd.arg("optimize")
        .arg(&input_path)
        .arg("--output")
        .arg(&output_path)
        .arg("--quality")
        .arg("80");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Successfully optimized CBZ"));
    
    assert!(output_path.exists());
}

#[test]
fn test_cli_optimize_epub() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.epub");
    let output_path = temp_dir.path().join("optimized.epub");
    
    create_test_epub(&input_path);
    
    let mut cmd = Command::cargo_bin("ebook").unwrap();
    cmd.arg("optimize")
        .arg(&input_path)
        .arg("--output")
        .arg(&output_path);
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Successfully optimized EPUB"));
    
    assert!(output_path.exists());
}

#[test]
fn test_cli_optimize_with_progress() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.cbz");
    
    create_test_cbz(&input_path);
    
    let mut cmd = Command::cargo_bin("ebook").unwrap();
    cmd.arg("optimize")
        .arg(&input_path)
        .arg("--progress");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Successfully optimized"));
}

#[test]
fn test_cli_optimize_custom_dimensions() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.cbz");
    let output_path = temp_dir.path().join("optimized.cbz");
    
    create_test_cbz(&input_path);
    
    let mut cmd = Command::cargo_bin("ebook").unwrap();
    cmd.arg("optimize")
        .arg(&input_path)
        .arg("--output")
        .arg(&output_path)
        .arg("--max-width")
        .arg("1024")
        .arg("--max-height")
        .arg("1024");
    
    cmd.assert()
        .success();
    
    assert!(output_path.exists());
}

#[test]
fn test_cli_optimize_no_resize() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.cbz");
    
    create_test_cbz(&input_path);
    
    let mut cmd = Command::cargo_bin("ebook").unwrap();
    cmd.arg("optimize")
        .arg(&input_path)
        .arg("--no-resize")
        .arg("--quality")
        .arg("90");
    
    cmd.assert()
        .success();
}

#[test]
fn test_cli_optimize_unsupported_format() {
    let temp_dir = TempDir::new().unwrap();
    let txt_path = temp_dir.path().join("test.txt");
    
    fs::write(&txt_path, "Test content").unwrap();
    
    let mut cmd = Command::cargo_bin("ebook").unwrap();
    cmd.arg("optimize")
        .arg(&txt_path);
    
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Image optimization only supports"));
}

#[test]
fn test_cli_optimize_help() {
    let mut cmd = Command::cargo_bin("ebook").unwrap();
    cmd.arg("optimize")
        .arg("--help");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("optimize"))
        .stdout(predicate::str::contains("max-width"))
        .stdout(predicate::str::contains("quality"));
}
