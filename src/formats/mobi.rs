use crate::{EbookError, Metadata, Result};
use crate::traits::{EbookReader, EbookWriter, EbookOperator, TocEntry, ImageData};
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Default)]
pub struct MobiHandler {
    metadata: Metadata,
    content: String,
    images: Vec<ImageData>,
    raw_data: Vec<u8>,
    mobi_header: Option<MobiHeader>,
    toc: Vec<TocEntry>,
}

#[derive(Debug, Clone, Default)]
struct MobiHeader {
    magic: [u8; 4],
    header_length: u32,
    mobi_type: u32,
    text_encoding: u32,
    _id: u32,
    _gen_version: u32,
    first_image_index: u32,
}

impl MobiHandler {
    pub fn new() -> Self {
        Self::default()
    }

    fn parse_mobi_header(&mut self) -> Result<()> {
        if self.raw_data.len() < 78 {
            return Err(EbookError::InvalidStructure("File too small".to_string()));
        }

        // Check for MOBI magic number at position 0x3C (60) in the file
        // MOBI files have a PalmDOC header first, then MOBI header
        let mobi_magic_pos = 60;
        if self.raw_data.len() > mobi_magic_pos + 4 {
            let magic = &self.raw_data[mobi_magic_pos..mobi_magic_pos + 4];
            if magic == b"MOBI" {
                return self.parse_full_mobi_header(mobi_magic_pos);
            }
        }

        // Fallback: simple name parsing for older formats
        let name = std::str::from_utf8(&self.raw_data[0..32])
            .unwrap_or("Unknown")
            .trim_end_matches('\0');

        if !name.is_empty() {
            self.metadata.title = Some(name.to_string());
        }

        self.metadata.format = Some("MOBI".to_string());
        Ok(())
    }

    fn parse_full_mobi_header(&mut self, pos: usize) -> Result<()> {
        if self.raw_data.len() < pos + 232 {
            return Err(EbookError::InvalidStructure("MOBI header too small".to_string()));
        }

        let mut header = MobiHeader::default();
        header.magic.copy_from_slice(&self.raw_data[pos..pos + 4]);

        // Parse header length (offset +4, 4 bytes)
        header.header_length = u32::from_be_bytes([
            self.raw_data[pos + 4],
            self.raw_data[pos + 5],
            self.raw_data[pos + 6],
            self.raw_data[pos + 7],
        ]);

        // Parse MOBI type (offset +8, 4 bytes)
        header.mobi_type = u32::from_be_bytes([
            self.raw_data[pos + 8],
            self.raw_data[pos + 9],
            self.raw_data[pos + 10],
            self.raw_data[pos + 11],
        ]);

        // Parse text encoding (offset +16, 4 bytes)
        header.text_encoding = u32::from_be_bytes([
            self.raw_data[pos + 16],
            self.raw_data[pos + 17],
            self.raw_data[pos + 18],
            self.raw_data[pos + 19],
        ]);

        // Parse first image index (offset +76, 4 bytes)
        if self.raw_data.len() > pos + 80 {
            header.first_image_index = u32::from_be_bytes([
                self.raw_data[pos + 76],
                self.raw_data[pos + 77],
                self.raw_data[pos + 78],
                self.raw_data[pos + 79],
            ]);
        }

        self.mobi_header = Some(header);

        // Extract full name length (offset +88, 1 byte)
        if self.raw_data.len() > pos + 88 {
            let name_length = self.raw_data[pos + 88] as usize;
            if self.raw_data.len() > pos + 92 + name_length {
                let name_bytes = &self.raw_data[pos + 92..pos + 92 + name_length];
                if let Ok(name) = std::str::from_utf8(name_bytes) {
                    self.metadata.title = Some(name.to_string());
                }
            }
        }

        // Extract language (offset +108, 2 bytes)
        if self.raw_data.len() > pos + 110 {
            let lang_id = u16::from_be_bytes([
                self.raw_data[pos + 108],
                self.raw_data[pos + 109],
            ]);
            self.metadata.language = Some(self.language_id_to_code(lang_id));
        }

        self.metadata.format = Some("MOBI".to_string());
        Ok(())
    }

    fn language_id_to_code(&self, id: u16) -> String {
        // Common language IDs from MOBI/PalmDOC spec
        match id {
            0 => "en".to_string(),
            1 => "fr".to_string(),
            2 => "de".to_string(),
            3 => "it".to_string(),
            4 => "es".to_string(),
            5 => "nl".to_string(),
            6 => "sv".to_string(),
            7 => "nb".to_string(),
            8 => "da".to_string(),
            9 => "fi".to_string(),
            10 => "ja".to_string(),
            11 => "zh".to_string(),
            12 => "ko".to_string(),
            13 => "ar".to_string(),
            _ => "en".to_string(),
        }
    }

    fn extract_text(&mut self) -> Result<()> {
        // Text content starts after the headers
        let text_start = if let Some(header) = &self.mobi_header {
            // For MOBI format, text typically starts after the full header
            header.header_length as usize + 60
        } else {
            78
        };

        if self.raw_data.len() > text_start {
            let text_data = &self.raw_data[text_start..];

            // Try to detect UTF-16 encoding first
            if text_data.len() >= 2 {
                let bom = u16::from_be_bytes([text_data[0], text_data[1]]);
                if bom == 0xFEFF || bom == 0xFFFE {
                    if let Ok(text) = String::from_utf16(
                        &text_data[2..]
                            .chunks(2)
                            .map(|c| u16::from_be_bytes([c[0], c[1]]))
                            .collect::<Vec<_>>()
                    ) {
                        self.content = text;
                        return Ok(());
                    }
                }
            }

            // Try UTF-8
            if let Ok(text) = std::str::from_utf8(text_data) {
                self.content = text.to_string();
            } else {
                // Fallback to encoding detection
                let (decoded, _, _) = encoding_rs::UTF_8.decode(text_data);
                self.content = decoded.to_string();
            }
        }

        // Clean up common MOBI formatting artifacts
        self.content = self.content
            .replace("<mbp:pagebreak>", "\n\n---\n\n")
            .replace("</mbp:pagebreak>", "")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&apos;", "'");

        Ok(())
    }

    fn extract_toc(&mut self) -> Result<()> {
        // Basic TOC extraction - look for chapter patterns
        let mut toc = Vec::new();
        let lines: Vec<&str> = self.content.lines().collect();

        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            // Look for potential chapter headings
            if trimmed.starts_with("Chapter ")
                || trimmed.starts_with("CHAPTER ")
                || trimmed.starts_with("# ")
                || (trimmed.len() < 100 && trimmed.chars().all(|c| c.is_uppercase() || c == ' '))
            {
                toc.push(TocEntry {
                    id: idx as u32,
                    level: 0,
                    title: trimmed.to_string(),
                    href: None,
                    children: Vec::new(),
                });
            }
        }

        self.toc = toc;
        Ok(())
    }
}

impl EbookReader for MobiHandler {
    fn read_from_file(&mut self, path: &Path) -> Result<()> {
        let mut file = File::open(path)?;
        file.read_to_end(&mut self.raw_data)?;

        self.parse_mobi_header()?;
        self.extract_text()?;
        self.extract_toc()?;

        Ok(())
    }

    fn get_metadata(&self) -> Result<Metadata> {
        Ok(self.metadata.clone())
    }

    fn get_content(&self) -> Result<String> {
        Ok(self.content.clone())
    }

    fn get_toc(&self) -> Result<Vec<TocEntry>> {
        Ok(self.toc.clone())
    }

    fn extract_images(&self) -> Result<Vec<ImageData>> {
        Ok(self.images.clone())
    }
}

impl EbookWriter for MobiHandler {
    fn set_metadata(&mut self, metadata: Metadata) -> Result<()> {
        self.metadata = metadata;
        Ok(())
    }

    fn set_content(&mut self, content: &str) -> Result<()> {
        self.content = content.to_string();
        Ok(())
    }

    fn add_chapter(&mut self, _title: &str, content: &str) -> Result<()> {
        self.content.push_str(content);
        self.content.push('\n');
        Ok(())
    }

    fn add_image(&mut self, name: &str, data: Vec<u8>) -> Result<()> {
        let mime_type = crate::utils::guess_mime_type(name);
        self.images.push(ImageData::new(name.to_string(), mime_type, data));
        Ok(())
    }

    fn write_to_file(&self, path: &Path) -> Result<()> {
        use std::io::Write;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = File::create(path)?;
        
        let mut header = vec![0u8; 78];
        let title = self.metadata.title.as_deref().unwrap_or("Untitled");
        let title_bytes = title.as_bytes();
        let copy_len = title_bytes.len().min(32);
        header[0..copy_len].copy_from_slice(&title_bytes[0..copy_len]);
        
        file.write_all(&header)?;
        file.write_all(self.content.as_bytes())?;
        
        Ok(())
    }
}

impl EbookOperator for MobiHandler {
    fn convert_to(&self, _target_format: &str, _output_path: &Path) -> Result<()> {
        Err(EbookError::NotSupported("Conversion not yet implemented".to_string()))
    }

    fn validate(&self) -> Result<bool> {
        Ok(!self.raw_data.is_empty())
    }

    fn repair(&mut self) -> Result<()> {
        if self.metadata.title.is_none() {
            self.metadata.title = Some("Untitled".to_string());
        }
        Ok(())
    }
}
