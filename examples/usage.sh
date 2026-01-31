#!/bin/bash

# Example usage script for ebook-cli
# This script demonstrates various CLI operations

set -e

echo "=== ebook-cli Usage Examples ==="
echo ""

# Build the project first
echo "Building ebook-cli..."
cargo build --release
EBOOK_CLI="./target/release/ebook"

echo ""
echo "1. Reading a text file"
echo "----------------------"
$EBOOK_CLI read examples/sample.txt

echo ""
echo "2. Showing metadata"
echo "-------------------"
$EBOOK_CLI read examples/sample.txt --metadata

echo ""
echo "3. Showing table of contents"
echo "----------------------------"
$EBOOK_CLI read examples/sample.txt --toc

echo ""
echo "4. Getting file info"
echo "--------------------"
$EBOOK_CLI info examples/sample.txt

echo ""
echo "5. Validating file"
echo "------------------"
$EBOOK_CLI validate examples/sample.txt

echo ""
echo "6. Creating a new ebook"
echo "-----------------------"
$EBOOK_CLI write /tmp/test_output.txt \
    --format txt \
    --title "Test Book" \
    --author "Test Author" \
    --content examples/sample.txt

echo ""
echo "7. Reading the created file"
echo "---------------------------"
$EBOOK_CLI read /tmp/test_output.txt --metadata

echo ""
echo "8. Converting TXT to Markdown"
echo "------------------------------"
$EBOOK_CLI convert examples/sample.txt /tmp/output.md

echo ""
echo "9. Repairing a file"
echo "-------------------"
$EBOOK_CLI repair examples/sample.txt --output /tmp/repaired.txt

echo ""
echo "=== All examples completed successfully! ==="
