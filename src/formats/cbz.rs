use crate::{EbookError, Metadata, Result};
use crate::traits::{EbookReader, EbookWriter, EbookOperator, TocEntry, ImageData};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use zip::ZipArchive;
use zip::write::{ZipWriter, FileOptions};

mod comic_info;
use comic_info::ComicInfo;

#[derive(Default)]
pub struct CbzHandler {
    metadata: Metadata,
    images: Vec<ImageData>,
    comic_info: Option<ComicInfo>,
}

impl CbzHandler {
    pub fn new() -> Self {
        Self::default()
    }

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
}

impl EbookReader for CbzHandler {
    fn read_from_file(&mut self, path: &Path) -> Result<()> {
        let file = File::open(path)?;
        let mut archive = ZipArchive::new(file)?;

        // Try to read ComicInfo.xml first
        if let Ok(mut comic_info_file) = archive.by_name("ComicInfo.xml") {
            let mut xml_content = String::new();
            comic_info_file.read_to_string(&mut xml_content)?;
            
            if let Ok(comic_info) = ComicInfo::parse_xml(&xml_content) {
                self.metadata = comic_info.to_metadata();
                self.comic_info = Some(comic_info);
            }
        }

        // Fallback to filename if no title from ComicInfo
        if self.metadata.title.is_none() {
            self.metadata.title = path
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string());
        }
        self.metadata.format = Some("CBZ".to_string());

        // Extract all images
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();
            
            // Skip ComicInfo.xml
            if name == "ComicInfo.xml" {
                continue;
            }
            
            if name.ends_with(".jpg") || name.ends_with(".jpeg") || 
               name.ends_with(".png") || name.ends_with(".gif") ||
               name.ends_with(".webp") {
                let mut data = Vec::new();
                file.read_to_end(&mut data)?;
                let mime_type = crate::utils::guess_mime_type(&name);
                self.images.push(ImageData::new(name, mime_type, data));
            }
        }

        self.images.sort_by(|a, b| a.name.cmp(&b.name));
        
        // Update page count in comic_info if present
        if let Some(ref mut comic_info) = self.comic_info {
            comic_info.page_count = Some(self.images.len() as u32);
        }
        
        Ok(())
    }

    fn get_metadata(&self) -> Result<Metadata> {
        Ok(self.metadata.clone())
    }

    fn get_content(&self) -> Result<String> {
        Ok(format!("CBZ archive with {} images", self.images.len()))
    }

    fn get_toc(&self) -> Result<Vec<TocEntry>> {
        Ok(Vec::new())
    }

    fn extract_images(&self) -> Result<Vec<ImageData>> {
        Ok(self.images.clone())
    }
}

impl EbookWriter for CbzHandler {
    fn set_metadata(&mut self, metadata: Metadata) -> Result<()> {
        self.metadata = metadata;
        Ok(())
    }

    fn set_content(&mut self, _content: &str) -> Result<()> {
        Ok(())
    }

    fn add_chapter(&mut self, _title: &str, _content: &str) -> Result<()> {
        Ok(())
    }

    fn add_image(&mut self, name: &str, data: Vec<u8>) -> Result<()> {
        let mime_type = crate::utils::guess_mime_type(name);
        self.images.push(ImageData::new(name.to_string(), mime_type, data));
        Ok(())
    }

    fn write_to_file(&self, path: &Path) -> Result<()> {
        let file = File::create(path)?;
        let mut zip = ZipWriter::new(file);
        let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Deflated);

        // Generate and write ComicInfo.xml
        let mut comic_info = if let Some(ref ci) = self.comic_info {
            ci.clone()
        } else {
            ComicInfo::from_metadata(&self.metadata)
        };
        
        // Update page count
        comic_info.page_count = Some(self.images.len() as u32);
        
        let xml_content = comic_info.to_xml()?;
        zip.start_file("ComicInfo.xml", options)?;
        zip.write_all(xml_content.as_bytes())?;

        // Write all images
        for image in &self.images {
            zip.start_file(&image.name, options)?;
            zip.write_all(&image.data)?;
        }

        zip.finish()?;
        Ok(())
    }
}

impl EbookOperator for CbzHandler {
    fn convert_to(&self, _target_format: &str, _output_path: &Path) -> Result<()> {
        Err(EbookError::NotSupported("Conversion not yet implemented".to_string()))
    }

    fn validate(&self) -> Result<bool> {
        Ok(!self.images.is_empty())
    }

    fn repair(&mut self) -> Result<()> {
        if self.metadata.title.is_none() {
            self.metadata.title = Some("Untitled Comic".to_string());
        }
        Ok(())
    }
}
