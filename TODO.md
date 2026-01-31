# TODO

## Completed ✅

- [x] Project structure setup
- [x] Core traits implementation (EbookReader, EbookWriter, EbookOperator)
- [x] Metadata structure
- [x] Error handling with thiserror
- [x] EPUB format handler
- [x] MOBI/KF8 format handler
- [x] FB2 format handler
- [x] CBZ format handler
- [x] TXT format handler
- [x] PDF format handler
- [x] CLI interface with clap
- [x] Format detection utility
- [x] Basic tests for TXT handler
- [x] README documentation
- [x] Architecture documentation
- [x] Example files
- [x] **MCP (Model Context Protocol) server implementation**
- [x] **MCP tools for ebook operations (read, write, validate, extract, info)**
- [x] **MCP documentation and configuration**
- [x] **Library API for use in Rust projects**
- [x] **Improved EPUB metadata extraction (cover image path, spine, manifest, tags)**
- [x] **Enhanced EPUB chapter parsing with proper spine order**
- [x] **Improved MOBI/KF8 header parsing with full metadata extraction**
- [x] **Enhanced MOBI TOC extraction and text encoding detection**
- [x] **Improved PDF text extraction with better content stream parsing**
- [x] **Added tags support to Metadata structure**

- [x] **Comprehensive tests for EPUB handler (6 tests)**
- [x] **Comprehensive tests for MOBI handler (6 tests)**
- [x] **Comprehensive tests for PDF handler (6 tests)**
- [x] **Fixed PDF metadata reading/writing with Info dictionary dereferencing**
- [x] **All 18 format handler tests passing**
- [x] **Format conversion implementation (TXT ↔ EPUB, TXT ↔ PDF, TXT ↔ MOBI, TXT ↔ FB2, EPUB → TXT, EPUB → PDF, MOBI → TXT, FB2 → TXT, PDF → TXT)**
- [x] **Progress indicator module for large file operations**
- [x] **Conversion integration with CLI convert command**
- [x] **Conversion tool in MCP server**
- [x] **11 comprehensive conversion tests (all passing)**
- [x] **CLI progress indicators for convert, write, and repair operations**
- [x] **13 comprehensive CLI integration tests (all passing)**
- [x] **ComicInfo.xml support for CBZ format (read/write)**
- [x] **Image optimization module with resize and compression**
- [x] **Image optimization for CBZ handler**
- [x] **Image optimization for EPUB handler**
- [x] **5 comprehensive CBZ metadata tests (all passing)**
- [x] **6 comprehensive image optimization tests (all passing)**
- [x] **CLI optimize command with full options (max-width, max-height, quality, no-resize, progress)**
- [x] **MCP optimize_images tool for AI assistant integration**
- [x] **7 comprehensive CLI optimize tests (all passing)**
- [x] **EPUB 3.0 features support (nav.xhtml, version switching, semantic markup)**
- [x] **7 comprehensive EPUB 3.0 tests (all passing)**
- [x] **Logging support with env_logger (log levels: trace, debug, info, warn, error)**
- [x] **Performance benchmarks with criterion (EPUB/CBZ read/write, optimization)**
- [x] **AZW format support (Kindle format with DRM detection and metadata extraction)**
- [x] **8 comprehensive AZW tests (all passing)**
- [x] **Streaming support for large files (TXT 10MB+, EPUB 50MB+ thresholds)**
- [x] **8 comprehensive streaming tests (all passing)**
- [x] **Improved error messages with hints using thiserror**
- [x] **Trait streaming helper tests (read_from_bytes/read_from_reader/write_to_writer) including concurrency safety (4 tests)**
- [x] **Error message hint tests for EbookError (5 tests)**
- [x] **Modern Rust formatting and code quality improvements**

## In Progress 🚧

*No items currently in progress*

## Planned 📋

### High Priority
*All high-priority items completed*

### Medium Priority
*All medium-priority items completed*

### Low Priority
- [ ] Add support for DJVU format
- [ ] Add support for CHM format
- [ ] Implement OCR for scanned PDFs
- [ ] Add GUI wrapper
- [ ] Create web service API
- [ ] Add batch processing capabilities
- [ ] Implement ebook library management features

### Code Quality
- [x] Add benchmarks for performance testing (6 benchmarks with criterion)
- [x] Add logging support (env_logger with configurable log levels)
- [x] Improve error messages with suggestions (thiserror with hints)
- [x] All tests passing (Currently: 101 tests - 13 unit + 8 AZW + 5 CBZ metadata + 7 CLI optimize + 13 CLI + 11 conversion + 7 EPUB + 6 EPUB3 + 6 image optimization + 6 MOBI + 6 PDF + 8 streaming + 6 TXT + 4 trait streaming + 5 error hints)
- [ ] Create CI/CD pipeline

### Documentation
- [ ] Add API documentation (rustdoc)
- [ ] Create user guide with more examples
- [ ] Add troubleshooting guide
- [ ] Document format-specific limitations
- [ ] Add contributing guidelines

## Known Issues 🐛

- [ ] EPUB: Chapter structure not fully parsed
- [ ] MOBI: Limited metadata extraction (basic implementation)
- [ ] PDF: Text extraction may miss formatting
- [ ] FB2: Image extraction not implemented
- [ ] Warning: Unused Chapter struct fields in EPUB handler

## Future Ideas 💡

- Plugin system for custom format handlers
- Cloud storage integration (S3, Google Drive)
- Ebook metadata editing without full rewrite
- Support for DRM-free ebook management
- Ebook format validation tools
- Automatic metadata fetching from online databases
- Reading statistics and analytics
