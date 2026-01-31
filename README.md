# ebook-cli

A comprehensive Rust tool for reading, writing, and operating on various ebook formats. Available as a **CLI**, **MCP Server**, and **Library**.

## Supported Formats

- **EPUB** - Electronic Publication format
- **MOBI** - Mobipocket format
- **KF8 (AZW3)** - Kindle Format 8
- **FB2** - FictionBook 2.0
- **CBZ** - Comic Book Archive (ZIP)
- **TXT** - Plain text files
- **PDF** - Portable Document Format

## Features

- ✅ Read ebook metadata and content
- ✅ Write/create ebooks in various formats
- ✅ Extract images from ebooks
- ✅ Display table of contents
- ✅ Validate ebook files
- ✅ Repair corrupted ebooks
- ✅ Convert between formats (limited)
- ✅ **MCP Server** for AI assistant integration
- ✅ **Library** for use in Rust projects

## Installation

```bash
cargo build --release
```

The binary will be available at `target/release/ebook`.

## Usage

### Read an ebook

```bash
# Display content
ebook read book.epub

# Show metadata only
ebook read book.epub --metadata

# Show table of contents
ebook read book.epub --toc

# Extract images to a directory
ebook read book.epub --extract-images ./images
```

### Write/Create an ebook

```bash
# Create a TXT ebook
ebook write output.txt --format txt --title "My Book" --author "John Doe" --content input.txt

# Create an EPUB
ebook write output.epub --format epub --title "My Novel" --author "Jane Smith" --content story.txt

# Create a PDF
ebook write output.pdf --format pdf --title "Document" --content text.txt
```

### Get ebook information

```bash
ebook info book.epub
```

### Validate an ebook

```bash
ebook validate book.epub
```

### Repair an ebook

```bash
# Repair in place
ebook repair book.epub

# Repair and save to new file
ebook repair book.epub --output book_fixed.epub
```

### Convert formats

```bash
# Convert TXT to Markdown
ebook convert input.txt output.md
```

### MCP Server (Model Context Protocol)

Start the MCP server for AI assistant integration:

```bash
# Start MCP server
ebook mcp
```

The MCP server provides tools for AI assistants to:
- Read ebook content and metadata
- Create new ebooks
- Extract images
- Validate ebook files
- Get ebook information

See [docs/MCP.md](docs/MCP.md) for detailed MCP integration guide.

**Quick Setup for Claude Desktop:**

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:
```json
{
  "mcpServers": {
    "ebook": {
      "command": "/path/to/ebook-cli/target/release/ebook",
      "args": ["mcp"]
    }
  }
}
```

## Library Usage

You can also use ebook-cli as a library in your Rust projects:

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

## Architecture

The project follows a trait-based architecture:

- **`EbookReader`** - Trait for reading ebooks
- **`EbookWriter`** - Trait for writing ebooks
- **`EbookOperator`** - Trait for advanced operations (convert, validate, repair)

Each format has its own handler that implements these traits:
- `EpubHandler`
- `MobiHandler`
- `Fb2Handler`
- `CbzHandler`
- `TxtHandler`
- `PdfHandler`

## Development

### Build

```bash
cargo build
```

### Run tests

```bash
cargo test
```

### Run with example

```bash
cargo run -- read examples/sample.txt
```

## Dependencies

- `clap` - CLI argument parsing
- `zip` - ZIP archive handling (EPUB, CBZ)
- `quick-xml` - XML parsing (EPUB, FB2)
- `lopdf` - PDF manipulation
- `encoding_rs` - Character encoding detection
- `serde` - Serialization/deserialization
