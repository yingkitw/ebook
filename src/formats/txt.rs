use crate::{EbookError, Metadata, Result};
use crate::traits::{EbookReader, EbookWriter, EbookOperator, TocEntry, ImageData};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;

#[derive(Default)]
pub struct TxtHandler {
    metadata: Metadata,
    content: String,
}

const STREAMING_THRESHOLD: usize = 10 * 1024 * 1024; // 10 MB

impl TxtHandler {
    pub fn new() -> Self {
        Self::default()
    }

    fn detect_encoding(data: &[u8]) -> Result<String> {
        if let Ok(text) = std::str::from_utf8(data) {
            return Ok(text.to_string());
        }

        let (decoded, _encoding, had_errors) = encoding_rs::UTF_8.decode(data);
        if !had_errors {
            return Ok(decoded.to_string());
        }

        let (decoded, _, _) = encoding_rs::WINDOWS_1252.decode(data);
        Ok(decoded.to_string())
    }

    /// Optimized streaming read for large text files
    pub fn read_from_file_streaming(&mut self, path: &Path) -> Result<()> {
        let file = File::open(path)?;
        let metadata = file.metadata()?;
        let file_size = metadata.len() as usize;

        self.metadata.title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string());
        self.metadata.format = Some("TXT".to_string());

        // For small files, use the regular method
        if file_size < STREAMING_THRESHOLD {
            return self.read_from_file(path);
        }

        // For large files, use streaming with buffered reading
        log::info!("Streaming large TXT file ({} bytes)", file_size);
        let reader = BufReader::with_capacity(128 * 1024, file); // 128KB buffer
        let mut content = String::with_capacity(file_size);

        for line in reader.lines() {
            let line = line?;
            content.push_str(&line);
            content.push('\n');
        }

        self.content = content;
        Ok(())
    }

    /// Optimized streaming write for large text files
    pub fn write_to_file_streaming(&self, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = File::create(path)?;
        let mut writer = io::BufWriter::with_capacity(128 * 1024, file); // 128KB buffer

        // Write in chunks to avoid memory pressure
        let content_bytes = self.content.as_bytes();
        let chunk_size = 64 * 1024; // 64KB chunks

        for chunk in content_bytes.chunks(chunk_size) {
            writer.write_all(chunk)?;
        }

        writer.flush()?;
        Ok(())
    }
}

impl EbookReader for TxtHandler {
    fn read_from_file(&mut self, path: &Path) -> Result<()> {
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        self.content = Self::detect_encoding(&data)?;
        
        self.metadata.title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string());
        self.metadata.format = Some("TXT".to_string());

        Ok(())
    }

    fn get_metadata(&self) -> Result<Metadata> {
        Ok(self.metadata.clone())
    }

    fn get_content(&self) -> Result<String> {
        Ok(self.content.clone())
    }

    fn get_toc(&self) -> Result<Vec<TocEntry>> {
        let mut toc = Vec::new();
        let lines: Vec<&str> = self.content.lines().collect();
        
        for line in lines.iter() {
            let trimmed = line.trim();
            if trimmed.starts_with("Chapter ") || trimmed.starts_with("CHAPTER ") {
                toc.push(TocEntry::new(trimmed.to_string(), 1));
            }
        }
        
        Ok(toc)
    }

    fn extract_images(&self) -> Result<Vec<ImageData>> {
        Ok(Vec::new())
    }
}

impl EbookWriter for TxtHandler {
    fn set_metadata(&mut self, metadata: Metadata) -> Result<()> {
        self.metadata = metadata;
        Ok(())
    }

    fn set_content(&mut self, content: &str) -> Result<()> {
        self.content = content.to_string();
        Ok(())
    }

    fn add_chapter(&mut self, title: &str, content: &str) -> Result<()> {
        self.content.push_str("\n\n");
        self.content.push_str(title);
        self.content.push_str("\n\n");
        self.content.push_str(content);
        Ok(())
    }

    fn add_image(&mut self, _name: &str, _data: Vec<u8>) -> Result<()> {
        Ok(())
    }

    fn write_to_file(&self, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = File::create(path)?;
        file.write_all(self.content.as_bytes())?;
        Ok(())
    }
}

impl EbookOperator for TxtHandler {
    fn convert_to(&self, target_format: &str, output_path: &Path) -> Result<()> {
        match target_format {
            "md" | "markdown" => {
                let mut file = File::create(output_path)?;
                let title = self.metadata.title.as_deref().unwrap_or("Untitled");
                file.write_all(format!("# {}\n\n{}", title, self.content).as_bytes())?;
                Ok(())
            }
            _ => Err(EbookError::NotSupported(format!("Conversion to {target_format} not supported")))
        }
    }

    fn validate(&self) -> Result<bool> {
        Ok(!self.content.is_empty())
    }

    fn repair(&mut self) -> Result<()> {
        self.content = self.content.trim().to_string();
        if self.metadata.title.is_none() {
            self.metadata.title = Some("Untitled".to_string());
        }
        Ok(())
    }
}
