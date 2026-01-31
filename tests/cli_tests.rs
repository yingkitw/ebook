//! Integration tests for CLI commands

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

fn get_cli_executable() -> PathBuf {
    // The built binary should be at target/debug/ebook
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove the test executable name
    if path.ends_with("deps") {
        path.pop();
    }
    path.push("ebook");
    path
}

fn setup_test_dir(name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!("cli_tests_{}_{}", name, std::process::id()));
    let _ = fs::create_dir_all(&dir);
    dir
}

fn cleanup_test_dir(dir: &PathBuf) {
    let _ = fs::remove_dir_all(dir);
}

fn create_test_txt(path: &PathBuf) {
    let mut file = File::create(path).unwrap();
    writeln!(file, "Title: Test Book").unwrap();
    writeln!(file, "Author: Test Author").unwrap();
    writeln!(file).unwrap();
    writeln!(file, "This is test content for CLI testing.").unwrap();
    file.sync_all().unwrap();
}

#[test]
fn test_cli_read_command() {
    let test_dir = setup_test_dir("read");
    let txt_path = test_dir.join("test.txt");
    create_test_txt(&txt_path);

    let cli = get_cli_executable();
    let output = Command::new(&cli)
        .arg("read")
        .arg(&txt_path)
        .output()
        .unwrap();

    assert!(output.status.success(), "CLI read command should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test content") || stdout.contains("Test content"), "Output should contain file content");

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_cli_read_metadata() {
    let test_dir = setup_test_dir("metadata");
    let txt_path = test_dir.join("test.txt");
    create_test_txt(&txt_path);

    let cli = get_cli_executable();
    let output = Command::new(&cli)
        .arg("read")
        .arg("--metadata")
        .arg(&txt_path)
        .output()
        .unwrap();

    assert!(output.status.success(), "CLI read --metadata should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("title"), "Output should contain metadata");

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_cli_info_command() {
    let test_dir = setup_test_dir("info");
    let txt_path = test_dir.join("test.txt");
    create_test_txt(&txt_path);

    let cli = get_cli_executable();
    let output = Command::new(&cli)
        .arg("info")
        .arg(&txt_path)
        .output()
        .unwrap();

    assert!(output.status.success(), "CLI info command should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Format:"), "Output should contain format info");

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_cli_validate_command() {
    let test_dir = setup_test_dir("validate");
    let txt_path = test_dir.join("test.txt");
    create_test_txt(&txt_path);

    let cli = get_cli_executable();
    let output = Command::new(&cli)
        .arg("validate")
        .arg(&txt_path)
        .output()
        .unwrap();

    assert!(output.status.success(), "CLI validate command should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("valid"), "Output should indicate validity");

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_cli_write_txt_command() {
    let test_dir = setup_test_dir("write_txt");
    let output_path = test_dir.join("output.txt");

    let cli = get_cli_executable();
    let output = Command::new(&cli)
        .arg("write")
        .arg("--format")
        .arg("txt")
        .arg("--title")
        .arg("Test Write")
        .arg("--author")
        .arg("Test Author")
        .arg(&output_path)
        .output()
        .unwrap();

    assert!(output.status.success(), "CLI write command should succeed: {:?}", String::from_utf8_lossy(&output.stderr));
    assert!(output_path.exists(), "Output file should be created");

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_cli_convert_txt_to_epub() {
    let test_dir = setup_test_dir("convert");
    let txt_path = test_dir.join("test.txt");
    let epub_path = test_dir.join("output.epub");
    create_test_txt(&txt_path);

    let cli = get_cli_executable();
    let output = Command::new(&cli)
        .arg("convert")
        .arg(&txt_path)
        .arg(&epub_path)
        .arg("--format")
        .arg("epub")
        .output()
        .unwrap();

    assert!(output.status.success(), "CLI convert command should succeed");
    assert!(epub_path.exists(), "EPUB file should be created");

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_cli_convert_with_progress() {
    let test_dir = setup_test_dir("progress");
    let txt_path = test_dir.join("test.txt");
    let epub_path = test_dir.join("output.epub");
    create_test_txt(&txt_path);

    let cli = get_cli_executable();
    let output = Command::new(&cli)
        .arg("convert")
        .arg(&txt_path)
        .arg(&epub_path)
        .arg("--format")
        .arg("epub")
        .arg("--progress")
        .output()
        .unwrap();

    assert!(output.status.success(), "CLI convert with progress should succeed");
    assert!(epub_path.exists(), "EPUB file should be created");

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_cli_repair_command() {
    let test_dir = setup_test_dir("repair");
    let txt_path = test_dir.join("test.txt");
    create_test_txt(&txt_path);

    let cli = get_cli_executable();
    let output = Command::new(&cli)
        .arg("repair")
        .arg(&txt_path)
        .output()
        .unwrap();

    assert!(output.status.success(), "CLI repair command should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Successfully repaired"), "Output should confirm repair");

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_cli_convert_auto_format_detection() {
    let test_dir = setup_test_dir("auto_detect");
    let txt_path = test_dir.join("test.txt");
    let epub_path = test_dir.join("output.epub");
    create_test_txt(&txt_path);

    let cli = get_cli_executable();
    let output = Command::new(&cli)
        .arg("convert")
        .arg(&txt_path)
        .arg(&epub_path)
        // Don't specify format - should auto-detect from extension
        .output()
        .unwrap();

    assert!(output.status.success(), "CLI convert without explicit format should succeed");
    assert!(epub_path.exists(), "EPUB file should be created");

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_cli_help_command() {
    let cli = get_cli_executable();
    let output = Command::new(&cli)
        .arg("--help")
        .output()
        .unwrap();

    assert!(output.status.success(), "CLI --help should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ebook"), "Help should mention the tool name");
    assert!(stdout.contains("Commands:"), "Help should list commands");
}

#[test]
fn test_cli_invalid_file() {
    let test_dir = setup_test_dir("invalid");
    let nonexistent_path = test_dir.join("does_not_exist.txt");

    let cli = get_cli_executable();
    let output = Command::new(&cli)
        .arg("info")
        .arg(&nonexistent_path)
        .output()
        .unwrap();

    assert!(!output.status.success(), "CLI should fail on nonexistent file");

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_cli_convert_unsupported_format() {
    let test_dir = setup_test_dir("unsupported");
    let txt_path = test_dir.join("test.txt");
    let mobi_path = test_dir.join("output.mobi");
    create_test_txt(&txt_path);

    let cli = get_cli_executable();
    let output = Command::new(&cli)
        .arg("convert")
        .arg(&txt_path)
        .arg(&mobi_path)
        .arg("--format")
        .arg("mobi")
        .output()
        .unwrap();

    // TXT to MOBI conversion is supported, so this should succeed
    assert!(output.status.success(), "TXT to MOBI conversion should succeed");
    assert!(mobi_path.exists(), "MOBI file should be created");

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_cli_write_with_content_file() {
    let test_dir = setup_test_dir("write_content");
    let content_path = test_dir.join("content.txt");
    let output_path = test_dir.join("output.epub");

    // Create content file
    let mut file = File::create(&content_path).unwrap();
    writeln!(file, "Chapter 1 content goes here.").unwrap();
    file.sync_all().unwrap();

    let cli = get_cli_executable();
    let output = Command::new(&cli)
        .arg("write")
        .arg("--format")
        .arg("epub")
        .arg("--title")
        .arg("Content Test")
        .arg("--content")
        .arg(&content_path)
        .arg(&output_path)
        .output()
        .unwrap();

    assert!(output.status.success(), "CLI write with content file should succeed: {:?}", String::from_utf8_lossy(&output.stderr));
    assert!(output_path.exists(), "EPUB file should be created");

    cleanup_test_dir(&test_dir);
}
