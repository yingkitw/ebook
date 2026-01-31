# Specification

## Project: ebook-cli

### Overview
A command-line tool and library for reading, writing, and manipulating ebook files in multiple formats.

### Supported Formats

| Format | Extension | Read | Write | Metadata | Images | TOC | Convert |
|--------|-----------|------|-------|----------|--------|-----|---------|
| EPUB   | .epub     | ✅   | ✅    | ✅       | ✅     | ✅  | 🚧      |
| MOBI   | .mobi     | ✅   | ✅    | ⚠️       | ❌     | ❌  | ❌      |
| KF8    | .azw3     | ✅   | ✅    | ⚠️       | ❌     | ❌  | ❌      |
| FB2    | .fb2      | ✅   | ✅    | ✅       | ❌     | ❌  | ❌      |
| CBZ    | .cbz      | ✅   | ✅    | ⚠️       | ✅     | ❌  | ❌      |
| TXT    | .txt      | ✅   | ✅    | ✅       | ❌     | ✅  | ✅      |
| PDF    | .pdf      | ✅   | ✅    | ✅       | ❌     | ⚠️  | ❌      |

Legend:
- ✅ Fully supported
- ⚠️ Partially supported
- 🚧 In development
- ❌ Not supported

### CLI Commands

#### `read`
Read and display ebook content or metadata.

**Usage:**
```bash
ebook read <INPUT> [OPTIONS]
```

**Options:**
- `-m, --metadata` - Show metadata only
- `-e, --extract-images <DIR>` - Extract images to directory
- `-t, --toc` - Show table of contents

**Examples:**
```bash
ebook read book.epub
ebook read book.epub --metadata
ebook read book.epub --extract-images ./images
ebook read book.txt --toc
```

#### `write`
Create a new ebook file.

**Usage:**
```bash
ebook write <OUTPUT> [OPTIONS]
```

**Options:**
- `-f, --format <FORMAT>` - Output format (required)
- `-t, --title <TITLE>` - Book title
- `-a, --author <AUTHOR>` - Book author
- `-c, --content <FILE>` - Content file path

**Examples:**
```bash
ebook write output.epub --format epub --title "My Book" --content story.txt
ebook write output.pdf --format pdf --title "Document" --author "John Doe"
```

#### `convert`
Convert between ebook formats.

**Usage:**
```bash
ebook convert <INPUT> <OUTPUT> [OPTIONS]
```

**Options:**
- `-f, --format <FORMAT>` - Target format (optional, auto-detected from output extension)

**Examples:**
```bash
ebook convert book.txt book.md
ebook convert story.txt story.epub --format epub
```

#### `info`
Display detailed information about an ebook.

**Usage:**
```bash
ebook info <INPUT>
```

**Examples:**
```bash
ebook info book.epub
```

#### `validate`
Validate ebook file structure and metadata.

**Usage:**
```bash
ebook validate <INPUT>
```

**Examples:**
```bash
ebook validate book.epub
```

#### `repair`
Repair corrupted or invalid ebook files.

**Usage:**
```bash
ebook repair <INPUT> [OPTIONS]
```

**Options:**
- `-o, --output <FILE>` - Output file (optional, defaults to input file)

**Examples:**
```bash
ebook repair broken.epub
ebook repair broken.epub --output fixed.epub
```

### Library API

#### Core Traits

**`EbookReader`**
```rust
pub trait EbookReader {
    fn read_from_file(&mut self, path: &Path) -> Result<()>;
    fn get_metadata(&self) -> Result<Metadata>;
    fn get_content(&self) -> Result<String>;
    fn get_toc(&self) -> Result<Vec<TocEntry>>;
    fn extract_images(&self) -> Result<Vec<ImageData>>;
}
```

**`EbookWriter`**
```rust
pub trait EbookWriter {
    fn set_metadata(&mut self, metadata: Metadata) -> Result<()>;
    fn set_content(&mut self, content: &str) -> Result<()>;
    fn add_chapter(&mut self, title: &str, content: &str) -> Result<()>;
    fn add_image(&mut self, name: &str, data: Vec<u8>) -> Result<()>;
    fn write_to_file(&self, path: &Path) -> Result<()>;
}
```

**`EbookOperator`**
```rust
pub trait EbookOperator: EbookReader + EbookWriter {
    fn convert_to(&self, target_format: &str, output_path: &Path) -> Result<()>;
    fn validate(&self) -> Result<bool>;
    fn repair(&mut self) -> Result<()>;
}
```

#### Data Structures

**`Metadata`**
```rust
pub struct Metadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub publisher: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub isbn: Option<String>,
    pub publication_date: Option<String>,
    pub cover_image: Option<Vec<u8>>,
    pub format: Option<String>,
    pub custom_fields: HashMap<String, String>,
}
```

**`TocEntry`**
```rust
pub struct TocEntry {
    pub title: String,
    pub level: usize,
    pub href: Option<String>,
    pub children: Vec<TocEntry>,
}
```

**`ImageData`**
```rust
pub struct ImageData {
    pub name: String,
    pub mime_type: String,
    pub data: Vec<u8>,
}
```

### Error Handling

All operations return `Result<T, EbookError>` where `EbookError` includes:
- `Io(std::io::Error)` - File I/O errors
- `Zip(zip::result::ZipError)` - ZIP archive errors
- `Xml(String)` - XML parsing errors
- `Pdf(lopdf::Error)` - PDF errors
- `UnsupportedFormat(String)` - Unknown format
- `InvalidMetadata(String)` - Invalid metadata
- `Parse(String)` - Parsing errors
- `Encoding(String)` - Encoding errors
- `NotFound(String)` - Resource not found
- `InvalidStructure(String)` - Invalid file structure
- `NotSupported(String)` - Unsupported operation

### Technical Requirements

#### Rust Edition
- Rust 2024 edition

#### Dependencies
- `clap` 4.5+ - CLI parsing
- `zip` 2.2+ - ZIP archive handling
- `quick-xml` 0.36+ - XML parsing
- `lopdf` 0.34+ - PDF manipulation
- `encoding_rs` 0.8+ - Character encoding
- `serde` 1.0+ - Serialization
- `thiserror` 1.0+ - Error handling

#### Build Requirements
- `cargo build` must succeed without errors
- `cargo test` must pass all tests
- No warnings in release builds (except dead code for future features)

### Performance Targets

- Read operations: < 1s for files up to 10MB
- Write operations: < 2s for files up to 10MB
- Memory usage: < 100MB for typical operations
- Streaming support for files > 100MB (future)

### Security Considerations

- No arbitrary code execution from ebook files
- Validate all file paths to prevent directory traversal
- Limit file sizes to prevent DoS
- Sanitize all user inputs
- No hardcoded credentials or API keys

### Compatibility

- macOS (primary)
- Linux (tested)
- Windows (should work, not tested)

### Exit Codes

- `0` - Success
- `1` - General error
- `101` - Compilation error (cargo)
