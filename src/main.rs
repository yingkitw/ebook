use clap::{Parser, Subcommand};
use ebook_cli::{EbookError, Result, Converter};
use ebook_cli::formats::{EpubHandler, MobiHandler, Fb2Handler, CbzHandler, TxtHandler, PdfHandler, AzwHandler};
use ebook_cli::traits::{EbookReader, EbookWriter, EbookOperator};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ebook-cli")]
#[command(about = "A CLI tool for reading, writing, and operating on various ebook formats", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Read {
        #[arg(help = "Path to the ebook file")]
        input: PathBuf,
        
        #[arg(short, long, help = "Show metadata only")]
        metadata: bool,
        
        #[arg(short, long, help = "Extract images to directory")]
        extract_images: Option<PathBuf>,
        
        #[arg(short, long, help = "Show table of contents")]
        toc: bool,
    },
    
    Write {
        #[arg(help = "Output file path")]
        output: PathBuf,

        #[arg(short, long, help = "Title of the ebook")]
        title: Option<String>,

        #[arg(short, long, help = "Author of the ebook")]
        author: Option<String>,

        #[arg(short, long, help = "Content file (text)")]
        content: Option<PathBuf>,

        #[arg(short, long, help = "Format (epub, mobi, fb2, cbz, txt, pdf)")]
        format: String,

        #[arg(short, long, help = "Show progress during write")]
        progress: bool,
    },
    
    Convert {
        #[arg(help = "Input file path")]
        input: PathBuf,

        #[arg(help = "Output file path")]
        output: PathBuf,

        #[arg(short, long, help = "Target format")]
        format: Option<String>,

        #[arg(short, long, help = "Show progress during conversion")]
        progress: bool,
    },
    
    Info {
        #[arg(help = "Path to the ebook file")]
        input: PathBuf,
    },
    
    Validate {
        #[arg(help = "Path to the ebook file")]
        input: PathBuf,
    },
    
    Repair {
        #[arg(help = "Path to the ebook file")]
        input: PathBuf,

        #[arg(short, long, help = "Output file path (if different from input)")]
        output: Option<PathBuf>,

        #[arg(short, long, help = "Show progress during repair")]
        progress: bool,
    },
    
    Optimize {
        #[arg(help = "Path to the ebook file (EPUB or CBZ)")]
        input: PathBuf,

        #[arg(short, long, help = "Output file path (if different from input)")]
        output: Option<PathBuf>,

        #[arg(long, help = "Maximum width for images", default_value = "1920")]
        max_width: u32,

        #[arg(long, help = "Maximum height for images", default_value = "1920")]
        max_height: u32,

        #[arg(short, long, help = "JPEG quality (1-100)", default_value = "85")]
        quality: u8,

        #[arg(long, help = "Skip resizing, only compress")]
        no_resize: bool,

        #[arg(short, long, help = "Show progress during optimization")]
        progress: bool,
    },
    
    #[command(about = "Start MCP server for Model Context Protocol integration")]
    Mcp,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Read { input, metadata, extract_images, toc } => {
            handle_read(input, metadata, extract_images, toc)?;
        }
        Commands::Write { output, title, author, content, format, progress } => {
            handle_write(output, title, author, content, format, progress)?;
        }
        Commands::Convert { input, output, format, progress } => {
            handle_convert(input, output, format, progress)?;
        }
        Commands::Info { input } => {
            handle_info(input)?;
        }
        Commands::Validate { input } => {
            handle_validate(input)?;
        }
        Commands::Repair { input, output, progress } => {
            handle_repair(input, output, progress)?;
        }
        Commands::Optimize { input, output, max_width, max_height, quality, no_resize, progress } => {
            handle_optimize(input, output, max_width, max_height, quality, no_resize, progress)?;
        }
        Commands::Mcp => {
            handle_mcp().await?;
        }
    }

    Ok(())
}

fn handle_read(
    input: PathBuf,
    show_metadata: bool,
    extract_images: Option<PathBuf>,
    show_toc: bool,
) -> Result<()> {
    let format = ebook_cli::utils::detect_format(&input)?;
    
    match format.as_str() {
        "epub" => {
            let mut handler = EpubHandler::new();
            handler.read_from_file(&input)?;
            
            if show_metadata {
                let metadata = handler.get_metadata()?;
                println!("{}", serde_json::to_string_pretty(&metadata).unwrap());
            } else if show_toc {
                let toc = handler.get_toc()?;
                for entry in toc {
                    println!("{}{}", "  ".repeat(entry.level - 1), entry.title);
                }
            } else {
                let content = handler.get_content()?;
                println!("{}", content);
            }
            
            if let Some(dir) = extract_images {
                std::fs::create_dir_all(&dir)?;
                let images = handler.extract_images()?;
                for image in &images {
                    let path = dir.join(&image.name);
                    std::fs::write(path, &image.data)?;
                }
                println!("Extracted {} images to {:?}", images.len(), dir);
            }
        }
        "mobi" => {
            let mut handler = MobiHandler::new();
            handler.read_from_file(&input)?;

            if show_metadata {
                let metadata = handler.get_metadata()?;
                println!("{}", serde_json::to_string_pretty(&metadata).unwrap());
            } else {
                let content = handler.get_content()?;
                println!("{}", content);
            }
        }
        "azw" | "azw3" => {
            let mut handler = AzwHandler::new();
            handler.read_from_file(&input)?;

            if show_metadata {
                let metadata = handler.get_metadata()?;
                println!("{}", serde_json::to_string_pretty(&metadata).unwrap());
            } else {
                let content = handler.get_content()?;
                println!("{}", content);
            }
        }
        "fb2" => {
            let mut handler = Fb2Handler::new();
            handler.read_from_file(&input)?;
            
            if show_metadata {
                let metadata = handler.get_metadata()?;
                println!("{}", serde_json::to_string_pretty(&metadata).unwrap());
            } else {
                let content = handler.get_content()?;
                println!("{}", content);
            }
        }
        "cbz" => {
            let mut handler = CbzHandler::new();
            handler.read_from_file(&input)?;
            
            if show_metadata {
                let metadata = handler.get_metadata()?;
                println!("{}", serde_json::to_string_pretty(&metadata).unwrap());
            } else {
                let content = handler.get_content()?;
                println!("{}", content);
            }
            
            if let Some(dir) = extract_images {
                std::fs::create_dir_all(&dir)?;
                let images = handler.extract_images()?;
                for image in &images {
                    let path = dir.join(&image.name);
                    std::fs::write(path, &image.data)?;
                }
                println!("Extracted {} images to {:?}", images.len(), dir);
            }
        }
        "txt" => {
            let mut handler = TxtHandler::new();
            handler.read_from_file(&input)?;
            
            if show_metadata {
                let metadata = handler.get_metadata()?;
                println!("{}", serde_json::to_string_pretty(&metadata).unwrap());
            } else if show_toc {
                let toc = handler.get_toc()?;
                for entry in toc {
                    println!("{}", entry.title);
                }
            } else {
                let content = handler.get_content()?;
                println!("{}", content);
            }
        }
        "pdf" => {
            let mut handler = PdfHandler::new();
            handler.read_from_file(&input)?;
            
            if show_metadata {
                let metadata = handler.get_metadata()?;
                println!("{}", serde_json::to_string_pretty(&metadata).unwrap());
            } else {
                let content = handler.get_content()?;
                println!("{}", content);
            }
        }
        _ => return Err(EbookError::UnsupportedFormat(format)),
    }
    
    Ok(())
}

fn handle_write(
    output: PathBuf,
    title: Option<String>,
    author: Option<String>,
    content_file: Option<PathBuf>,
    format: String,
    show_progress: bool,
) -> Result<()> {
    let content = if let Some(path) = content_file {
        if show_progress {
            eprint!("Reading content from file...");
        }
        let c = std::fs::read_to_string(path)?;
        if show_progress {
            eprintln!(" Done.");
        }
        c
    } else {
        String::new()
    };

    let mut metadata = ebook_cli::Metadata::new();
    if let Some(t) = title {
        metadata.title = Some(t);
    }
    if let Some(a) = author {
        metadata.author = Some(a);
    }

    if show_progress {
        eprint!("Writing {} ebook...", format);
    }

    match format.as_str() {
        "epub" => {
            let mut handler = EpubHandler::new();
            handler.set_metadata(metadata)?;
            handler.set_content(&content)?;
            handler.write_to_file(&output)?;
        }
        "mobi" => {
            let mut handler = MobiHandler::new();
            handler.set_metadata(metadata)?;
            handler.set_content(&content)?;
            handler.write_to_file(&output)?;
        }
        "azw" | "azw3" => {
            let mut handler = AzwHandler::new();
            handler.set_metadata(metadata)?;
            handler.set_content(&content)?;
            handler.write_to_file(&output)?;
        }
        "fb2" => {
            let mut handler = Fb2Handler::new();
            handler.set_metadata(metadata)?;
            handler.set_content(&content)?;
            handler.write_to_file(&output)?;
        }
        "cbz" => {
            let mut handler = CbzHandler::new();
            handler.set_metadata(metadata)?;
            handler.write_to_file(&output)?;
        }
        "txt" => {
            let mut handler = TxtHandler::new();
            handler.set_metadata(metadata)?;
            handler.set_content(&content)?;
            handler.write_to_file(&output)?;
        }
        "pdf" => {
            let mut handler = PdfHandler::new();
            handler.set_metadata(metadata)?;
            handler.set_content(&content)?;
            handler.write_to_file(&output)?;
        }
        _ => return Err(EbookError::UnsupportedFormat(format)),
    }

    if show_progress {
        eprintln!(" Done.");
    }

    println!("Successfully wrote ebook to {:?}", output);
    Ok(())
}

fn handle_convert(input: PathBuf, output: PathBuf, target_format: Option<String>, show_progress: bool) -> Result<()> {
    let source_format = ebook_cli::utils::detect_format(&input)?;
    let target = target_format.unwrap_or_else(|| {
        ebook_cli::utils::detect_format(&output).unwrap_or_else(|_| "txt".to_string())
    });

    println!("Converting from {} to {}", source_format, target);

    if show_progress {
        let progress_name = format!("Converting {} to {}", source_format, target);
        Converter::convert_with_progress(&input, &output, &target, Some(progress_name))?;
    } else {
        Converter::convert(&input, &output, &target)?;
    }

    println!("Successfully converted to {:?}", output);
    Ok(())
}

fn handle_info(input: PathBuf) -> Result<()> {
    let format = ebook_cli::utils::detect_format(&input)?;
    
    println!("File: {:?}", input);
    println!("Format: {}", format);
    
    match format.as_str() {
        "epub" => {
            let mut handler = EpubHandler::new();
            handler.read_from_file(&input)?;
            let metadata = handler.get_metadata()?;
            println!("\nMetadata:");
            println!("{}", serde_json::to_string_pretty(&metadata).unwrap());
        }
        "mobi" => {
            let mut handler = MobiHandler::new();
            handler.read_from_file(&input)?;
            let metadata = handler.get_metadata()?;
            println!("\nMetadata:");
            println!("{}", serde_json::to_string_pretty(&metadata).unwrap());
        }
        "azw" | "azw3" => {
            let mut handler = AzwHandler::new();
            handler.read_from_file(&input)?;
            let metadata = handler.get_metadata()?;
            println!("\nMetadata:");
            println!("{}", serde_json::to_string_pretty(&metadata).unwrap());
        }
        "fb2" => {
            let mut handler = Fb2Handler::new();
            handler.read_from_file(&input)?;
            let metadata = handler.get_metadata()?;
            println!("\nMetadata:");
            println!("{}", serde_json::to_string_pretty(&metadata).unwrap());
        }
        "cbz" => {
            let mut handler = CbzHandler::new();
            handler.read_from_file(&input)?;
            let metadata = handler.get_metadata()?;
            let images = handler.extract_images()?;
            println!("\nMetadata:");
            println!("{}", serde_json::to_string_pretty(&metadata).unwrap());
            println!("\nImages: {}", images.len());
        }
        "txt" => {
            let mut handler = TxtHandler::new();
            handler.read_from_file(&input)?;
            let metadata = handler.get_metadata()?;
            let content = handler.get_content()?;
            println!("\nMetadata:");
            println!("{}", serde_json::to_string_pretty(&metadata).unwrap());
            println!("\nSize: {} characters", content.len());
        }
        "pdf" => {
            let mut handler = PdfHandler::new();
            handler.read_from_file(&input)?;
            let metadata = handler.get_metadata()?;
            println!("\nMetadata:");
            println!("{}", serde_json::to_string_pretty(&metadata).unwrap());
        }
        _ => return Err(EbookError::UnsupportedFormat(format)),
    }
    
    Ok(())
}

fn handle_validate(input: PathBuf) -> Result<()> {
    let format = ebook_cli::utils::detect_format(&input)?;
    
    let is_valid = match format.as_str() {
        "epub" => {
            let mut handler = EpubHandler::new();
            handler.read_from_file(&input)?;
            handler.validate()?
        }
        "mobi" => {
            let mut handler = MobiHandler::new();
            handler.read_from_file(&input)?;
            handler.validate()?
        }
        "azw" | "azw3" => {
            let mut handler = AzwHandler::new();
            handler.read_from_file(&input)?;
            handler.validate()?
        }
        "fb2" => {
            let mut handler = Fb2Handler::new();
            handler.read_from_file(&input)?;
            handler.validate()?
        }
        "cbz" => {
            let mut handler = CbzHandler::new();
            handler.read_from_file(&input)?;
            handler.validate()?
        }
        "txt" => {
            let mut handler = TxtHandler::new();
            handler.read_from_file(&input)?;
            handler.validate()?
        }
        "pdf" => {
            let mut handler = PdfHandler::new();
            handler.read_from_file(&input)?;
            handler.validate()?
        }
        _ => return Err(EbookError::UnsupportedFormat(format)),
    };
    
    if is_valid {
        println!("✓ File is valid");
    } else {
        println!("✗ File has validation issues");
    }
    
    Ok(())
}

fn handle_repair(input: PathBuf, output: Option<PathBuf>, show_progress: bool) -> Result<()> {
    let format = ebook_cli::utils::detect_format(&input)?;
    let output_path = output.unwrap_or_else(|| input.clone());

    if show_progress {
        eprint!("Reading {} file...", format);
    }

    match format.as_str() {
        "epub" => {
            let mut handler = EpubHandler::new();
            handler.read_from_file(&input)?;
            if show_progress {
                eprintln!(" Done.");
                eprint!("Repairing...");
            }
            handler.repair()?;
            if show_progress {
                eprintln!(" Done.");
                eprint!("Writing...");
            }
            handler.write_to_file(&output_path)?;
        }
        "mobi" => {
            let mut handler = MobiHandler::new();
            handler.read_from_file(&input)?;
            if show_progress {
                eprintln!(" Done.");
                eprint!("Repairing...");
            }
            handler.repair()?;
            if show_progress {
                eprintln!(" Done.");
                eprint!("Writing...");
            }
            handler.write_to_file(&output_path)?;
        }
        "azw" | "azw3" => {
            let mut handler = AzwHandler::new();
            handler.read_from_file(&input)?;
            if show_progress {
                eprintln!(" Done.");
                eprint!("Repairing...");
            }
            handler.repair()?;
            if show_progress {
                eprintln!(" Done.");
                eprint!("Writing...");
            }
            handler.write_to_file(&output_path)?;
        }
        "fb2" => {
            let mut handler = Fb2Handler::new();
            handler.read_from_file(&input)?;
            if show_progress {
                eprintln!(" Done.");
                eprint!("Repairing...");
            }
            handler.repair()?;
            if show_progress {
                eprintln!(" Done.");
                eprint!("Writing...");
            }
            handler.write_to_file(&output_path)?;
        }
        "cbz" => {
            let mut handler = CbzHandler::new();
            handler.read_from_file(&input)?;
            if show_progress {
                eprintln!(" Done.");
                eprint!("Repairing...");
            }
            handler.repair()?;
            if show_progress {
                eprintln!(" Done.");
                eprint!("Writing...");
            }
            handler.write_to_file(&output_path)?;
        }
        "txt" => {
            let mut handler = TxtHandler::new();
            handler.read_from_file(&input)?;
            if show_progress {
                eprintln!(" Done.");
                eprint!("Repairing...");
            }
            handler.repair()?;
            if show_progress {
                eprintln!(" Done.");
                eprint!("Writing...");
            }
            handler.write_to_file(&output_path)?;
        }
        "pdf" => {
            let mut handler = PdfHandler::new();
            handler.read_from_file(&input)?;
            if show_progress {
                eprintln!(" Done.");
                eprint!("Repairing...");
            }
            handler.repair()?;
            if show_progress {
                eprintln!(" Done.");
                eprint!("Writing...");
            }
            handler.write_to_file(&output_path)?;
        }
        _ => return Err(EbookError::UnsupportedFormat(format)),
    }

    if show_progress {
        eprintln!(" Done.");
    }

    println!("Successfully repaired and saved to {:?}", output_path);
    Ok(())
}

fn handle_optimize(
    input: PathBuf,
    output: Option<PathBuf>,
    max_width: u32,
    max_height: u32,
    quality: u8,
    no_resize: bool,
    show_progress: bool,
) -> Result<()> {
    use ebook_cli::image_optimizer::OptimizationOptions;
    
    let format = ebook_cli::utils::detect_format(&input)?;
    let output_path = output.unwrap_or_else(|| input.clone());

    if show_progress {
        eprint!("Reading {}...", input.display());
    }

    let mut options = OptimizationOptions::default()
        .with_quality(quality);
    
    if no_resize {
        options = options.no_resize();
    } else {
        options = options.with_max_dimensions(max_width, max_height);
    }

    match format.as_str() {
        "epub" => {
            let mut handler = EpubHandler::new();
            handler.read_from_file(&input)?;
            
            if show_progress {
                eprintln!(" Done.");
                eprint!("Optimizing images...");
            }
            
            let savings = handler.optimize_images(options)?;
            
            if show_progress {
                eprintln!(" Done.");
                eprint!("Writing optimized EPUB...");
            }
            
            handler.write_to_file(&output_path)?;
            
            if show_progress {
                eprintln!(" Done.");
            }
            
            println!("Successfully optimized EPUB");
            println!("Saved {} bytes ({:.1}% reduction)", 
                savings, 
                if savings > 0 { (savings as f64 / 1024.0 / 1024.0) } else { 0.0 }
            );
        }
        "cbz" => {
            let mut handler = CbzHandler::new();
            handler.read_from_file(&input)?;
            
            if show_progress {
                eprintln!(" Done.");
                eprint!("Optimizing images...");
            }
            
            let savings = handler.optimize_images(options)?;
            
            if show_progress {
                eprintln!(" Done.");
                eprint!("Writing optimized CBZ...");
            }
            
            handler.write_to_file(&output_path)?;
            
            if show_progress {
                eprintln!(" Done.");
            }
            
            println!("Successfully optimized CBZ");
            println!("Saved {} bytes ({:.1} MB reduction)", 
                savings,
                savings as f64 / 1024.0 / 1024.0
            );
        }
        _ => {
            return Err(EbookError::UnsupportedFormat(
                format!("Image optimization only supports EPUB and CBZ formats, got: {}", format)
            ));
        }
    }

    println!("Output saved to {:?}", output_path);
    Ok(())
}

async fn handle_mcp() -> Result<()> {
    use ebook_cli::mcp::McpServer;
    
    eprintln!("Starting MCP server...");
    let server = McpServer::new();
    server.run().await.map_err(|e| EbookError::Parse(e.to_string()))?;
    
    Ok(())
}
