# Architecture

## Overview

The ebook-cli project follows a modular, trait-based architecture that separates concerns and enables easy extension for new ebook formats.

## Core Components

### 1. Traits (`src/traits.rs`)

The foundation of the architecture consists of three main traits:

#### `EbookReader`
Provides read operations for ebook formats:
- `read_from_file()` - Load ebook from file
- `get_metadata()` - Extract metadata
- `get_content()` - Extract text content
- `get_toc()` - Get table of contents
- `extract_images()` - Extract embedded images

#### `EbookWriter`
Provides write operations for ebook formats:
- `set_metadata()` - Set ebook metadata
- `set_content()` - Set text content
- `add_chapter()` - Add a chapter
- `add_image()` - Add an image
- `write_to_file()` - Save ebook to file

#### `EbookOperator`
Advanced operations (combines Reader + Writer):
- `convert_to()` - Convert to another format
- `validate()` - Validate ebook structure
- `repair()` - Repair corrupted ebooks

### 2. Format Handlers (`src/formats/`)

Each ebook format has a dedicated handler that implements the core traits:

- **`EpubHandler`** - EPUB (ZIP-based, XML metadata)
- **`MobiHandler`** - MOBI/KF8 (binary format)
- **`Fb2Handler`** - FictionBook 2.0 (XML-based)
- **`CbzHandler`** - Comic Book Archive (ZIP with images)
- **`TxtHandler`** - Plain text files
- **`PdfHandler`** - PDF documents

### 3. Metadata (`src/metadata.rs`)

Unified metadata structure across all formats:
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

### 4. Error Handling (`src/error.rs`)

Centralized error types using `thiserror`:
- `Io` - File I/O errors
- `Zip` - ZIP archive errors
- `Xml` - XML parsing errors
- `Pdf` - PDF-specific errors
- `UnsupportedFormat` - Unknown format
- `InvalidMetadata` - Metadata validation errors
- `Parse` - General parsing errors
- `Encoding` - Character encoding errors
- `NotFound` - Resource not found
- `InvalidStructure` - Invalid file structure
- `NotSupported` - Unsupported operation

### 5. Utilities (`src/utils.rs`)

Helper functions:
- `detect_format()` - Auto-detect ebook format from file extension
- `sanitize_filename()` - Clean filenames for safe file operations
- `guess_mime_type()` - Determine MIME type from filename

## Design Principles

### 1. Separation of Concerns
Each format handler is independent and self-contained. Changes to one format don't affect others.

### 2. Trait-Based Design
All handlers implement common traits, enabling:
- Polymorphic usage
- Easy testing with mock implementations
- Consistent API across formats

### 3. DRY (Don't Repeat Yourself)
Common functionality is extracted to utilities and shared structures.

### 4. KISS (Keep It Simple, Stupid)
Each component has a single, well-defined responsibility.

### 5. Test-Friendly
Traits enable easy mocking and testing. Each handler can be tested independently.

## Data Flow

### Reading an Ebook
```
File → Handler.read_from_file() → Parse Format → Extract Metadata/Content → Return Data
```

### Writing an Ebook
```
Metadata + Content → Handler.set_*() → Format Data → Handler.write_to_file() → File
```

### Converting
```
Source File → Reader.read_from_file() → Extract Data → Writer.set_*() → Writer.write_to_file() → Target File
```

## Extension Points

### Adding a New Format

1. Create a new handler in `src/formats/`:
```rust
pub struct NewFormatHandler {
    metadata: Metadata,
    content: String,
}
```

2. Implement the core traits:
```rust
impl EbookReader for NewFormatHandler { ... }
impl EbookWriter for NewFormatHandler { ... }
impl EbookOperator for NewFormatHandler { ... }
```

3. Add to `src/formats/mod.rs`:
```rust
pub mod newformat;
pub use newformat::NewFormatHandler;
```

4. Update `src/utils.rs` to detect the format:
```rust
"newext" => Ok("newformat".to_string()),
```

5. Add CLI support in `src/main.rs`

6. Write tests in `tests/newformat_tests.rs`

## Dependencies

### Core
- `clap` - CLI parsing
- `anyhow` - Error handling convenience
- `thiserror` - Error type derivation
- `serde` - Serialization

### Format-Specific
- `zip` - EPUB, CBZ
- `quick-xml` - EPUB, FB2
- `lopdf` - PDF
- `encoding_rs` - Text encoding detection
- `pulldown-cmark` - Markdown support

### Testing
- `tempfile` - Temporary test files
- `assert_fs` - Filesystem assertions
- `predicates` - Test predicates

## Performance Considerations

- **Lazy Loading**: Content is only parsed when requested
- **Streaming**: Large files can be processed in chunks
- **Memory Efficiency**: Avoid loading entire files into memory when possible
- **Caching**: Parsed metadata is cached in handler structs

## Future Enhancements

- Async I/O support
- Streaming conversion for large files
- Plugin system for custom formats
- Advanced PDF text extraction
- EPUB 3.0 full support
- Metadata editing without full rewrite
