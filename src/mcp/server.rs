use crate::mcp::types::*;
use crate::formats::{AzwHandler, CbzHandler, EpubHandler, Fb2Handler, MobiHandler, PdfHandler, TxtHandler};
use crate::traits::{EbookReader, EbookWriter, EbookOperator};
use crate::{Metadata, Converter};
use serde_json::json;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

pub struct McpServer;

impl McpServer {
    pub fn new() -> Self {
        Self
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let stdin = std::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut stdout = std::io::stdout();

        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    if line.trim().is_empty() {
                        continue;
                    }

                    let response = self.handle_request(&line).await;
                    let response_str = serde_json::to_string(&response)?;
                    writeln!(stdout, "{response_str}")?;
                    stdout.flush()?;
                }
                Err(e) => {
                    eprintln!("Error reading input: {e}");
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_request(&self, request_str: &str) -> JsonRpcResponse {
        let request: JsonRpcRequest = match serde_json::from_str(request_str) {
            Ok(req) => req,
            Err(e) => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {e}"),
                        data: None,
                    }),
                };
            }
        };

        match request.method.as_str() {
            "initialize" => self.handle_initialize(request.id),
            "tools/list" => self.handle_tools_list(request.id),
            "tools/call" => self.handle_tools_call(request.id, request.params).await,
            _ => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: format!("Method not found: {}", request.method),
                    data: None,
                }),
            },
        }
    }

    fn handle_initialize(&self, id: Option<serde_json::Value>) -> JsonRpcResponse {
        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: Some(false),
                }),
            },
            server_info: ServerInfo {
                name: "ebook-mcp-server".to_string(),
                version: "0.1.0".to_string(),
            },
        };

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(serde_json::to_value(result).unwrap()),
            error: None,
        }
    }

    fn handle_tools_list(&self, id: Option<serde_json::Value>) -> JsonRpcResponse {
        let tools = vec![
            Tool {
                name: "read_ebook".to_string(),
                description: "Read an ebook file and extract content/metadata/TOC (supports: epub, pdf, txt, mobi, fb2, azw, cbz)".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the ebook file"
                        },
                        "extract_metadata": {
                            "type": "boolean",
                            "description": "Whether to extract metadata only",
                            "default": false
                        },
                        "extract_toc": {
                            "type": "boolean",
                            "description": "Whether to extract table of contents",
                            "default": false
                        }
                    },
                    "required": ["path"]
                }),
            },
            Tool {
                name: "write_ebook".to_string(),
                description: "Create a new ebook file from text content (supports: epub, pdf, txt, mobi, fb2, azw)".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Output path for the ebook file"
                        },
                        "format": {
                            "type": "string",
                            "description": "Format of the ebook (epub, mobi, fb2, azw, txt, pdf)",
                            "enum": ["epub", "mobi", "fb2", "azw", "txt", "pdf"]
                        },
                        "title": {
                            "type": "string",
                            "description": "Title of the ebook"
                        },
                        "author": {
                            "type": "string",
                            "description": "Author of the ebook"
                        },
                        "content": {
                            "type": "string",
                            "description": "Text content of the ebook"
                        }
                    },
                    "required": ["path", "format"]
                }),
            },
            Tool {
                name: "extract_images".to_string(),
                description: "Extract images from an ebook file".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the ebook file"
                        }
                    },
                    "required": ["path"]
                }),
            },
            Tool {
                name: "validate_ebook".to_string(),
                description: "Validate an ebook file structure and metadata (supports: epub, pdf, txt, mobi, fb2, azw, cbz)".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the ebook file"
                        }
                    },
                    "required": ["path"]
                }),
            },
            Tool {
                name: "get_ebook_info".to_string(),
                description: "Get detailed information about an ebook file (supports: epub, pdf, txt, mobi, fb2, azw, cbz)".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the ebook file"
                        }
                    },
                    "required": ["path"]
                }),
            },
            Tool {
                name: "convert_ebook".to_string(),
                description: "Convert an ebook from one format to another".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "input_path": {
                            "type": "string",
                            "description": "Path to the input ebook file"
                        },
                        "output_path": {
                            "type": "string",
                            "description": "Path for the output ebook file"
                        },
                        "target_format": {
                            "type": "string",
                            "description": "Target format (Converter supports: epub, mobi, fb2, pdf, txt)",
                            "enum": ["epub", "mobi", "fb2", "pdf", "txt"]
                        }
                    },
                    "required": ["input_path", "output_path", "target_format"]
                }),
            },
            Tool {
                name: "optimize_images".to_string(),
                description: "Optimize images in EPUB or CBZ files by resizing and compressing them".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "input_path": {
                            "type": "string",
                            "description": "Path to the ebook file (EPUB or CBZ)"
                        },
                        "output_path": {
                            "type": "string",
                            "description": "Path for the optimized output file (optional, defaults to input path)"
                        },
                        "max_width": {
                            "type": "integer",
                            "description": "Maximum width for images in pixels",
                            "default": 1920
                        },
                        "max_height": {
                            "type": "integer",
                            "description": "Maximum height for images in pixels",
                            "default": 1920
                        },
                        "quality": {
                            "type": "integer",
                            "description": "JPEG quality (1-100)",
                            "default": 85,
                            "minimum": 1,
                            "maximum": 100
                        },
                        "no_resize": {
                            "type": "boolean",
                            "description": "Skip resizing, only compress",
                            "default": false
                        }
                    },
                    "required": ["input_path"]
                }),
            },
        ];

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({ "tools": tools })),
            error: None,
        }
    }

    async fn handle_tools_call(
        &self,
        id: Option<serde_json::Value>,
        params: Option<serde_json::Value>,
    ) -> JsonRpcResponse {
        let params: CallToolParams = match params {
            Some(p) => match serde_json::from_value(p) {
                Ok(params) => params,
                Err(e) => {
                    return JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32602,
                            message: format!("Invalid params: {e}"),
                            data: None,
                        }),
                    };
                }
            },
            None => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Missing params".to_string(),
                        data: None,
                    }),
                };
            }
        };

        let result = match params.name.as_str() {
            "read_ebook" => self.tool_read_ebook(params.arguments).await,
            "write_ebook" => self.tool_write_ebook(params.arguments).await,
            "extract_images" => self.tool_extract_images(params.arguments).await,
            "validate_ebook" => self.tool_validate_ebook(params.arguments).await,
            "get_ebook_info" => self.tool_get_ebook_info(params.arguments).await,
            "convert_ebook" => self.tool_convert_ebook(params.arguments).await,
            "optimize_images" => self.tool_optimize_images(params.arguments).await,
            _ => Err(format!("Unknown tool: {}", params.name)),
        };

        match result {
            Ok(tool_result) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(serde_json::to_value(tool_result).unwrap()),
                error: None,
            },
            Err(e) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(serde_json::to_value(ToolResult {
                    content: vec![ToolContent::Text {
                        text: format!("Error: {e}"),
                    }],
                    is_error: Some(true),
                }).unwrap()),
                error: None,
            },
        }
    }

    async fn tool_read_ebook(
        &self,
        args: std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<ToolResult, String> {
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'path' argument")?;
        let extract_metadata = args
            .get("extract_metadata")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let extract_toc = args
            .get("extract_toc")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let path_buf = PathBuf::from(path);
        let format = crate::utils::detect_format(&path_buf)
            .map_err(|e| format!("Failed to detect format: {e}"))?;

        let text = match format.as_str() {
            "epub" => {
                let mut handler = EpubHandler::new();
                handler.read_from_file(&path_buf)
                    .map_err(|e| format!("Failed to read EPUB: {e}"))?;

                if extract_metadata {
                    let metadata = handler.get_metadata()
                        .map_err(|e| format!("Failed to get metadata: {e}"))?;
                    serde_json::to_string_pretty(&metadata).unwrap()
                } else if extract_toc {
                    let toc = handler.get_toc()
                        .map_err(|e| format!("Failed to get TOC: {e}"))?;
                    format!("Table of Contents:\n{}", 
                        toc.iter().map(|e| format!("{}{}", "  ".repeat(e.level - 1), e.title))
                            .collect::<Vec<_>>().join("\n"))
                } else {
                    handler.get_content()
                        .map_err(|e| format!("Failed to get content: {e}"))?
                }
            }
            "cbz" => {
                let mut handler = CbzHandler::new();
                handler.read_from_file(&path_buf)
                    .map_err(|e| format!("Failed to read CBZ: {e}"))?;

                if extract_metadata {
                    let metadata = handler.get_metadata()
                        .map_err(|e| format!("Failed to get metadata: {e}"))?;
                    serde_json::to_string_pretty(&metadata).unwrap()
                } else if extract_toc {
                    let toc = handler.get_toc()
                        .map_err(|e| format!("Failed to get TOC: {e}"))?;
                    format!(
                        "Table of Contents:\n{}",
                        toc.iter()
                            .map(|e| format!("{}{}", "  ".repeat(e.level.saturating_sub(1)), e.title))
                            .collect::<Vec<_>>()
                            .join("\n")
                    )
                } else {
                    handler.get_content()
                        .map_err(|e| format!("Failed to get content: {e}"))?
                }
            }
            "txt" => {
                let mut handler = TxtHandler::new();
                handler.read_from_file(&path_buf)
                    .map_err(|e| format!("Failed to read TXT: {e}"))?;

                if extract_metadata {
                    let metadata = handler.get_metadata()
                        .map_err(|e| format!("Failed to get metadata: {e}"))?;
                    serde_json::to_string_pretty(&metadata).unwrap()
                } else if extract_toc {
                    let toc = handler.get_toc()
                        .map_err(|e| format!("Failed to get TOC: {e}"))?;
                    format!("Table of Contents:\n{}", 
                        toc.iter().map(|e| e.title.clone()).collect::<Vec<_>>().join("\n"))
                } else {
                    handler.get_content()
                        .map_err(|e| format!("Failed to get content: {e}"))?
                }
            }
            "pdf" => {
                let mut handler = PdfHandler::new();
                handler.read_from_file(&path_buf)
                    .map_err(|e| format!("Failed to read PDF: {e}"))?;

                if extract_metadata {
                    let metadata = handler.get_metadata()
                        .map_err(|e| format!("Failed to get metadata: {e}"))?;
                    serde_json::to_string_pretty(&metadata).unwrap()
                } else {
                    handler.get_content()
                        .map_err(|e| format!("Failed to get content: {e}"))?
                }
            }
            "mobi" => {
                let mut handler = MobiHandler::new();
                handler.read_from_file(&path_buf)
                    .map_err(|e| format!("Failed to read MOBI: {e}"))?;

                if extract_metadata {
                    let metadata = handler.get_metadata()
                        .map_err(|e| format!("Failed to get metadata: {e}"))?;
                    serde_json::to_string_pretty(&metadata).unwrap()
                } else if extract_toc {
                    let toc = handler.get_toc()
                        .map_err(|e| format!("Failed to get TOC: {e}"))?;
                    format!(
                        "Table of Contents:\n{}",
                        toc.iter()
                            .map(|e| format!("{}{}", "  ".repeat(e.level.saturating_sub(1)), e.title))
                            .collect::<Vec<_>>()
                            .join("\n")
                    )
                } else {
                    handler.get_content()
                        .map_err(|e| format!("Failed to get content: {e}"))?
                }
            }
            "azw" => {
                let mut handler = AzwHandler::new();
                handler.read_from_file(&path_buf)
                    .map_err(|e| format!("Failed to read AZW: {e}"))?;

                if extract_metadata {
                    let metadata = handler.get_metadata()
                        .map_err(|e| format!("Failed to get metadata: {e}"))?;
                    serde_json::to_string_pretty(&metadata).unwrap()
                } else if extract_toc {
                    let toc = handler.get_toc()
                        .map_err(|e| format!("Failed to get TOC: {e}"))?;
                    format!(
                        "Table of Contents:\n{}",
                        toc.iter()
                            .map(|e| format!("{}{}", "  ".repeat(e.level.saturating_sub(1)), e.title))
                            .collect::<Vec<_>>()
                            .join("\n")
                    )
                } else {
                    handler.get_content()
                        .map_err(|e| format!("Failed to get content: {e}"))?
                }
            }
            "fb2" => {
                let mut handler = Fb2Handler::new();
                handler.read_from_file(&path_buf)
                    .map_err(|e| format!("Failed to read FB2: {e}"))?;

                if extract_metadata {
                    let metadata = handler.get_metadata()
                        .map_err(|e| format!("Failed to get metadata: {e}"))?;
                    serde_json::to_string_pretty(&metadata).unwrap()
                } else if extract_toc {
                    let toc = handler.get_toc()
                        .map_err(|e| format!("Failed to get TOC: {e}"))?;
                    format!(
                        "Table of Contents:\n{}",
                        toc.iter()
                            .map(|e| format!("{}{}", "  ".repeat(e.level.saturating_sub(1)), e.title))
                            .collect::<Vec<_>>()
                            .join("\n")
                    )
                } else {
                    handler.get_content()
                        .map_err(|e| format!("Failed to get content: {e}"))?
                }
            }
            _ => return Err(format!("Unsupported format: {format}")),
        };

        Ok(ToolResult {
            content: vec![ToolContent::Text { text }],
            is_error: None,
        })
    }

    async fn tool_write_ebook(
        &self,
        args: std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<ToolResult, String> {
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'path' argument")?;
        let format = args
            .get("format")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'format' argument")?;
        let title = args.get("title").and_then(|v| v.as_str());
        let author = args.get("author").and_then(|v| v.as_str());
        let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");

        let path_buf = PathBuf::from(path);
        let mut metadata = Metadata::new();
        if let Some(t) = title {
            metadata.title = Some(t.to_string());
        }
        if let Some(a) = author {
            metadata.author = Some(a.to_string());
        }

        match format {
            "txt" => {
                let mut handler = TxtHandler::new();
                handler.set_metadata(metadata)
                    .map_err(|e| format!("Failed to set metadata: {e}"))?;
                handler.set_content(content)
                    .map_err(|e| format!("Failed to set content: {e}"))?;
                handler.write_to_file(&path_buf)
                    .map_err(|e| format!("Failed to write file: {e}"))?;
            }
            "epub" => {
                let mut handler = EpubHandler::new();
                handler.set_metadata(metadata)
                    .map_err(|e| format!("Failed to set metadata: {e}"))?;
                handler.set_content(content)
                    .map_err(|e| format!("Failed to set content: {e}"))?;
                handler.write_to_file(&path_buf)
                    .map_err(|e| format!("Failed to write file: {e}"))?;
            }
            "pdf" => {
                let mut handler = PdfHandler::new();
                handler.set_metadata(metadata)
                    .map_err(|e| format!("Failed to set metadata: {e}"))?;
                handler.set_content(content)
                    .map_err(|e| format!("Failed to set content: {e}"))?;
                handler.write_to_file(&path_buf)
                    .map_err(|e| format!("Failed to write file: {e}"))?;
            }
            "mobi" => {
                let mut handler = MobiHandler::new();
                handler.set_metadata(metadata)
                    .map_err(|e| format!("Failed to set metadata: {e}"))?;
                handler.set_content(content)
                    .map_err(|e| format!("Failed to set content: {e}"))?;
                handler.write_to_file(&path_buf)
                    .map_err(|e| format!("Failed to write file: {e}"))?;
            }
            "azw" => {
                let mut handler = AzwHandler::new();
                handler.set_metadata(metadata)
                    .map_err(|e| format!("Failed to set metadata: {e}"))?;
                handler.set_content(content)
                    .map_err(|e| format!("Failed to set content: {e}"))?;
                handler.write_to_file(&path_buf)
                    .map_err(|e| format!("Failed to write file: {e}"))?;
            }
            "fb2" => {
                let mut handler = Fb2Handler::new();
                handler.set_metadata(metadata)
                    .map_err(|e| format!("Failed to set metadata: {e}"))?;
                handler.set_content(content)
                    .map_err(|e| format!("Failed to set content: {e}"))?;
                handler.write_to_file(&path_buf)
                    .map_err(|e| format!("Failed to write file: {e}"))?;
            }
            _ => return Err(format!("Unsupported format: {format}")),
        }

        Ok(ToolResult {
            content: vec![ToolContent::Text {
                text: format!("Successfully wrote ebook to {path}"),
            }],
            is_error: None,
        })
    }

    async fn tool_extract_images(
        &self,
        args: std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<ToolResult, String> {
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'path' argument")?;

        let path_buf = PathBuf::from(path);
        let format = crate::utils::detect_format(&path_buf)
            .map_err(|e| format!("Failed to detect format: {e}"))?;

        let images = match format.as_str() {
            "epub" => {
                let mut handler = EpubHandler::new();
                handler.read_from_file(&path_buf)
                    .map_err(|e| format!("Failed to read EPUB: {e}"))?;
                handler.extract_images()
                    .map_err(|e| format!("Failed to extract images: {e}"))?
            }
            "cbz" => {
                let mut handler = CbzHandler::new();
                handler.read_from_file(&path_buf)
                    .map_err(|e| format!("Failed to read CBZ: {e}"))?;
                handler.extract_images()
                    .map_err(|e| format!("Failed to extract images: {e}"))?
            }
            _ => return Err(format!("Format {format} does not support image extraction")),
        };

        let mut content = vec![];
        for image in images {
            let base64_data = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &image.data);
            content.push(ToolContent::Image {
                data: base64_data,
                mime_type: image.mime_type,
            });
        }

        Ok(ToolResult {
            content,
            is_error: None,
        })
    }

    async fn tool_validate_ebook(
        &self,
        args: std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<ToolResult, String> {
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'path' argument")?;

        let path_buf = PathBuf::from(path);
        let format = crate::utils::detect_format(&path_buf)
            .map_err(|e| format!("Failed to detect format: {e}"))?;

        let is_valid = match format.as_str() {
            "txt" => {
                let mut handler = TxtHandler::new();
                handler.read_from_file(&path_buf)
                    .map_err(|e| format!("Failed to read TXT: {e}"))?;
                handler.validate()
                    .map_err(|e| format!("Failed to validate: {e}"))?
            }
            "epub" => {
                let mut handler = EpubHandler::new();
                handler.read_from_file(&path_buf)
                    .map_err(|e| format!("Failed to read EPUB: {e}"))?;
                handler.validate()
                    .map_err(|e| format!("Failed to validate: {e}"))?
            }
            "pdf" => {
                let mut handler = PdfHandler::new();
                handler.read_from_file(&path_buf)
                    .map_err(|e| format!("Failed to read PDF: {e}"))?;
                handler.validate()
                    .map_err(|e| format!("Failed to validate: {e}"))?
            }
            "mobi" => {
                let mut handler = MobiHandler::new();
                handler.read_from_file(&path_buf)
                    .map_err(|e| format!("Failed to read MOBI: {e}"))?;
                handler.validate()
                    .map_err(|e| format!("Failed to validate: {e}"))?
            }
            "azw" => {
                let mut handler = AzwHandler::new();
                handler.read_from_file(&path_buf)
                    .map_err(|e| format!("Failed to read AZW: {e}"))?;
                handler.validate()
                    .map_err(|e| format!("Failed to validate: {e}"))?
            }
            "fb2" => {
                let mut handler = Fb2Handler::new();
                handler.read_from_file(&path_buf)
                    .map_err(|e| format!("Failed to read FB2: {e}"))?;
                handler.validate()
                    .map_err(|e| format!("Failed to validate: {e}"))?
            }
            "cbz" => {
                let mut handler = CbzHandler::new();
                handler.read_from_file(&path_buf)
                    .map_err(|e| format!("Failed to read CBZ: {e}"))?;
                handler.validate()
                    .map_err(|e| format!("Failed to validate: {e}"))?
            }
            _ => return Err(format!("Validation not supported for format: {format}")),
        };

        let text = if is_valid {
            format!("✓ File {path} is valid")
        } else {
            format!("✗ File {path} has validation issues")
        };

        Ok(ToolResult {
            content: vec![ToolContent::Text { text }],
            is_error: None,
        })
    }

    async fn tool_get_ebook_info(
        &self,
        args: std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<ToolResult, String> {
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'path' argument")?;

        let path_buf = PathBuf::from(path);
        let format = crate::utils::detect_format(&path_buf)
            .map_err(|e| format!("Failed to detect format: {e}"))?;

        let info = match format.as_str() {
            "txt" => {
                let mut handler = TxtHandler::new();
                handler.read_from_file(&path_buf)
                    .map_err(|e| format!("Failed to read TXT: {e}"))?;
                let metadata = handler.get_metadata()
                    .map_err(|e| format!("Failed to get metadata: {e}"))?;
                let content = handler.get_content()
                    .map_err(|e| format!("Failed to get content: {e}"))?;
                
                format!("File: {}\nFormat: {}\nMetadata:\n{}\n\nSize: {} characters",
                    path, format, serde_json::to_string_pretty(&metadata).unwrap(), content.len())
            }
            "epub" => {
                let mut handler = EpubHandler::new();
                handler.read_from_file(&path_buf)
                    .map_err(|e| format!("Failed to read EPUB: {e}"))?;
                let metadata = handler.get_metadata()
                    .map_err(|e| format!("Failed to get metadata: {e}"))?;
                
                format!("File: {}\nFormat: {}\nMetadata:\n{}",
                    path, format, serde_json::to_string_pretty(&metadata).unwrap())
            }
            "pdf" => {
                let mut handler = PdfHandler::new();
                handler.read_from_file(&path_buf)
                    .map_err(|e| format!("Failed to read PDF: {e}"))?;
                let metadata = handler.get_metadata()
                    .map_err(|e| format!("Failed to get metadata: {e}"))?;
                
                format!("File: {}\nFormat: {}\nMetadata:\n{}",
                    path, format, serde_json::to_string_pretty(&metadata).unwrap())
            }
            "mobi" => {
                let mut handler = MobiHandler::new();
                handler.read_from_file(&path_buf)
                    .map_err(|e| format!("Failed to read MOBI: {e}"))?;
                let metadata = handler.get_metadata()
                    .map_err(|e| format!("Failed to get metadata: {e}"))?;
                
                format!("File: {}\nFormat: {}\nMetadata:\n{}",
                    path, format, serde_json::to_string_pretty(&metadata).unwrap())
            }
            "azw" => {
                let mut handler = AzwHandler::new();
                handler.read_from_file(&path_buf)
                    .map_err(|e| format!("Failed to read AZW: {e}"))?;
                let metadata = handler.get_metadata()
                    .map_err(|e| format!("Failed to get metadata: {e}"))?;
                
                format!("File: {}\nFormat: {}\nMetadata:\n{}",
                    path, format, serde_json::to_string_pretty(&metadata).unwrap())
            }
            "fb2" => {
                let mut handler = Fb2Handler::new();
                handler.read_from_file(&path_buf)
                    .map_err(|e| format!("Failed to read FB2: {e}"))?;
                let metadata = handler.get_metadata()
                    .map_err(|e| format!("Failed to get metadata: {e}"))?;
                
                format!("File: {}\nFormat: {}\nMetadata:\n{}",
                    path, format, serde_json::to_string_pretty(&metadata).unwrap())
            }
            "cbz" => {
                let mut handler = CbzHandler::new();
                handler.read_from_file(&path_buf)
                    .map_err(|e| format!("Failed to read CBZ: {e}"))?;
                let metadata = handler.get_metadata()
                    .map_err(|e| format!("Failed to get metadata: {e}"))?;
                
                format!("File: {}\nFormat: {}\nMetadata:\n{}",
                    path, format, serde_json::to_string_pretty(&metadata).unwrap())
            }
            _ => return Err(format!("Info not supported for format: {format}")),
        };

        Ok(ToolResult {
            content: vec![ToolContent::Text { text: info }],
            is_error: None,
        })
    }

    async fn tool_convert_ebook(
        &self,
        args: std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<ToolResult, String> {
        let input_path = args
            .get("input_path")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'input_path' argument")?;
        let output_path = args
            .get("output_path")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'output_path' argument")?;
        let target_format = args
            .get("target_format")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'target_format' argument")?;

        let input_buf = PathBuf::from(input_path);
        let output_buf = PathBuf::from(output_path);

        Converter::convert(&input_buf, &output_buf, target_format)
            .map_err(|e| format!("Conversion failed: {e}"))?;

        Ok(ToolResult {
            content: vec![ToolContent::Text {
                text: format!("Successfully converted {input_path} to {target_format} format"),
            }],
            is_error: None,
        })
    }

    async fn tool_optimize_images(
        &self,
        args: std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<ToolResult, String> {
        use crate::image_optimizer::OptimizationOptions;

        let input_path = args
            .get("input_path")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'input_path' argument")?;
        
        let output_path = args
            .get("output_path")
            .and_then(|v| v.as_str())
            .unwrap_or(input_path);
        
        let max_width = args
            .get("max_width")
            .and_then(|v| v.as_u64())
            .unwrap_or(1920) as u32;
        
        let max_height = args
            .get("max_height")
            .and_then(|v| v.as_u64())
            .unwrap_or(1920) as u32;
        
        let quality = args
            .get("quality")
            .and_then(|v| v.as_u64())
            .unwrap_or(85) as u8;
        
        let no_resize = args
            .get("no_resize")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let input_buf = PathBuf::from(input_path);
        let output_buf = PathBuf::from(output_path);

        let format = crate::utils::detect_format(&input_buf)
            .map_err(|e| format!("Failed to detect format: {e}"))?;

        let mut options = OptimizationOptions::default().with_quality(quality);
        
        if no_resize {
            options = options.no_resize();
        } else {
            options = options.with_max_dimensions(max_width, max_height);
        }

        let savings = match format.as_str() {
            "epub" => {
                let mut handler = EpubHandler::new();
                handler.read_from_file(&input_buf)
                    .map_err(|e| format!("Failed to read EPUB: {e}"))?;
                
                let savings = handler.optimize_images(options)
                    .map_err(|e| format!("Failed to optimize images: {e}"))?;
                
                handler.write_to_file(&output_buf)
                    .map_err(|e| format!("Failed to write EPUB: {e}"))?;
                
                savings
            }
            "cbz" => {
                let mut handler = CbzHandler::new();
                handler.read_from_file(&input_buf)
                    .map_err(|e| format!("Failed to read CBZ: {e}"))?;
                
                let savings = handler.optimize_images(options)
                    .map_err(|e| format!("Failed to optimize images: {e}"))?;
                
                handler.write_to_file(&output_buf)
                    .map_err(|e| format!("Failed to write CBZ: {e}"))?;
                
                savings
            }
            _ => {
                return Err(format!("Image optimization only supports EPUB and CBZ formats, got: {format}"));
            }
        };

        let savings_mb = savings as f64 / 1024.0 / 1024.0;
        let message = format!(
            "Successfully optimized images in {input_path}\nSaved: {savings} bytes ({savings_mb:.2} MB)\nOutput: {output_path}"
        );

        Ok(ToolResult {
            content: vec![ToolContent::Text { text: message }],
            is_error: None,
        })
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}
