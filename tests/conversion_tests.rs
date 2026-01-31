//! Integration tests for ebook format conversion

use ebook_cli::Converter;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn setup_test_dir() -> PathBuf {
    use std::time::SystemTime;
    let mut dir = std::env::temp_dir();
    // Use process ID + timestamp + random number for uniqueness
    let unique_id = format!("ebook_conversion_tests_{}_{}",
        std::process::id(),
        SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    );
    dir.push(unique_id);
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn cleanup_test_dir(dir: &PathBuf) {
    let _ = std::fs::remove_dir_all(dir);
}

fn create_test_txt(path: &PathBuf) {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let mut file = File::create(&path).unwrap();
    writeln!(file, "Title: Test Book").unwrap();
    writeln!(file, "Author: Test Author").unwrap();
    writeln!(file).unwrap();
    writeln!(file, "This is test content for conversion.").unwrap();
    file.sync_all().unwrap();
}

#[test]
fn test_txt_to_epub_conversion() {
    let test_dir = setup_test_dir();
    let txt_path = test_dir.join("test.txt");
    let epub_path = test_dir.join("test.epub");

    create_test_txt(&txt_path);

    let result = Converter::convert(&txt_path, &epub_path, "epub");
    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());
    
    // Verify file exists before cleanup
    let exists = epub_path.exists();
    cleanup_test_dir(&test_dir);
    assert!(exists, "Output EPUB file should exist");
}

#[test]
fn test_txt_to_pdf_conversion() {
    let test_dir = setup_test_dir();
    let txt_path = test_dir.join("test.txt");
    let pdf_path = test_dir.join("test.pdf");

    create_test_txt(&txt_path);

    let result = Converter::convert(&txt_path, &pdf_path, "pdf");
    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());
    let exists = pdf_path.exists();
    cleanup_test_dir(&test_dir);
    assert!(exists, "Output PDF file should exist");
}

#[test]
fn test_txt_to_mobi_conversion() {
    let test_dir = setup_test_dir();
    let txt_path = test_dir.join("test.txt");
    let mobi_path = test_dir.join("test.mobi");

    create_test_txt(&txt_path);

    let result = Converter::convert(&txt_path, &mobi_path, "mobi");
    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());
    assert!(mobi_path.exists(), "Output MOBI file should exist");

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_txt_to_fb2_conversion() {
    let test_dir = setup_test_dir();
    let txt_path = test_dir.join("test.txt");
    let fb2_path = test_dir.join("test.fb2");

    create_test_txt(&txt_path);

    let result = Converter::convert(&txt_path, &fb2_path, "fb2");
    assert!(result.is_ok(), "Conversion should succeed: {:?}", result.err());
    assert!(fb2_path.exists(), "Output FB2 file should exist");

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_epub_to_txt_conversion() {
    let test_dir = setup_test_dir();
    let txt_path = test_dir.join("source.txt");
    let epub_path = test_dir.join("test.epub");
    let out_txt_path = test_dir.join("output.txt");

    create_test_txt(&txt_path);

    // First convert TXT to EPUB
    let result1 = Converter::convert(&txt_path, &epub_path, "epub");
    assert!(result1.is_ok(), "TXT to EPUB should succeed: {:?}", result1.err());
    
    let epub_exists = epub_path.exists();
    assert!(epub_exists, "EPUB file should exist");

    // Then convert EPUB back to TXT
    let result = Converter::convert(&epub_path, &out_txt_path, "txt");
    assert!(result.is_ok(), "EPUB to TXT should succeed: {:?}", result.err());
    
    let txt_exists = out_txt_path.exists();
    cleanup_test_dir(&test_dir);
    assert!(txt_exists, "Output TXT file should exist");
}

#[test]
fn test_pdf_to_txt_conversion() {
    let test_dir = setup_test_dir();
    let txt_path = test_dir.join("source.txt");
    let pdf_path = test_dir.join("test.pdf");
    let out_txt_path = test_dir.join("output.txt");

    create_test_txt(&txt_path);

    // First convert TXT to PDF
    let result1 = Converter::convert(&txt_path, &pdf_path, "pdf");
    assert!(result1.is_ok(), "TXT to PDF should succeed: {:?}", result1.err());
    assert!(pdf_path.exists(), "PDF file should exist");

    // Then convert PDF back to TXT
    let result = Converter::convert(&pdf_path, &out_txt_path, "txt");
    assert!(result.is_ok(), "PDF to TXT should succeed: {:?}", result.err());
    assert!(out_txt_path.exists(), "Output TXT file should exist");

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_epub_to_pdf_conversion() {
    let test_dir = setup_test_dir();
    let txt_path = test_dir.join("source.txt");
    let epub_path = test_dir.join("test.epub");
    let pdf_path = test_dir.join("test.pdf");

    create_test_txt(&txt_path);

    // First convert TXT to EPUB
    let result1 = Converter::convert(&txt_path, &epub_path, "epub");
    assert!(result1.is_ok(), "TXT to EPUB should succeed: {:?}", result1.err());

    // Then convert EPUB to PDF
    let result = Converter::convert(&epub_path, &pdf_path, "pdf");
    assert!(result.is_ok(), "EPUB to PDF should succeed: {:?}", result.err());
    
    let exists = pdf_path.exists();
    cleanup_test_dir(&test_dir);
    assert!(exists, "Output PDF file should exist");
}

#[test]
fn test_mobi_to_txt_conversion() {
    let test_dir = setup_test_dir();
    let txt_path = test_dir.join("source.txt");
    let mobi_path = test_dir.join("test.mobi");
    let out_txt_path = test_dir.join("output.txt");

    create_test_txt(&txt_path);

    // First convert TXT to MOBI
    let result1 = Converter::convert(&txt_path, &mobi_path, "mobi");
    assert!(result1.is_ok(), "TXT to MOBI should succeed: {:?}", result1.err());
    assert!(mobi_path.exists(), "MOBI file should exist");

    // Then convert MOBI back to TXT
    let result = Converter::convert(&mobi_path, &out_txt_path, "txt");
    assert!(result.is_ok(), "MOBI to TXT should succeed: {:?}", result.err());
    assert!(out_txt_path.exists(), "Output TXT file should exist");

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_fb2_to_txt_conversion() {
    let test_dir = setup_test_dir();
    let txt_path = test_dir.join("source.txt");
    let fb2_path = test_dir.join("test.fb2");
    let out_txt_path = test_dir.join("output.txt");

    create_test_txt(&txt_path);

    // First convert TXT to FB2
    let result1 = Converter::convert(&txt_path, &fb2_path, "fb2");
    assert!(result1.is_ok(), "TXT to FB2 should succeed: {:?}", result1.err());
    assert!(fb2_path.exists(), "FB2 file should exist");

    // Then convert FB2 back to TXT
    let result = Converter::convert(&fb2_path, &out_txt_path, "txt");
    assert!(result.is_ok(), "FB2 to TXT should succeed: {:?}", result.err());
    assert!(out_txt_path.exists(), "Output TXT file should exist");

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_conversion_with_progress() {
    let test_dir = setup_test_dir();
    let txt_path = test_dir.join("test.txt");
    let epub_path = test_dir.join("test.epub");

    create_test_txt(&txt_path);

    let result = Converter::convert_with_progress(
        &txt_path,
        &epub_path,
        "epub",
        Some("Test Conversion".to_string()),
    );
    assert!(result.is_ok(), "Conversion with progress should succeed: {:?}", result.err());
    
    let exists = epub_path.exists();
    cleanup_test_dir(&test_dir);
    assert!(exists, "Output EPUB file should exist");
}

#[test]
fn test_unsupported_conversion() {
    let test_dir = setup_test_dir();
    let txt_path = test_dir.join("test.txt");
    let epub_path = test_dir.join("test.epub");
    let mobi_path = test_dir.join("test.mobi");

    create_test_txt(&txt_path);

    // Convert TXT to EPUB first
    Converter::convert(&txt_path, &epub_path, "epub").unwrap();

    // Try to convert EPUB to MOBI (not supported)
    let result = Converter::convert(&epub_path, &mobi_path, "mobi");
    assert!(result.is_err(), "EPUB to MOBI conversion should not be supported");

    cleanup_test_dir(&test_dir);
}
