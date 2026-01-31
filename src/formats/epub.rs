use crate::{EbookError, Metadata, Result};
use crate::traits::{EbookReader, EbookWriter, EbookOperator, TocEntry, ImageData};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::collections::HashMap;
use zip::ZipArchive;
use zip::write::{ZipWriter, FileOptions};

#[derive(Default)]
pub struct EpubHandler {
    metadata: Metadata,
    content: String,
    chapters: Vec<Chapter>,
    images: Vec<ImageData>,
    toc: Vec<TocEntry>,
    epub_version: EpubVersion,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(Default)]
pub enum EpubVersion {
    V2,
    #[default]
    V3,
}


#[derive(Debug, Clone)]
struct Chapter {
    title: String,
    content: String,
    filename: String,
}

const STREAMING_THRESHOLD: u64 = 50 * 1024 * 1024; // 50 MB

impl EpubHandler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_epub_version(&mut self, version: EpubVersion) {
        self.epub_version = version;
    }

    pub fn get_epub_version(&self) -> EpubVersion {
        self.epub_version
    }

    /// Check if file should use streaming based on size
    pub fn should_use_streaming(path: &Path) -> Result<bool> {
        let metadata = std::fs::metadata(path)?;
        Ok(metadata.len() > STREAMING_THRESHOLD)
    }

    fn generate_nav_xhtml(&self) -> String {
        let mut nav = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<head>
    <title>Navigation</title>
</head>
<body>
    <nav epub:type="toc" id="toc">
        <h1>Table of Contents</h1>
        <ol>
"#);

        for chapter in self.chapters.iter() {
            nav.push_str(&format!(
                "            <li><a href=\"{}\">{}</a></li>\n",
                chapter.filename,
                chapter.title
            ));
        }

        nav.push_str(r#"        </ol>
    </nav>
</body>
</html>"#);

        nav
    }

    fn parse_opf(&mut self, opf_content: &str) -> Result<()> {
        use quick_xml::Reader;
        use quick_xml::events::Event;

        let mut reader = Reader::from_str(opf_content);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        let mut in_metadata = false;
        let mut in_manifest = false;
        let mut in_spine = false;
        let mut current_tag = String::new();
        let mut manifest_items: HashMap<String, String> = HashMap::new();
        let mut spine_items: Vec<String> = Vec::new();
        let mut cover_id: Option<String> = None;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if name == "metadata" {
                        in_metadata = true;
                    } else if name == "manifest" {
                        in_manifest = true;
                    } else if name == "spine" {
                        in_spine = true;
                    }

                    // Check for cover image in metadata
                    if in_metadata && name == "meta" {
                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                let value = String::from_utf8_lossy(&attr.value).to_string();
                                if key == "name" && value == "cover" {
                                    cover_id = Some(String::new()); // Will be filled by content attribute
                                } else if key == "content" && cover_id.is_some() {
                                    cover_id = Some(value);
                                }
                            }
                        }
                    }

                    // Parse manifest items
                    if in_manifest && name == "item" {
                        let mut id = String::new();
                        let mut href = String::new();
                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                let value = String::from_utf8_lossy(&attr.value).to_string();
                                if key == "id" {
                                    id = value;
                                } else if key == "href" {
                                    href = value;
                                }
                            }
                        }
                        if !id.is_empty() && !href.is_empty() {
                            manifest_items.insert(id, href);
                        }
                    }

                    // Parse spine items
                    if in_spine && name == "itemref" {
                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                if key == "idref" {
                                    spine_items.push(String::from_utf8_lossy(&attr.value).to_string());
                                }
                            }
                        }
                    }

                    current_tag = name;
                }
                Ok(Event::Text(e)) => {
                    if in_metadata {
                        let text = e.unescape().unwrap_or_default().to_string();
                        match current_tag.as_str() {
                            "dc:title" => self.metadata.title = Some(text),
                            "dc:creator" => self.metadata.author = Some(text),
                            "dc:publisher" => self.metadata.publisher = Some(text),
                            "dc:description" => self.metadata.description = Some(text),
                            "dc:language" => self.metadata.language = Some(text),
                            "dc:identifier" => {
                                if self.metadata.isbn.is_none() {
                                    self.metadata.isbn = Some(text);
                                }
                            }
                            "dc:date" => self.metadata.publication_date = Some(text),
                            "dc:subject" => {
                                if self.metadata.tags.is_none() {
                                    self.metadata.tags = Some(Vec::new());
                                }
                                if let Some(tags) = &mut self.metadata.tags {
                                    tags.push(text);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Ok(Event::End(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if name == "metadata" {
                        in_metadata = false;
                    } else if name == "manifest" {
                        in_manifest = false;
                    } else if name == "spine" {
                        in_spine = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(EbookError::Xml(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        // Store cover image path if found
        if let Some(cover) = cover_id {
            if let Some(cover_path) = manifest_items.get(&cover) {
                self.metadata.cover_image_path = Some(cover_path.clone());
            }
        }

        self.metadata.format = Some("EPUB".to_string());
        Ok(())
    }

    fn find_opf_path(archive: &mut ZipArchive<File>) -> Result<String> {
        let container = archive.by_name("META-INF/container.xml")?;
        let mut content = String::new();
        std::io::BufReader::new(container).read_to_string(&mut content)?;

        use quick_xml::Reader;
        use quick_xml::events::Event;

        let mut reader = Reader::from_str(&content);
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(e)) | Ok(Event::Start(e)) => {
                    if e.name().as_ref() == b"rootfile" {
                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                if attr.key.as_ref() == b"full-path" {
                                    return Ok(String::from_utf8_lossy(&attr.value).to_string());
                                }
                            }
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(EbookError::Xml(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        Err(EbookError::NotFound("OPF path not found".to_string()))
    }
}

impl EbookReader for EpubHandler {
    fn read_from_file(&mut self, path: &Path) -> Result<()> {
        log::info!("Reading EPUB file: {path:?}");
        let file = File::open(path)?;
        let mut archive = ZipArchive::new(file)?;
        log::debug!("EPUB archive opened with {} files", archive.len());

        let opf_path = Self::find_opf_path(&mut archive)?;

        let mut opf_content = String::new();
        archive.by_name(&opf_path)?.read_to_string(&mut opf_content)?;

        self.parse_opf(&opf_content)?;

        // Parse spine and manifest to get ordered chapter list
        let opf_dir = opf_path.rsplit('/').skip(1).collect::<Vec<&str>>().join("/");
        let (spine_items, manifest_items) = self.parse_spine_and_manifest(&opf_content)?;

        // Read content files in spine order
        for (idx, itemref) in spine_items.iter().enumerate() {
            if let Some(href) = manifest_items.get(itemref) {
                let full_path = if opf_dir.is_empty() {
                    href.clone()
                } else {
                    format!("{opf_dir}/{href}")
                };

                if let Ok(mut file) = archive.by_name(&full_path) {
                    let mut content = String::new();
                    file.read_to_string(&mut content)?;

                    // Extract title from content
                    let title = self.extract_chapter_title(&content)
                        .unwrap_or_else(|| format!("Chapter {}", idx + 1));

                    self.chapters.push(Chapter {
                        title,
                        content: content.clone(),
                        filename: full_path.clone(),
                    });

                    self.content.push_str(&content);
                    self.content.push('\n');

                    // Add to TOC
                    self.toc.push(TocEntry {
                        id: idx as u32,
                        level: 0,
                        title: self.chapters.last().map(|c| c.title.clone()).unwrap_or_default(),
                        href: Some(full_path.clone()),
                        children: Vec::new(),
                    });
                }
            }
        }

        // Extract images
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();

            if name.ends_with(".jpg") || name.ends_with(".jpeg") ||
               name.ends_with(".png") || name.ends_with(".gif") ||
               name.ends_with(".svg") {
                let mut data = Vec::new();
                file.read_to_end(&mut data)?;
                let mime_type = crate::utils::guess_mime_type(&name);
                self.images.push(ImageData::new(name, mime_type, data));
            }
        }

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

impl EpubHandler {
    pub fn optimize_images(&mut self, options: crate::image_optimizer::OptimizationOptions) -> Result<usize> {
        use crate::image_optimizer::ImageOptimizer;
        
        let optimizer = ImageOptimizer::new(options);
        let mut total_savings = 0usize;
        
        for image in &mut self.images {
            let original_size = image.data.len();
            
            match optimizer.optimize(&image.data, &image.mime_type) {
                Ok(optimized_data) => {
                    let new_size = optimized_data.len();
                    if new_size < original_size {
                        total_savings += original_size - new_size;
                        image.data = optimized_data;
                    }
                }
                Err(_) => {
                    // Skip images that fail to optimize
                    continue;
                }
            }
        }
        
        Ok(total_savings)
    }

    fn parse_spine_and_manifest(&self, opf_content: &str) -> Result<(Vec<String>, HashMap<String, String>)> {
        use quick_xml::Reader;
        use quick_xml::events::Event;

        let mut reader = Reader::from_str(opf_content);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        let mut in_manifest = false;
        let mut in_spine = false;
        let mut manifest_items: HashMap<String, String> = HashMap::new();
        let mut spine_items: Vec<String> = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if name == "manifest" {
                        in_manifest = true;
                    } else if name == "spine" {
                        in_spine = true;
                    }

                    if in_manifest && name == "item" {
                        let mut id = String::new();
                        let mut href = String::new();
                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                let value = String::from_utf8_lossy(&attr.value).to_string();
                                if key == "id" {
                                    id = value;
                                } else if key == "href" {
                                    href = value;
                                }
                            }
                        }
                        if !id.is_empty() {
                            manifest_items.insert(id, href);
                        }
                    }

                    if in_spine && name == "itemref" {
                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                if key == "idref" {
                                    spine_items.push(String::from_utf8_lossy(&attr.value).to_string());
                                }
                            }
                        }
                    }
                }
                Ok(Event::End(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if name == "manifest" {
                        in_manifest = false;
                    } else if name == "spine" {
                        in_spine = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(EbookError::Xml(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        Ok((spine_items, manifest_items))
    }

    fn extract_chapter_title(&self, content: &str) -> Option<String> {
        use quick_xml::Reader;
        use quick_xml::events::Event;

        let mut reader = Reader::from_str(content);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        let mut in_title = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if name == "h1" || name == "h2" || name == "title" {
                        in_title = true;
                    }
                }
                Ok(Event::Text(e)) if in_title => {
                    return Some(e.unescape().unwrap_or_default().to_string());
                }
                Ok(Event::End(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if name == "h1" || name == "h2" || name == "title" {
                        in_title = false;
                    }
                }
                Ok(Event::Eof) => break,
                _ => {}
            }
            buf.clear();
        }

        None
    }
}

impl EbookWriter for EpubHandler {
    fn set_metadata(&mut self, metadata: Metadata) -> Result<()> {
        self.metadata = metadata;
        Ok(())
    }

    fn set_content(&mut self, content: &str) -> Result<()> {
        self.content = content.to_string();
        Ok(())
    }

    fn add_chapter(&mut self, title: &str, content: &str) -> Result<()> {
        let filename = format!("chapter{}.xhtml", self.chapters.len() + 1);
        self.chapters.push(Chapter {
            title: title.to_string(),
            content: content.to_string(),
            filename,
        });
        Ok(())
    }

    fn add_image(&mut self, name: &str, data: Vec<u8>) -> Result<()> {
        let mime_type = crate::utils::guess_mime_type(name);
        self.images.push(ImageData::new(name.to_string(), mime_type, data));
        Ok(())
    }

    fn write_to_file(&self, path: &Path) -> Result<()> {
        log::info!("Writing EPUB file: {:?} (version: {:?})", path, self.epub_version);
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = File::create(path)?;
        let mut zip = ZipWriter::new(file);
        log::debug!("Writing {} chapters and {} images", self.chapters.len(), self.images.len());
        let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("mimetype", FileOptions::<()>::default().compression_method(zip::CompressionMethod::Stored))?;
        zip.write_all(b"application/epub+zip")?;

        zip.start_file("META-INF/container.xml", options)?;
        zip.write_all(br#"<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#)?;

        let title = self.metadata.title.as_deref().unwrap_or("Untitled");
        let author = self.metadata.author.as_deref().unwrap_or("Unknown");
        let language = self.metadata.language.as_deref().unwrap_or("en");

        // Build manifest items list
        let mut manifest_items = String::new();
        
        // Add navigation items based on EPUB version
        if self.epub_version == EpubVersion::V3 {
            manifest_items.push_str(r#"    <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>"#);
            manifest_items.push('\n');
        }
        manifest_items.push_str(r#"    <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>"#);

        // Add chapter items to manifest
        for (idx, chapter) in self.chapters.iter().enumerate() {
            manifest_items.push_str(&format!(
                r#"
    <item id="ch{}" href="{}" media-type="application/xhtml+xml"/>"#,
                idx, chapter.filename
            ));
        }

        // Add image items to manifest
        for (idx, image) in self.images.iter().enumerate() {
            let media_type = &image.mime_type;
            manifest_items.push_str(&format!(
                r#"
    <item id="img{}" href="{}" media-type="{}"/>"#,
                idx, image.name, media_type
            ));
        }

        // Build spine items list
        let mut spine_items = String::new();
        for (idx, _chapter) in self.chapters.iter().enumerate() {
            spine_items.push_str(&format!(r#"    <itemref idref="ch{idx}"/>"#));
        }

        zip.start_file("OEBPS/content.opf", options)?;
        let version_str = match self.epub_version {
            EpubVersion::V2 => "2.0",
            EpubVersion::V3 => "3.0",
        };
        let opf = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="{}" unique-identifier="BookID">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>{}</dc:title>
    <dc:creator>{}</dc:creator>
    <dc:language>{}</dc:language>
    <dc:identifier id="BookID">urn:uuid:{}</dc:identifier>
  </metadata>
  <manifest>
{}
  </manifest>
  <spine toc="ncx">
{}
  </spine>
</package>"#, version_str, title, author, language, uuid::Uuid::new_v4(), manifest_items, spine_items);
        zip.write_all(opf.as_bytes())?;

        // Write TOC
        zip.start_file("OEBPS/toc.ncx", options)?;
        let mut ncx_content = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
  <head>
    <meta name="dtb:uid" content="{}"/>
    <meta name="dtb:depth" content="1"/>
    <meta name="dtb:totalPageCount" content="0"/>
    <meta name="dtb:maxPageNumber" content="0"/>
  </head>
  <docTitle>
    <text>{}</text>
  </docTitle>
  <navMap>"#, uuid::Uuid::new_v4(), title);

        for (idx, chapter) in self.chapters.iter().enumerate() {
            ncx_content.push_str(&format!(r#"
    <navPoint id="navPoint-{}" playOrder="{}">
      <navLabel>
        <text>{}</text>
      </navLabel>
      <content src="{}"/>
    </navPoint>"#, idx, idx + 1, chapter.title, chapter.filename));
        }

        ncx_content.push_str(r#"
  </navMap>
</ncx>"#);
        zip.write_all(ncx_content.as_bytes())?;

        // Write nav.xhtml for EPUB 3.0
        if self.epub_version == EpubVersion::V3 {
            zip.start_file("OEBPS/nav.xhtml", options)?;
            let nav_content = self.generate_nav_xhtml();
            zip.write_all(nav_content.as_bytes())?;
        }

        // Write chapters
        for chapter in &self.chapters {
            let filename = format!("OEBPS/{}", chapter.filename);
            zip.start_file(&filename, options)?;
            zip.write_all(chapter.content.as_bytes())?;
        }

        // Write images
        for image in &self.images {
            let filename = format!("OEBPS/{}", image.name);
            zip.start_file(&filename, options)?;
            zip.write_all(&image.data)?;
        }

        zip.finish()?;
        Ok(())
    }
}

impl EbookOperator for EpubHandler {
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
