# ebook-cli

A comprehensive Rust tool for reading, writing, and operating on various ebook formats. Available as a **CLI**, **MCP Server**, and **Library**.

## Why ebook-cli?

Ebook files come in many formats (EPUB, MOBI, PDF, CBZ, etc.), and each format has its own quirks and internal structure. Working with ebooks programmatically often requires:

- **Format detection** - Automatically identifying what type of ebook you're working with
- **Metadata extraction** - Getting title, author, and other information reliably
- **Content access** - Reading text, chapters, and images from different formats
- **Conversion** - Transforming between formats for different devices
- **Validation** - Checking if ebook files are well-formed
- **AI integration** - Allowing AI assistants to work with ebook files

ebook-cli solves these problems with a unified interface across all supported formats, whether you're using it from the command line, as an MCP server with AI assistants, or as a Rust library in your own projects.

## Supported Formats

- **EPUB** (2.0 & 3.0) - Electronic Publication format
- **MOBI** - Mobipocket format
- **AZW** - Kindle format with DRM detection
- **AZW3 (KF8)** - Kindle Format 8
- **FB2** - FictionBook 2.0
- **CBZ** - Comic Book Archive with ComicInfo.xml support
- **TXT** - Plain text files with encoding detection
- **PDF** - Portable Document Format

## Features

### Core Operations
- ✅ Read ebook metadata, content, and table of contents
- ✅ Write/create ebooks in all supported formats
- ✅ Extract images from ebooks (EPUB, CBZ, PDF)
- ✅ Validate ebook file structure and integrity
- ✅ Repair corrupted ebook files
- ✅ Convert between formats (TXT ↔ EPUB, TXT ↔ PDF, TXT ↔ MOBI, EPUB → PDF, etc.)

### Advanced Features
- ✅ **Image optimization** - Resize and compress images in EPUB/CBZ files
- ✅ **Streaming support** - Handle large files efficiently (10MB+ TXT, 50MB+ EPUB)
- ✅ **Progress indicators** - Visual feedback for long operations
- ✅ **Encoding detection** - Automatic character encoding detection for TXT files
- ✅ **Format auto-detection** - Works based on file extension

### Integration
- ✅ **MCP Server** - AI assistant integration via Model Context Protocol
- ✅ **Library API** - Use as a Rust library in your projects
- ✅ **CLI** - Full-featured command-line interface

## Installation

### From source

```bash
git clone https://github.com/yingkitw/ebook.git
cd ebook
cargo build --release
```

The binary will be available at `target/release/ebook`.

### As a library

Add to your `Cargo.toml`:

```toml
[dependencies]
ebook = "0.1.0"
```

## Usage

### CLI Examples

#### Read an ebook

```bash
# Display full content
ebook read book.epub

# Show metadata only (title, author, etc.)
ebook read book.epub --metadata

# Show table of contents
ebook read book.epub --toc

# Extract images to a directory
ebook read book.epub --extract-images ./images

# Read specific format (auto-detected by extension)
ebook read comic.cbz
ebook read novel.mobi
ebook read document.pdf
```

#### Write/Create an ebook

```bash
# Create from a text file
ebook write output.txt --format txt --title "My Book" --author "John Doe" --content input.txt

# Create an EPUB with all metadata
ebook write output.epub --format epub \
  --title "My Novel" \
  --author "Jane Smith" \
  --publisher "My Press" \
  --isbn "978-0-1234567-8-9" \
  --content story.txt

# Create a PDF
ebook write output.pdf --format pdf --title "Document" --content text.txt

# Create a CBZ comic archive
ebook write comic.cbz --format cbz --title "Super Comic" --content pages/
```

#### Get ebook information

```bash
# Quick info display
ebook info book.epub

# Output example:
# Format: EPUB
# Title: The Great Book
# Author: John Doe
# Size: 1.2 MB
# Valid: Yes
```

#### Validate an ebook

```bash
# Validate file structure
ebook validate book.epub

# Returns detailed validation results
ebook validate --verbose book.epub
```

#### Repair an ebook

```bash
# Repair in place (creates backup)
ebook repair book.epub

# Repair and save to new file
ebook repair book.epub --output book_fixed.epub
```

#### Convert between formats

```bash
# TXT to EPUB (for e-readers)
ebook convert novel.txt novel.epub

# EPUB to PDF (for printing/sharing)
ebook convert book.epub book.pdf

# MOBI to TXT (extract text)
ebook convert kindle.mobi article.txt

# FB2 to EPUB
ebook convert book.fb2 book.epub
```

#### Optimize images in ebooks

```bash
# Optimize all images in an EPUB (reduces file size)
ebook optimize book.epub

# Custom dimensions and quality
ebook optimize comic.cbz --max-width 1200 --max-height 1600 --quality 80

# Optimize without resizing (compression only)
ebook optimize photo-album.epub --no-resize --quality 75
```

### MCP Server (Model Context Protocol)

The MCP server allows AI assistants (like Claude Desktop) to work with ebook files directly.

#### Starting the server

```bash
ebook mcp
```

#### Available MCP tools

| Tool | Description |
|------|-------------|
| `read_ebook` | Read content, metadata, and table of contents |
| `write_ebook` | Create new ebooks in any supported format |
| `extract_images` | Extract images from ebooks |
| `validate_ebook` | Validate ebook file structure |
| `get_ebook_info` | Get detailed ebook information |
| `convert_ebook` | Convert between formats |
| `optimize_images` | Optimize images in EPUB/CBZ files |

#### Quick Setup for Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "ebook": {
      "command": "/path/to/ebook/target/release/ebook",
      "args": ["mcp"]
    }
  }
}
```

#### Example AI workflows

**Summarize a book:**
```
User: Read the ebook at ~/Documents/book.epub and summarize chapter 1
Claude: [Uses read_ebook tool, analyzes content, provides summary]
```

**Convert a document:**
```
User: Convert ~/Downloads/novel.txt to EPUB format
Claude: [Uses convert_ebook tool, creates novel.epub]
```

**Extract images:**
```
User: Extract all images from the comic book at ~/comics/issue1.cbz
Claude: [Uses extract_images tool, returns images with metadata]
```

See [docs/MCP.md](docs/MCP.md) for detailed MCP documentation.

## Library Usage

Use ebook-cli as a Rust library in your projects:

### Basic example

```rust
use ebook_cli::formats::TxtHandler;
use ebook_cli::traits::{EbookReader, EbookWriter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read a text file
    let mut handler = TxtHandler::new();
    handler.read_from_file("book.txt".as_ref())?;

    // Get content
    let content = handler.get_content()?;
    println!("{}", content);

    // Get metadata
    let metadata = handler.get_metadata()?;
    println!("Title: {:?}", metadata.title);

    Ok(())
}
```

### Working with different formats

```rust
use ebook_cli::formats::{EpubHandler, MobiHandler, PdfHandler};
use ebook_cli::traits::EbookReader;

// Read EPUB
let mut epub = EpubHandler::new();
epub.read_from_file("book.epub".as_ref())?;
let toc = epub.get_toc()?;
println!("Table of Contents: {:?}", toc);

// Read MOBI
let mut mobi = MobiHandler::new();
mobi.read_from_file("kindle.mobi".as_ref())?;
let metadata = mobi.get_metadata()?;

// Read PDF
let mut pdf = PdfHandler::new();
pdf.read_from_file("document.pdf".as_ref())?;
let content = pdf.get_content()?;
```

### Format detection

```rust
use ebook_cli::utils::detect_format;
use ebook_cli::formats::EbookFormat;

let format = detect_format("book.epub")?;
assert_eq!(format, EbookFormat::Epub);
```

### Conversion

```rust
use ebook_cli::conversion::convert;

// Convert TXT to EPUB
convert(
    "input.txt".as_ref(),
    "output.epub".as_ref(),
    None  // auto-detect formats
)?;
```

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed library documentation.

## Architecture

The project follows a trait-based architecture for consistent API across all formats:

### Core Traits

- **`EbookReader`** - Read operations: content, metadata, table of contents, images
- **`EbookWriter`** - Write operations: create ebooks with content and metadata
- **`EbookOperator`** - Advanced operations: convert, validate, repair

### Format Handlers

Each format has a dedicated handler implementing all applicable traits:

| Handler | Read | Write | Metadata | TOC | Convert | Images |
|---------|------|-------|----------|-----|---------|--------|
| `EpubHandler` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `MobiHandler` | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| `AzwHandler` | ✅ | ❌ | ✅ | ✅ | ❌ | ❌ |
| `Fb2Handler` | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| `CbzHandler` | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ |
| `TxtHandler` | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ |
| `PdfHandler` | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ |

### Key Features

- **Streaming** - Large files are processed in chunks (10MB+ TXT, 50MB+ EPUB)
- **Progress bars** - Visual feedback for long-running operations
- **Error recovery** - Helpful error messages with suggestions
- **Thread-safe** - Safe for concurrent use

## Project Status

**Version:** 0.1.0

**License:** Apache-2.0

**Test Status:** ✅ All 103 tests passing

**Supported Platforms:** macOS, Linux, Windows (Rust-supported platforms)

**Recent Updates:**
- MCP server with full tool support (read, write, validate, info, convert, optimize)
- AZW format support with DRM detection
- Image optimization for EPUB/CBZ files
- EPUB 3.0 support (nav.xhtml, semantic markup, version switching)
- Streaming for large file handling (10MB+ TXT, 50MB+ EPUB thresholds)
- Comprehensive format conversion with CLI and MCP integration
- Progress indicators for long operations
- 103 comprehensive tests with full coverage

**Planned Features:**
- DJVU and CHM format support
- OCR for scanned PDFs
- Enhanced metadata editing
- Web service API
- Batch processing

See [TODO.md](TODO.md) for complete roadmap and known issues.

## Documentation

- [ARCHITECTURE.md](ARCHITECTURE.md) - Detailed architecture documentation
- [SPEC.md](SPEC.md) - Original specification document
- [docs/MCP.md](docs/MCP.md) - MCP server integration guide
- [TODO.md](TODO.md) - Development roadmap and known issues

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

Licensed under the Apache License, Version 2.0 ([LICENSE](https://www.apache.org/licenses/LICENSE-2.0) or http://www.apache.org/licenses/LICENSE-2.0)

## Development

### Build

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release
```

### Run tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_epub_read

# Run with output
cargo test -- --nocapture

# Run tests in parallel
cargo test -- --test-threads=4
```

**Test Coverage:** 103 tests covering:
- Format handlers (EPUB, MOBI, AZW, FB2, CBZ, TXT, PDF)
- CLI integration tests
- MCP integration tests
- Conversion tests
- Streaming tests
- Image optimization tests
- EPUB 3.0 features
- Error handling

### Run benchmarks

```bash
# Performance benchmarks (requires criterion)
cargo bench
```

Benchmarks available for:
- EPUB read/write performance
- CBZ read/write performance
- Image optimization performance

### Example files

```bash
# Run with example file
cargo run -- read examples/sample.txt

# Create an EPUB
cargo run -- write output.epub --format epub --title "Test" --content examples/sample.txt
```

### Enable logging

```bash
# Info level
RUST_LOG=info cargo run -- read book.epub

# Debug level (verbose)
RUST_LOG=debug cargo run -- read book.epub

# Trace level (very verbose)
RUST_LOG=trace cargo run -- read book.epub
```
