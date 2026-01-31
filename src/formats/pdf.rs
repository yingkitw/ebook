use crate::{EbookError, Metadata, Result};
use crate::traits::{EbookReader, EbookWriter, EbookOperator, TocEntry, ImageData};
use std::path::Path;
use lopdf::{Document, dictionary};

#[derive(Default)]
pub struct PdfHandler {
    metadata: Metadata,
    content: String,
    document: Option<Document>,
}

impl PdfHandler {
    pub fn new() -> Self {
        Self::default()
    }

    fn extract_metadata(&mut self, doc: &Document) -> Result<()> {
        if let Ok(info_ref) = doc.trailer.get(b"Info") {
            // Dereference if it's an indirect object
            let info_obj = if let Ok(obj_id) = info_ref.as_reference() {
                doc.get_object(obj_id).ok()
            } else {
                Some(info_ref)
            };
            
            if let Some(info) = info_obj {
                if let Ok(info_dict) = info.as_dict() {
                    if let Ok(title) = info_dict.get(b"Title") {
                        if let Ok(title_str) = title.as_string() {
                            self.metadata.title = Some(title_str.to_string());
                        }
                    }
                    
                    if let Ok(author) = info_dict.get(b"Author") {
                        if let Ok(author_str) = author.as_string() {
                            self.metadata.author = Some(author_str.to_string());
                        }
                    }
                    
                    if let Ok(subject) = info_dict.get(b"Subject") {
                        if let Ok(subject_str) = subject.as_string() {
                            self.metadata.publisher = Some(subject_str.to_string());
                        }
                    }
                }
            }
        }
        
        self.metadata.format = Some("PDF".to_string());
        Ok(())
    }

    fn extract_text(&mut self, doc: &Document) -> Result<()> {
        let mut text = String::new();
        let pages = doc.get_pages();

        for (page_num, page_id) in pages.iter() {
            // Try to extract text using the page's content
            if let Ok(content) = doc.get_page_content(*page_id) {
                let page_text = self.decode_pdf_text(&content);
                text.push_str(&page_text);
                text.push('\n');
            }

            // Add page separator
            text.push_str(&format!("\n--- Page {page_num} ---\n"));
        }

        // Clean up the extracted text
        self.content = self.clean_pdf_text(&text);
        Ok(())
    }

    fn decode_pdf_text(&self, content: &[u8]) -> String {
        let mut text = String::new();
        let content_str = String::from_utf8_lossy(content);

        // Parse PDF content stream operators
        let mut i = 0;
        let chars: Vec<char> = content_str.chars().collect();

        while i < chars.len() {
            // Look for text operators
            if i + 1 < chars.len() {
                let c1 = chars[i];
                let c2 = chars[i + 1];

                // Tj operator: single string
                if c1 == 'T' && c2 == 'j' {
                    // Find the string before this operator
                    let substring = self.extract_last_string(&content_str[..i]);
                    text.push_str(&substring);
                    text.push(' ');
                }
                // TJ operator: array of strings with spacing
                else if c1 == 'T' && c2 == 'J' {
                    let substring = self.extract_last_string(&content_str[..i]);
                    text.push_str(&substring);
                    text.push(' ');
                }
            }
            i += 1;
        }

        text
    }

    fn extract_last_string(&self, content: &str) -> String {
        // Find the last balanced parenthesized string
        let mut result = String::new();
        let mut paren_depth = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let mut temp = String::new();

        for c in content.chars().rev() {
            if escape_next {
                temp.insert(0, c);
                escape_next = false;
                continue;
            }

            if c == '\\' {
                escape_next = true;
                temp.insert(0, c);
                continue;
            }

            if c == ')' {
                paren_depth += 1;
                in_string = true;
                temp.insert(0, c);
            } else if c == '(' {
                paren_depth -= 1;
                temp.insert(0, c);
                if paren_depth == 0 && in_string {
                    result = temp;
                    break;
                }
            } else if in_string {
                temp.insert(0, c);
            }
        }

        // Remove the parentheses and unescape
        result.trim_start_matches('(')
            .trim_end_matches(')')
            .replace("\\(", "(")
            .replace("\\)", ")")
            .replace("\\\\", "\\")
            .to_string()
    }

    fn clean_pdf_text(&self, text: &str) -> String {
        text.replace("\\(", "(")
            .replace("\\)", ")")
            .replace("\\[", "[")
            .replace("\\]", "]")
            .replace("\\{", "{")
            .replace("\\}", "}")
            .replace("\\\\", "\\")
            .split("--- Page")
            .map(|s| s.trim())
            .collect::<Vec<_>>()
            .join("\n\n")
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.starts_with("---"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl EbookReader for PdfHandler {
    fn read_from_file(&mut self, path: &Path) -> Result<()> {
        let doc = Document::load(path)?;
        
        self.extract_metadata(&doc)?;
        self.extract_text(&doc)?;
        
        self.document = Some(doc);
        Ok(())
    }

    fn get_metadata(&self) -> Result<Metadata> {
        Ok(self.metadata.clone())
    }

    fn get_content(&self) -> Result<String> {
        Ok(self.content.clone())
    }

    fn get_toc(&self) -> Result<Vec<TocEntry>> {
        if let Some(doc) = &self.document {
            if let Ok(catalog) = doc.catalog() {
                if let Ok(_outlines) = catalog.get(b"Outlines") {
                    return Ok(Vec::new());
                }
            }
        }
        Ok(Vec::new())
    }

    fn extract_images(&self) -> Result<Vec<ImageData>> {
        Ok(Vec::new())
    }
}

impl EbookWriter for PdfHandler {
    fn set_metadata(&mut self, metadata: Metadata) -> Result<()> {
        self.metadata = metadata;
        Ok(())
    }

    fn set_content(&mut self, content: &str) -> Result<()> {
        self.content = content.to_string();
        Ok(())
    }

    fn add_chapter(&mut self, _title: &str, content: &str) -> Result<()> {
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

        let mut doc = Document::with_version("1.5");
        
        let pages_id = doc.new_object_id();
        let font_id = doc.add_object(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Helvetica",
        });
        
        let resources_id = doc.add_object(dictionary! {
            "Font" => dictionary! {
                "F1" => font_id,
            },
        });
        
        let content = format!("BT /F1 12 Tf 50 750 Td ({}) Tj ET", 
                             self.content.replace(')', "\\)").replace('(', "\\("));
        let content_id = doc.add_object(lopdf::Stream::new(
            dictionary! {},
            content.as_bytes().to_vec(),
        ));
        
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "Contents" => content_id,
            "Resources" => resources_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
        });
        
        let pages = dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page_id.into()],
            "Count" => 1,
        };
        doc.objects.insert(pages_id, lopdf::Object::Dictionary(pages));
        
        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        
        doc.trailer.set("Root", catalog_id);
        
        // Create Info dictionary with metadata
        let mut info_dict = lopdf::Dictionary::new();
        
        if let Some(title) = &self.metadata.title {
            info_dict.set("Title", lopdf::Object::String(title.as_bytes().to_vec(), lopdf::StringFormat::Literal));
        }
        
        if let Some(author) = &self.metadata.author {
            info_dict.set("Author", lopdf::Object::String(author.as_bytes().to_vec(), lopdf::StringFormat::Literal));
        }
        
        if let Some(publisher) = &self.metadata.publisher {
            info_dict.set("Subject", lopdf::Object::String(publisher.as_bytes().to_vec(), lopdf::StringFormat::Literal));
        }
        
        if !info_dict.is_empty() {
            let info_id = doc.add_object(info_dict);
            doc.trailer.set("Info", info_id);
        }
        
        doc.save(path)?;
        Ok(())
    }
}

impl EbookOperator for PdfHandler {
    fn convert_to(&self, _target_format: &str, _output_path: &Path) -> Result<()> {
        Err(EbookError::NotSupported("Conversion not yet implemented".to_string()))
    }

    fn validate(&self) -> Result<bool> {
        Ok(self.document.is_some())
    }

    fn repair(&mut self) -> Result<()> {
        if self.metadata.title.is_none() {
            self.metadata.title = Some("Untitled".to_string());
        }
        Ok(())
    }
}
