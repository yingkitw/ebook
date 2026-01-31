use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

fn start_mcp() -> (Child, ChildStdin, BufReader<ChildStdout>) {
    let bin = assert_cmd::cargo::cargo_bin("ebook");
    let mut child = Command::new(bin)
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);
    (child, stdin, reader)
}

fn send(stdin: &mut ChildStdin, req: &Value) {
    writeln!(stdin, "{}", req.to_string()).unwrap();
    stdin.flush().unwrap();
}

fn recv(reader: &mut BufReader<ChildStdout>) -> Value {
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    serde_json::from_str(&line).unwrap()
}

#[test]
fn test_mcp_initialize_and_tools_list() {
    let (mut child, mut stdin, mut reader) = start_mcp();

    let init = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": null
    });
    send(&mut stdin, &init);
    let resp = recv(&mut reader);
    assert!(resp.get("result").is_some());
    assert_eq!(resp["id"], 1);
    assert!(resp["result"]["serverInfo"]["name"].as_str().unwrap().contains("ebook"));

    let list = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": null
    });
    send(&mut stdin, &list);
    let resp = recv(&mut reader);
    assert_eq!(resp["id"], 2);

    let tools = resp["result"]["tools"].as_array().unwrap();
    let write_tool = tools
        .iter()
        .find(|t| t["name"].as_str() == Some("write_ebook"))
        .unwrap();

    let write_enum = write_tool["inputSchema"]["properties"]["format"]["enum"]
        .as_array()
        .unwrap();
    assert!(write_enum.iter().any(|v| v.as_str() == Some("azw")));

    drop(stdin);
    let _ = child.wait();
}

#[test]
fn test_mcp_write_read_validate_info_txt_and_azw() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let txt_path = temp_dir.path().join("mcp_test.txt");
    let azw_path = temp_dir.path().join("mcp_test.azw");

    let (mut child, mut stdin, mut reader) = start_mcp();

    let init = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": null
    });
    send(&mut stdin, &init);
    let _ = recv(&mut reader);

    let write_txt = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 10,
        "method": "tools/call",
        "params": {
            "name": "write_ebook",
            "arguments": {
                "path": txt_path.to_string_lossy(),
                "format": "txt",
                "title": "Test",
                "author": "Author",
                "content": "hello from mcp"
            }
        }
    });
    send(&mut stdin, &write_txt);
    let resp = recv(&mut reader);
    assert_eq!(resp["id"], 10);
    assert!(resp["result"]["content"][0]["text"].as_str().unwrap().contains("Successfully"));
    assert!(txt_path.exists());

    let read_txt = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 11,
        "method": "tools/call",
        "params": {
            "name": "read_ebook",
            "arguments": {
                "path": txt_path.to_string_lossy()
            }
        }
    });
    send(&mut stdin, &read_txt);
    let resp = recv(&mut reader);
    assert_eq!(resp["id"], 11);
    assert!(resp["result"]["content"][0]["text"].as_str().unwrap().contains("hello from mcp"));

    let validate_txt = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 12,
        "method": "tools/call",
        "params": {
            "name": "validate_ebook",
            "arguments": {
                "path": txt_path.to_string_lossy()
            }
        }
    });
    send(&mut stdin, &validate_txt);
    let resp = recv(&mut reader);
    assert_eq!(resp["id"], 12);
    assert!(resp["result"]["content"][0]["text"].as_str().unwrap().contains("valid"));

    let info_txt = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 13,
        "method": "tools/call",
        "params": {
            "name": "get_ebook_info",
            "arguments": {
                "path": txt_path.to_string_lossy()
            }
        }
    });
    send(&mut stdin, &info_txt);
    let resp = recv(&mut reader);
    assert_eq!(resp["id"], 13);
    assert!(resp["result"]["content"][0]["text"].as_str().unwrap().contains("Format"));

    let write_azw = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 20,
        "method": "tools/call",
        "params": {
            "name": "write_ebook",
            "arguments": {
                "path": azw_path.to_string_lossy(),
                "format": "azw",
                "title": "AZW Test",
                "content": "azw content"
            }
        }
    });
    send(&mut stdin, &write_azw);
    let resp = recv(&mut reader);
    assert_eq!(resp["id"], 20);
    assert!(azw_path.exists());

    let read_azw = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 21,
        "method": "tools/call",
        "params": {
            "name": "read_ebook",
            "arguments": {
                "path": azw_path.to_string_lossy()
            }
        }
    });
    send(&mut stdin, &read_azw);
    let resp = recv(&mut reader);
    assert_eq!(resp["id"], 21);
    assert!(resp["result"]["content"][0]["text"].as_str().unwrap().contains("azw"));

    drop(stdin);
    let _ = child.wait();
}
