# MCP (Model Context Protocol) Integration

The ebook-cli provides MCP server capability, allowing AI assistants and other tools to interact with ebook files through the Model Context Protocol.

## What is MCP?

Model Context Protocol (MCP) is a standardized protocol that allows AI assistants to interact with external tools and services. By running ebook-cli as an MCP server, AI assistants can read, write, and manipulate ebook files.

## Starting the MCP Server

```bash
# Start the MCP server (listens on stdin/stdout)
ebook mcp
```

The server communicates via JSON-RPC 2.0 over stdin/stdout, making it compatible with MCP clients.

## Available Tools

The MCP server exposes the following tools:

### 1. `read_ebook`
Read an ebook file and extract its content and metadata.

**Parameters:**
- `path` (string, required): Path to the ebook file
- `extract_metadata` (boolean, optional): Whether to extract metadata only (default: false)
- `extract_toc` (boolean, optional): Whether to extract table of contents (default: false)

**Example:**
```json
{
  "name": "read_ebook",
  "arguments": {
    "path": "/path/to/book.epub",
    "extract_metadata": true
  }
}
```

### 2. `write_ebook`
Create a new ebook file with specified content and metadata.

**Parameters:**
- `path` (string, required): Output path for the ebook file
- `format` (string, required): Format of the ebook (epub, mobi, fb2, cbz, txt, pdf)
- `title` (string, optional): Title of the ebook
- `author` (string, optional): Author of the ebook
- `content` (string, optional): Text content of the ebook

**Example:**
```json
{
  "name": "write_ebook",
  "arguments": {
    "path": "/path/to/output.epub",
    "format": "epub",
    "title": "My Book",
    "author": "John Doe",
    "content": "Chapter 1\n\nOnce upon a time..."
  }
}
```

### 3. `extract_images`
Extract images from an ebook file.

**Parameters:**
- `path` (string, required): Path to the ebook file

**Returns:** Base64-encoded images with MIME types

**Example:**
```json
{
  "name": "extract_images",
  "arguments": {
    "path": "/path/to/book.epub"
  }
}
```

### 4. `validate_ebook`
Validate an ebook file structure and metadata.

**Parameters:**
- `path` (string, required): Path to the ebook file

**Example:**
```json
{
  "name": "validate_ebook",
  "arguments": {
    "path": "/path/to/book.epub"
  }
}
```

### 5. `get_ebook_info`
Get detailed information about an ebook file.

**Parameters:**
- `path` (string, required): Path to the ebook file

**Example:**
```json
{
  "name": "get_ebook_info",
  "arguments": {
    "path": "/path/to/book.txt"
  }
}
```

## Configuration

### Claude Desktop

To use ebook-cli with Claude Desktop, add the following to your Claude configuration file:

**macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "ebook": {
      "command": "/path/to/ebook-cli/target/release/ebook",
      "args": ["mcp"]
    }
  }
}
```

### Other MCP Clients

For other MCP-compatible clients, configure them to run:
```bash
/path/to/ebook-cli/target/release/ebook mcp
```

## Protocol Details

The MCP server implements JSON-RPC 2.0 over stdio:

### Initialize
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {}
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "tools": {
        "listChanged": false
      }
    },
    "serverInfo": {
      "name": "ebook-mcp-server",
      "version": "0.1.0"
    }
  }
}
```

### List Tools
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/list",
  "params": {}
}
```

### Call Tool
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "read_ebook",
    "arguments": {
      "path": "/path/to/book.txt"
    }
  }
}
```

## Use Cases

### With AI Assistants

AI assistants can use the MCP server to:
- Read ebook content for analysis or summarization
- Extract metadata for cataloging
- Create new ebooks from generated content
- Validate ebook files
- Extract images for processing

### Example Workflow

1. AI reads an ebook: `read_ebook` with `path`
2. AI analyzes the content
3. AI creates a summary ebook: `write_ebook` with summary content
4. AI validates the output: `validate_ebook`

## Troubleshooting

### Server Not Starting
- Ensure the binary is built: `cargo build --release`
- Check the path in your MCP configuration
- Verify permissions on the binary

### Tool Calls Failing
- Check the file paths are absolute
- Ensure the ebook format is supported
- Verify file permissions

### No Response from Server
- Check stderr for error messages
- Ensure JSON-RPC format is correct
- Verify the MCP client is compatible

## Development

To test the MCP server manually:

```bash
# Start the server
./target/release/ebook mcp

# Send a request (paste JSON and press Enter)
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}

# List tools
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}

# Call a tool
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"read_ebook","arguments":{"path":"examples/sample.txt"}}}
```

## Security Considerations

- The MCP server has full filesystem access
- Only expose it to trusted MCP clients
- File paths are not sandboxed
- Consider running in a restricted environment for production use
