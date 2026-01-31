#!/bin/bash

# Test script for MCP server functionality
# This script sends JSON-RPC requests to the MCP server

set -e

echo "=== Testing MCP Server ==="
echo ""

# Build the project
echo "Building project..."
cargo build --release

EBOOK_BIN="./target/release/ebook"

# Create a test file
echo "Creating test file..."
echo "Chapter 1

This is a test ebook for MCP server testing.

Chapter 2

More content here." > /tmp/test_mcp.txt

echo ""
echo "Starting MCP server tests..."
echo ""

# Test 1: Initialize
echo "Test 1: Initialize"
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | $EBOOK_BIN mcp | head -1
echo ""

# Test 2: List tools
echo "Test 2: List tools"
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' | $EBOOK_BIN mcp | head -1
echo ""

# Test 3: Read ebook
echo "Test 3: Read ebook"
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"read_ebook","arguments":{"path":"/tmp/test_mcp.txt"}}}' | $EBOOK_BIN mcp | head -1
echo ""

# Test 4: Read ebook metadata
echo "Test 4: Read ebook metadata"
echo '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"read_ebook","arguments":{"path":"/tmp/test_mcp.txt","extract_metadata":true}}}' | $EBOOK_BIN mcp | head -1
echo ""

# Test 5: Get ebook info
echo "Test 5: Get ebook info"
echo '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"get_ebook_info","arguments":{"path":"/tmp/test_mcp.txt"}}}' | $EBOOK_BIN mcp | head -1
echo ""

# Test 6: Validate ebook
echo "Test 6: Validate ebook"
echo '{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"validate_ebook","arguments":{"path":"/tmp/test_mcp.txt"}}}' | $EBOOK_BIN mcp | head -1
echo ""

# Test 7: Write ebook
echo "Test 7: Write ebook"
echo '{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"write_ebook","arguments":{"path":"/tmp/test_output_mcp.txt","format":"txt","title":"Test Book","author":"Test Author","content":"Test content from MCP"}}}' | $EBOOK_BIN mcp | head -1
echo ""

# Cleanup
rm -f /tmp/test_mcp.txt /tmp/test_output_mcp.txt

echo "=== All MCP tests completed ==="
