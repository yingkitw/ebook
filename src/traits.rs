use crate::{Metadata, Result};
use std::path::Path;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn unique_temp_file(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let c = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{name}_{}_{}_{}.tmp", std::process::id(), nanos, c))
}

pub trait EbookReader {
    fn read_from_file(&mut self, path: &Path) -> Result<()>;

    /// Read ebook from a generic reader (for streaming large files)
    fn read_from_reader<R: Read>(&mut self, reader: R) -> Result<()> {
        // Default implementation buffers the entire reader
        let mut buffer = Vec::new();
        let mut reader = reader;
        reader.read_to_end(&mut buffer)?;
        self.read_from_bytes(&buffer)
    }

    /// Read ebook from bytes (helper for streaming)
    fn read_from_bytes(&mut self, data: &[u8]) -> Result<()> {
        // Default implementation creates a temporary file
        let temp_file = unique_temp_file("ebook_temp_read");
        {
            let mut file = std::fs::File::create(&temp_file)?;
            file.write_all(data)?;
        }
        let result = self.read_from_file(&temp_file);
        let _ = std::fs::remove_file(&temp_file);
        result
    }

    fn get_metadata(&self) -> Result<Metadata>;
    fn get_content(&self) -> Result<String>;
    fn get_toc(&self) -> Result<Vec<TocEntry>>;
    fn extract_images(&self) -> Result<Vec<ImageData>>;
}

pub trait EbookWriter {
    fn set_metadata(&mut self, metadata: Metadata) -> Result<()>;
    fn set_content(&mut self, content: &str) -> Result<()>;
    fn add_chapter(&mut self, title: &str, content: &str) -> Result<()>;
    fn add_image(&mut self, name: &str, data: Vec<u8>) -> Result<()>;
    fn write_to_file(&self, path: &Path) -> Result<()>;

    /// Write ebook to a generic writer (for streaming large files)
    fn write_to_writer<W: Write>(&self, writer: W) -> Result<()> {
        // Default implementation writes to a buffer first
        let mut buffer = Vec::new();
        {
            let mut writer = std::io::Cursor::new(&mut buffer);
            self.write_to_writer_internal(&mut writer)?;
        }
        let mut writer = writer;
        writer.write_all(&buffer)?;
        Ok(())
    }

    /// Internal write method for streaming (override in handlers)
    fn write_to_writer_internal<W: Write>(&self, writer: &mut W) -> Result<()> {
        // Default implementation: create temp file and copy it
        use std::io::Read;
        let temp_file = unique_temp_file("ebook_temp_write");
        self.write_to_file(&temp_file)?;
        let mut file = std::fs::File::open(&temp_file)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        writer.write_all(&buffer)?;
        let _ = std::fs::remove_file(&temp_file);
        Ok(())
    }
}

pub trait EbookOperator: EbookReader + EbookWriter {
    fn convert_to(&self, target_format: &str, output_path: &Path) -> Result<()>;
    fn validate(&self) -> Result<bool>;
    fn repair(&mut self) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct TocEntry {
    pub id: u32,
    pub title: String,
    pub level: usize,
    pub href: Option<String>,
    pub children: Vec<TocEntry>,
}

#[derive(Debug, Clone)]
pub struct ImageData {
    pub name: String,
    pub mime_type: String,
    pub data: Vec<u8>,
}

impl TocEntry {
    pub fn new(title: String, level: usize) -> Self {
        Self {
            id: 0,
            title,
            level,
            href: None,
            children: Vec::new(),
        }
    }

    pub fn with_id(mut self, id: u32) -> Self {
        self.id = id;
        self
    }

    pub fn with_href(mut self, href: String) -> Self {
        self.href = Some(href);
        self
    }
}

impl ImageData {
    pub fn new(name: String, mime_type: String, data: Vec<u8>) -> Self {
        Self {
            name,
            mime_type,
            data,
        }
    }
}
