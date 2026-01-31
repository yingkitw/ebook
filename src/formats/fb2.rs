use crate::{EbookError, Metadata, Result};
use crate::traits::{EbookReader, EbookWriter, EbookOperator, TocEntry, ImageData};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Default)]
pub struct Fb2Handler {
    metadata: Metadata,
    content: String,
    chapters: Vec<String>,
    images: Vec<ImageData>,
}

impl Fb2Handler {
    pub fn new() -> Self {
        Self::default()
    }

    fn parse_fb2(&mut self, xml_content: &str) -> Result<()> {
        use quick_xml::Reader;
        use quick_xml::events::Event;

        let mut reader = Reader::from_str(xml_content);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        let mut in_title_info = false;
        let mut in_body = false;
        let mut current_tag = String::new();
        let mut current_text = String::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if name == "title-info" {
                        in_title_info = true;
                    } else if name == "body" {
                        in_body = true;
                    }
                    current_tag = name;
                }
                Ok(Event::Text(e)) => {
                    let text = e.unescape().unwrap_or_default().to_string();
                    
                    if in_title_info {
                        match current_tag.as_str() {
                            "book-title" => self.metadata.title = Some(text.clone()),
                            "first-name" | "last-name" => {
                                let author = self.metadata.author.get_or_insert_with(String::new);
                                if !author.is_empty() {
                                    author.push(' ');
                                }
                                author.push_str(&text);
                            }
                            "lang" => self.metadata.language = Some(text.clone()),
                            _ => {}
                        }
                    }
                    
                    if in_body {
                        current_text.push_str(&text);
                        current_text.push(' ');
                    }
                }
                Ok(Event::End(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if name == "title-info" {
                        in_title_info = false;
                    } else if name == "body" {
                        in_body = false;
                    } else if name == "p" && in_body {
                        current_text.push('\n');
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(EbookError::Xml(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        self.content = current_text;
        self.metadata.format = Some("FB2".to_string());
        Ok(())
    }
}

impl EbookReader for Fb2Handler {
    fn read_from_file(&mut self, path: &Path) -> Result<()> {
        let mut file = File::open(path)?;
        let mut xml_content = String::new();
        file.read_to_string(&mut xml_content)?;

        self.parse_fb2(&xml_content)?;
        Ok(())
    }

    fn get_metadata(&self) -> Result<Metadata> {
        Ok(self.metadata.clone())
    }

    fn get_content(&self) -> Result<String> {
        Ok(self.content.clone())
    }

    fn get_toc(&self) -> Result<Vec<TocEntry>> {
        Ok(Vec::new())
    }

    fn extract_images(&self) -> Result<Vec<ImageData>> {
        Ok(self.images.clone())
    }
}

impl EbookWriter for Fb2Handler {
    fn set_metadata(&mut self, metadata: Metadata) -> Result<()> {
        self.metadata = metadata;
        Ok(())
    }

    fn set_content(&mut self, content: &str) -> Result<()> {
        self.content = content.to_string();
        Ok(())
    }

    fn add_chapter(&mut self, _title: &str, content: &str) -> Result<()> {
        self.chapters.push(content.to_string());
        Ok(())
    }

    fn add_image(&mut self, name: &str, data: Vec<u8>) -> Result<()> {
        let mime_type = crate::utils::guess_mime_type(name);
        self.images.push(ImageData::new(name.to_string(), mime_type, data));
        Ok(())
    }

    fn write_to_file(&self, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = File::create(path)?;
        
        let title = self.metadata.title.as_deref().unwrap_or("Untitled");
        let author = self.metadata.author.as_deref().unwrap_or("Unknown");
        let lang = self.metadata.language.as_deref().unwrap_or("en");

        let fb2_content = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<FictionBook xmlns="http://www.gribuser.ru/xml/fictionbook/2.0">
  <description>
    <title-info>
      <book-title>{}</book-title>
      <author>
        <first-name>{}</first-name>
      </author>
      <lang>{}</lang>
    </title-info>
  </description>
  <body>
    <section>
      <p>{}</p>
    </section>
  </body>
</FictionBook>"#, title, author, lang, self.content.replace('\n', "</p>\n      <p>"));

        file.write_all(fb2_content.as_bytes())?;
        Ok(())
    }
}

impl EbookOperator for Fb2Handler {
    fn convert_to(&self, _target_format: &str, _output_path: &Path) -> Result<()> {
        Err(EbookError::NotSupported("Conversion not yet implemented".to_string()))
    }

    fn validate(&self) -> Result<bool> {
        Ok(self.metadata.title.is_some())
    }

    fn repair(&mut self) -> Result<()> {
        if self.metadata.title.is_none() {
            self.metadata.title = Some("Untitled".to_string());
        }
        Ok(())
    }
}
