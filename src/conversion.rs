use crate::{EbookError, Result, Progress};
use crate::traits::{EbookReader, EbookWriter};
use crate::formats::{EpubHandler, TxtHandler, MobiHandler, Fb2Handler, PdfHandler};
use std::path::Path;

/// Conversion utility for converting between ebook formats
pub struct Converter {
    // Placeholder for future conversion options
}

impl Converter {
    pub fn new() -> Self {
        Self {}
    }

    /// Convert an ebook from one format to another
    pub fn convert(
        input_path: &Path,
        output_path: &Path,
        target_format: &str,
    ) -> Result<()> {
        Self::convert_with_progress(input_path, output_path, target_format, None)
    }

    /// Convert an ebook with optional progress reporting
    pub fn convert_with_progress(
        input_path: &Path,
        output_path: &Path,
        target_format: &str,
        progress_name: Option<String>,
    ) -> Result<()> {
        let input_format = crate::utils::detect_format(input_path)?;
        let progress = progress_name.map(|name| Progress::new(name, 3));

        if let Some(ref p) = progress {
            p.increment(0);
            p.print_with_message("Reading input file");
        }

        let result = match (input_format.as_str(), target_format) {
            ("txt", "epub") => {
                if let Some(ref p) = progress { p.increment(1); p.print_with_message("Converting to EPUB"); }
                let r = Self::txt_to_epub(input_path, output_path);
                if let Some(ref p) = progress { p.increment(1); p.print_with_message("Writing EPUB"); }
                r
            }
            ("txt", "pdf") => {
                if let Some(ref p) = progress { p.increment(1); p.print_with_message("Converting to PDF"); }
                let r = Self::txt_to_pdf(input_path, output_path);
                if let Some(ref p) = progress { p.increment(1); p.print_with_message("Writing PDF"); }
                r
            }
            ("txt", "mobi") => {
                if let Some(ref p) = progress { p.increment(1); p.print_with_message("Converting to MOBI"); }
                let r = Self::txt_to_mobi(input_path, output_path);
                if let Some(ref p) = progress { p.increment(1); p.print_with_message("Writing MOBI"); }
                r
            }
            ("epub", "txt") => {
                if let Some(ref p) = progress { p.increment(1); p.print_with_message("Converting to TXT"); }
                let r = Self::epub_to_txt(input_path, output_path);
                if let Some(ref p) = progress { p.increment(1); p.print_with_message("Writing TXT"); }
                r
            }
            ("epub", "pdf") => {
                if let Some(ref p) = progress { p.increment(1); p.print_with_message("Converting EPUB to PDF"); }
                let r = Self::epub_to_pdf(input_path, output_path);
                if let Some(ref p) = progress { p.increment(1); p.print_with_message("Writing PDF"); }
                r
            }
            ("mobi", "txt") => {
                if let Some(ref p) = progress { p.increment(1); p.print_with_message("Converting MOBI to TXT"); }
                let r = Self::mobi_to_txt(input_path, output_path);
                if let Some(ref p) = progress { p.increment(1); p.print_with_message("Writing TXT"); }
                r
            }
            ("fb2", "txt") => {
                if let Some(ref p) = progress { p.increment(1); p.print_with_message("Converting FB2 to TXT"); }
                let r = Self::fb2_to_txt(input_path, output_path);
                if let Some(ref p) = progress { p.increment(1); p.print_with_message("Writing TXT"); }
                r
            }
            ("pdf", "txt") => {
                if let Some(ref p) = progress { p.increment(1); p.print_with_message("Converting PDF to TXT"); }
                let r = Self::pdf_to_txt(input_path, output_path);
                if let Some(ref p) = progress { p.increment(1); p.print_with_message("Writing TXT"); }
                r
            }
            ("txt", "fb2") => {
                if let Some(ref p) = progress { p.increment(1); p.print_with_message("Converting to FB2"); }
                let r = Self::txt_to_fb2(input_path, output_path);
                if let Some(ref p) = progress { p.increment(1); p.print_with_message("Writing FB2"); }
                r
            }
            _ => Err(EbookError::NotSupported(format!(
                "Conversion from {input_format} to {target_format} is not supported"
            ))),
        };

        if let Some(ref p) = progress {
            p.finish();
        }

        result
    }

    fn txt_to_epub(input_path: &Path, output_path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut txt_handler = TxtHandler::new();
        txt_handler.read_from_file(input_path)?;

        let content = txt_handler.get_content()?;
        let metadata = txt_handler.get_metadata()?;

        let mut epub_handler = EpubHandler::new();
        epub_handler.set_metadata(metadata)?;
        epub_handler.set_content(&content)?;

        // Split content into chapters
        let chapters: Vec<&str> = content.split("\n\n---\n\n")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if chapters.is_empty() {
            // If no chapter markers, treat entire content as one chapter
            epub_handler.add_chapter("Chapter 1", &content)?;
        } else {
            for (idx, chapter) in chapters.iter().enumerate() {
                epub_handler.add_chapter(&format!("Chapter {}", idx + 1), chapter)?;
            }
        }

        epub_handler.write_to_file(output_path)?;
        Ok(())
    }

    fn txt_to_pdf(input_path: &Path, output_path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut txt_handler = TxtHandler::new();
        txt_handler.read_from_file(input_path)?;

        let content = txt_handler.get_content()?;
        let metadata = txt_handler.get_metadata()?;

        let mut pdf_handler = PdfHandler::new();
        pdf_handler.set_metadata(metadata)?;
        pdf_handler.set_content(&content)?;
        pdf_handler.write_to_file(output_path)?;
        Ok(())
    }

    fn txt_to_mobi(input_path: &Path, output_path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut txt_handler = TxtHandler::new();
        txt_handler.read_from_file(input_path)?;

        let content = txt_handler.get_content()?;
        let metadata = txt_handler.get_metadata()?;

        let mut mobi_handler = MobiHandler::new();
        mobi_handler.set_metadata(metadata)?;
        mobi_handler.set_content(&content)?;
        mobi_handler.write_to_file(output_path)?;
        Ok(())
    }

    fn epub_to_txt(input_path: &Path, output_path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut epub_handler = EpubHandler::new();
        epub_handler.read_from_file(input_path)?;

        let content = epub_handler.get_content()?;
        let metadata = epub_handler.get_metadata()?;

        let mut txt_handler = TxtHandler::new();
        txt_handler.set_metadata(metadata)?;
        txt_handler.set_content(&content)?;
        txt_handler.write_to_file(output_path)?;
        Ok(())
    }

    fn epub_to_pdf(input_path: &Path, output_path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut epub_handler = EpubHandler::new();
        epub_handler.read_from_file(input_path)?;

        let content = epub_handler.get_content()?;
        let metadata = epub_handler.get_metadata()?;

        let mut pdf_handler = PdfHandler::new();
        pdf_handler.set_metadata(metadata)?;
        pdf_handler.set_content(&content)?;
        pdf_handler.write_to_file(output_path)?;
        Ok(())
    }

    fn mobi_to_txt(input_path: &Path, output_path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut mobi_handler = MobiHandler::new();
        mobi_handler.read_from_file(input_path)?;

        let content = mobi_handler.get_content()?;
        let metadata = mobi_handler.get_metadata()?;

        let mut txt_handler = TxtHandler::new();
        txt_handler.set_metadata(metadata)?;
        txt_handler.set_content(&content)?;
        txt_handler.write_to_file(output_path)?;
        Ok(())
    }

    fn fb2_to_txt(input_path: &Path, output_path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut fb2_handler = Fb2Handler::new();
        fb2_handler.read_from_file(input_path)?;

        let content = fb2_handler.get_content()?;
        let metadata = fb2_handler.get_metadata()?;

        let mut txt_handler = TxtHandler::new();
        txt_handler.set_metadata(metadata)?;
        txt_handler.set_content(&content)?;
        txt_handler.write_to_file(output_path)?;
        Ok(())
    }

    fn pdf_to_txt(input_path: &Path, output_path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut pdf_handler = PdfHandler::new();
        pdf_handler.read_from_file(input_path)?;

        let content = pdf_handler.get_content()?;
        let metadata = pdf_handler.get_metadata()?;

        let mut txt_handler = TxtHandler::new();
        txt_handler.set_metadata(metadata)?;
        txt_handler.set_content(&content)?;
        txt_handler.write_to_file(output_path)?;
        Ok(())
    }

    fn txt_to_fb2(input_path: &Path, output_path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut txt_handler = TxtHandler::new();
        txt_handler.read_from_file(input_path)?;

        let content = txt_handler.get_content()?;
        let metadata = txt_handler.get_metadata()?;

        let mut fb2_handler = Fb2Handler::new();
        fb2_handler.set_metadata(metadata)?;
        fb2_handler.set_content(&content)?;
        fb2_handler.write_to_file(output_path)?;
        Ok(())
    }
}

impl Default for Converter {
    fn default() -> Self {
        Self::new()
    }
}
