use crate::{Result, EbookError};
use image::{DynamicImage, ImageFormat, ImageReader, GenericImageView};
use std::io::Cursor;

#[derive(Debug, Clone, Copy)]
pub struct OptimizationOptions {
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub quality: u8,
    pub preserve_aspect_ratio: bool,
}

impl Default for OptimizationOptions {
    fn default() -> Self {
        Self {
            max_width: Some(1920),
            max_height: Some(1920),
            quality: 85,
            preserve_aspect_ratio: true,
        }
    }
}

impl OptimizationOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_dimensions(mut self, width: u32, height: u32) -> Self {
        self.max_width = Some(width);
        self.max_height = Some(height);
        self
    }

    pub fn with_quality(mut self, quality: u8) -> Self {
        self.quality = quality.min(100);
        self
    }

    pub fn no_resize(mut self) -> Self {
        self.max_width = None;
        self.max_height = None;
        self
    }
}

pub struct ImageOptimizer {
    options: OptimizationOptions,
}

impl ImageOptimizer {
    pub fn new(options: OptimizationOptions) -> Self {
        Self { options }
    }

    pub fn with_default_options() -> Self {
        Self::new(OptimizationOptions::default())
    }

    pub fn optimize(&self, image_data: &[u8], mime_type: &str) -> Result<Vec<u8>> {
        // Load the image
        let img = ImageReader::new(Cursor::new(image_data))
            .with_guessed_format()
            .map_err(|e| EbookError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?
            .decode()
            .map_err(|e| EbookError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;

        // Resize if needed
        let resized_img = self.resize_if_needed(img)?;

        // Encode with compression
        self.encode_image(resized_img, mime_type)
    }

    fn resize_if_needed(&self, img: DynamicImage) -> Result<DynamicImage> {
        let (width, height) = img.dimensions();
        
        let max_width = self.options.max_width.unwrap_or(u32::MAX);
        let max_height = self.options.max_height.unwrap_or(u32::MAX);

        if width <= max_width && height <= max_height {
            return Ok(img);
        }

        let (new_width, new_height) = if self.options.preserve_aspect_ratio {
            self.calculate_aspect_ratio_dimensions(width, height, max_width, max_height)
        } else {
            (max_width.min(width), max_height.min(height))
        };

        Ok(img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3))
    }

    fn calculate_aspect_ratio_dimensions(
        &self,
        width: u32,
        height: u32,
        max_width: u32,
        max_height: u32,
    ) -> (u32, u32) {
        let width_ratio = max_width as f64 / width as f64;
        let height_ratio = max_height as f64 / height as f64;
        let ratio = width_ratio.min(height_ratio);

        if ratio >= 1.0 {
            (width, height)
        } else {
            ((width as f64 * ratio) as u32, (height as f64 * ratio) as u32)
        }
    }

    fn encode_image(&self, img: DynamicImage, mime_type: &str) -> Result<Vec<u8>> {
        let mut buffer = Cursor::new(Vec::new());

        match mime_type {
            "image/jpeg" | "image/jpg" => {
                let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                    &mut buffer,
                    self.options.quality,
                );
                img.write_with_encoder(encoder)
                    .map_err(|e| EbookError::Io(std::io::Error::other(e)))?;
            }
            "image/png" => {
                let encoder = image::codecs::png::PngEncoder::new(&mut buffer);
                img.write_with_encoder(encoder)
                    .map_err(|e| EbookError::Io(std::io::Error::other(e)))?;
            }
            "image/webp" => {
                // WebP encoding with quality
                img.write_to(&mut buffer, ImageFormat::WebP)
                    .map_err(|e| EbookError::Io(std::io::Error::other(e)))?;
            }
            _ => {
                // Default to PNG for unknown formats
                let encoder = image::codecs::png::PngEncoder::new(&mut buffer);
                img.write_with_encoder(encoder)
                    .map_err(|e| EbookError::Io(std::io::Error::other(e)))?;
            }
        }

        Ok(buffer.into_inner())
    }

    pub fn calculate_savings(&self, original_size: usize, optimized_size: usize) -> f64 {
        if original_size == 0 {
            return 0.0;
        }
        ((original_size - optimized_size) as f64 / original_size as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image() -> Vec<u8> {
        // Create a simple 100x100 red image
        let img = DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
            100,
            100,
            image::Rgb([255, 0, 0]),
        ));
        
        let mut buffer = Cursor::new(Vec::new());
        img.write_to(&mut buffer, ImageFormat::Png).unwrap();
        buffer.into_inner()
    }

    #[test]
    fn test_image_optimization() {
        let optimizer = ImageOptimizer::with_default_options();
        let test_image = create_test_image();
        
        let result = optimizer.optimize(&test_image, "image/png");
        assert!(result.is_ok());
        
        let optimized = result.unwrap();
        assert!(!optimized.is_empty());
    }

    #[test]
    fn test_resize_large_image() {
        let options = OptimizationOptions::default().with_max_dimensions(50, 50);
        let optimizer = ImageOptimizer::new(options);
        
        let test_image = create_test_image();
        let result = optimizer.optimize(&test_image, "image/png");
        assert!(result.is_ok());
    }

    #[test]
    fn test_quality_setting() {
        let options = OptimizationOptions::default().with_quality(50);
        let optimizer = ImageOptimizer::new(options);
        
        let test_image = create_test_image();
        let result = optimizer.optimize(&test_image, "image/jpeg");
        assert!(result.is_ok());
    }

    #[test]
    fn test_savings_calculation() {
        let optimizer = ImageOptimizer::with_default_options();
        let savings = optimizer.calculate_savings(1000, 500);
        assert_eq!(savings, 50.0);
    }
}
