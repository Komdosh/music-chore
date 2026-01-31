//! Integration tests for MCP Server functionality
//!
//! Tests the MCP server by running it as a subprocess and communicating
//! via JSON-RPC over stdio. This provides end-to-end testing of the
//! complete MCP protocol implementation.

use serde_json::{json, Value};
use std::io::Write;
use std::process::{Command, Stdio};

/// Send JSON-RPC request and get response
fn send_json_rpc_request(request: &Value) -> Result<Value, Box<dyn std::error::Error>> {
    let mut child = Command::new("cargo")
        .args(&["run", "--bin", "musicctl-mcp", "--"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let request_str = serde_json::to_string(request)?;

    // Send request
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(request_str.as_bytes())?;
        stdin.write_all(b"\n")?;
        stdin.flush()?;
    }

    // Read response with timeout
    let output = child.wait_with_output()?;
    let response_str = String::from_utf8(output.stdout)?;

    if response_str.trim().is_empty() {
        return Err("Empty response from MCP server".into());
    }

    let response: Value = serde_json::from_str(&response_str)?;
    Ok(response)
}

#[test]
fn test_mcp_server_initialization() {
    // Test that the MCP server can start and respond to initialization
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    });

    let response =
        send_json_rpc_request(&init_request).expect("Failed to send initialization request");

    // Check basic response structure
    assert!(response.get("jsonrpc").is_some());
    assert_eq!(response.get("id"), Some(&json!(1)));
    assert!(response.get("result").is_some());

    let result = response.get("result").unwrap();
    assert!(result.get("protocolVersion").is_some());
    assert!(result.get("capabilities").is_some());
    assert!(result.get("serverInfo").is_some());

    let server_info = result.get("serverInfo").unwrap();
    assert_eq!(server_info.get("name"), Some(&json!("music-chore")));
    assert!(server_info.get("version").is_some());
}

#[test]
fn test_mcp_server_tools_list() {
    // Test that we can list available tools
    let tools_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });

    let response =
        send_json_rpc_request(&tools_request).expect("Failed to send tools/list request");

    assert_eq!(response.get("id"), Some(&json!(2)));
    let result = response.get("result").unwrap();
    let tools = result.get("tools").unwrap().as_array().unwrap();

    // Should have exactly 5 tools
    assert_eq!(tools.len(), 5);

    // Check that expected tools are present
    let tool_names: Vec<String> = tools
        .iter()
        .filter_map(|t| t.get("name")?.as_str().map(String::from))
        .collect();

    assert!(tool_names.contains(&"scan_directory".to_string()));
    assert!(tool_names.contains(&"get_library_tree".to_string()));
    assert!(tool_names.contains(&"read_file_metadata".to_string()));
    assert!(tool_names.contains(&"normalize_titles".to_string()));
    assert!(tool_names.contains(&"emit_library_metadata".to_string()));
}

#[test]
fn test_mcp_server_scan_directory_tool() {
    // Test the scan_directory tool with a real fixture
    let scan_request = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "scan_directory",
            "arguments": {
                "path": "tests/fixtures/flac/simple",
                "json_output": true
            }
        }
    });

    let response =
        send_json_rpc_request(&scan_request).expect("Failed to send scan_directory request");

    assert_eq!(response.get("id"), Some(&json!(3)));
    let result = response.get("result").unwrap();
    let content = result.get("content").unwrap().as_array().unwrap();

    assert_eq!(content.len(), 1);
    let text_content = &content[0];
    assert_eq!(text_content.get("type"), Some(&json!("text")));

    let text = text_content.get("text").unwrap().as_str().unwrap();
    let scan_result: Value = serde_json::from_str(text).unwrap();

    // Should find 2 tracks in the simple fixture
    assert_eq!(scan_result.as_array().unwrap().len(), 2);
}

#[test]
fn test_mcp_server_get_library_tree_tool() {
    // Test the get_library_tree tool
    let tree_request = json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "get_library_tree",
            "arguments": {
                "path": "tests/fixtures/flac/nested",
                "json_output": false
            }
        }
    });

    let response =
        send_json_rpc_request(&tree_request).expect("Failed to send get_library_tree request");

    assert_eq!(response.get("id"), Some(&json!(4)));
    let result = response.get("result").unwrap();
    let content = result.get("content").unwrap().as_array().unwrap();

    assert_eq!(content.len(), 1);
    let text_content = &content[0];
    assert_eq!(text_content.get("type"), Some(&json!("text")));

    let text = text_content.get("text").unwrap().as_str().unwrap();
    let tree_result: Value = serde_json::from_str(text).unwrap();
    let tree_text = tree_result.get("tree").unwrap().as_str().unwrap();

    // Should contain expected tree structure
    assert!(tree_text.contains("=== MUSIC LIBRARY TREE ==="));
    assert!(tree_text.contains("Total Artists: 1"));
    assert!(tree_text.contains("Total Albums: 1"));
    assert!(tree_text.contains("Total Tracks: 2"));
    assert!(tree_text.contains("📁 The Beatles"));
    assert!(tree_text.contains("📂 Abbey Road"));
}

#[test]
fn test_mcp_server_read_file_metadata_tool() {
    // Test the read_file_metadata tool
    let read_request = json!({
        "jsonrpc": "2.0",
        "id": 5,
        "method": "tools/call",
        "params": {
            "name": "read_file_metadata",
            "arguments": {
                "file_path": "tests/fixtures/flac/simple/track1.flac"
            }
        }
    });

    let response =
        send_json_rpc_request(&read_request).expect("Failed to send read_file_metadata request");

    assert_eq!(response.get("id"), Some(&json!(5)));
    let result = response.get("result").unwrap();
    let content = result.get("content").unwrap().as_array().unwrap();

    assert_eq!(content.len(), 1);
    let text_content = &content[0];
    assert_eq!(text_content.get("type"), Some(&json!("text")));

    let text = text_content.get("text").unwrap().as_str().unwrap();
    let metadata: Value = serde_json::from_str(text).unwrap();

    // Should contain file path and metadata structure
    assert!(metadata.get("file_path").is_some());
    assert!(metadata.get("metadata").is_some());
    assert_eq!(
        metadata.get("file_path").unwrap().as_str().unwrap(),
        "tests/fixtures/flac/simple/track1.flac"
    );
}

#[test]
fn test_mcp_server_normalize_titles_tool() {
    // Test the normalize_titles tool with dry run
    let normalize_request = json!({
        "jsonrpc": "2.0",
        "id": 6,
        "method": "tools/call",
        "params": {
            "name": "normalize_titles",
            "arguments": {
                "path": "tests/fixtures/normalization",
                "dry_run": true
            }
        }
    });

    let response =
        send_json_rpc_request(&normalize_request).expect("Failed to send normalize_titles request");

    assert_eq!(response.get("id"), Some(&json!(6)));
    let result = response.get("result").unwrap();
    let content = result.get("content").unwrap().as_array().unwrap();

    assert_eq!(content.len(), 1);
    let text_content = &content[0];
    assert_eq!(text_content.get("type"), Some(&json!("text")));

    let text = text_content.get("text").unwrap().as_str().unwrap();
    let normalize_result: Value = serde_json::from_str(text).unwrap();

    // Should contain operation summary
    assert!(normalize_result.get("processed").is_some());
    assert!(normalize_result.get("updated").is_some());
    assert!(normalize_result.get("errors").is_some());
    assert_eq!(normalize_result.get("dry_run").unwrap(), &json!(true));
}

#[test]
fn test_mcp_server_emit_library_metadata_tool() {
    // Test the emit_library_metadata tool with text format
    let emit_request = json!({
        "jsonrpc": "2.0",
        "id": 7,
        "method": "tools/call",
        "params": {
            "name": "emit_library_metadata",
            "arguments": {
                "path": "tests/fixtures/flac/simple",
                "format": "text"
            }
        }
    });

    let response =
        send_json_rpc_request(&emit_request).expect("Failed to send emit_library_metadata request");

    assert_eq!(response.get("id"), Some(&json!(7)));
    let result = response.get("result").unwrap();
    let content = result.get("content").unwrap().as_array().unwrap();

    assert_eq!(content.len(), 1);
    let text_content = &content[0];
    assert_eq!(text_content.get("type"), Some(&json!("text")));

    let text = text_content.get("text").unwrap().as_str().unwrap();
    let emit_result: Value = serde_json::from_str(text).unwrap();
    let metadata_text = emit_result.get("metadata").unwrap().as_str().unwrap();

    // Should contain structured metadata output
    assert!(metadata_text.contains("=== MUSIC LIBRARY METADATA ==="));
    assert!(metadata_text.contains("Total Artists: 1"));
    assert!(metadata_text.contains("Total Albums: 1"));
    assert!(metadata_text.contains("Total Tracks: 2"));
    assert!(metadata_text.contains("ARTIST: flac"));
    assert!(metadata_text.contains("ALBUM: simple"));
}

#[test]
fn test_mcp_server_error_handling() {
    // Test error handling with invalid tool name
    let error_request = json!({
        "jsonrpc": "2.0",
        "id": 8,
        "method": "tools/call",
        "params": {
            "name": "nonexistent_tool",
            "arguments": {}
        }
    });

    let response =
        send_json_rpc_request(&error_request).expect("Failed to send error test request");

    assert_eq!(response.get("id"), Some(&json!(8)));
    assert!(response.get("result").is_some());

    let result = response.get("result").unwrap();
    assert_eq!(result.get("isError"), Some(&json!(true)));

    let content = result.get("content").unwrap().as_array().unwrap();
    assert_eq!(content.len(), 1);
    let text_content = &content[0];
    assert_eq!(text_content.get("type"), Some(&json!("text")));
    assert!(text_content
        .get("text")
        .unwrap()
        .as_str()
        .unwrap()
        .contains("not found"));
}

#[test]
fn test_mcp_server_tool_parameter_validation() {
    // Test parameter validation with missing required parameter
    let validation_request = json!({
        "jsonrpc": "2.0",
        "id": 9,
        "method": "tools/call",
        "params": {
            "name": "scan_directory",
            "arguments": {
                "json_output": true
                // Missing required "path" parameter
            }
        }
    });

    let response =
        send_json_rpc_request(&validation_request).expect("Failed to send validation test request");

    assert_eq!(response.get("id"), Some(&json!(9)));
    let result = response.get("result").unwrap();
    assert_eq!(result.get("isError"), Some(&json!(true)));

    let content = result.get("content").unwrap().as_array().unwrap();
    assert_eq!(content.len(), 1);
    let text_content = &content[0];
    assert_eq!(text_content.get("type"), Some(&json!("text")));
    assert!(text_content
        .get("text")
        .unwrap()
        .as_str()
        .unwrap()
        .contains("Missing required parameter"));
}

#[test]
fn test_mcp_server_nonexistent_directory() {
    // Test handling of non-existent directory
    let nonexistent_request = json!({
        "jsonrpc": "2.0",
        "id": 10,
        "method": "tools/call",
        "params": {
            "name": "scan_directory",
            "arguments": {
                "path": "/nonexistent/path",
                "json_output": true
            }
        }
    });

    let response = send_json_rpc_request(&nonexistent_request)
        .expect("Failed to send nonexistent directory test");

    assert_eq!(response.get("id"), Some(&json!(10)));
    let result = response.get("result").unwrap();
    let content = result.get("content").unwrap().as_array().unwrap();

    // Should return success with empty array (no error, just no files found)
    assert_eq!(content.len(), 1);
    let text_content = &content[0];
    let text = text_content.get("text").unwrap().as_str().unwrap();
    let scan_result: Value = serde_json::from_str(text).unwrap();

    // Should return empty array for non-existent directory
    assert_eq!(scan_result.as_array().unwrap().len(), 0);
}

#[test]
fn test_mcp_server_binary_help() {
    // Test that the MCP server binary responds to --help
    let output = Command::new("cargo")
        .args(&["run", "--bin", "musicctl-mcp", "--", "--help"])
        .output()
        .expect("Failed to run musicctl-mcp --help");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");

    assert!(stdout.contains("MCP server for Music Chore CLI tool"));
    assert!(stdout.contains("verbose"));
}

#[test]
fn test_mcp_server_binary_version() {
    // Test that the MCP server binary responds to --version
    let output = Command::new("cargo")
        .args(&["run", "--bin", "musicctl-mcp", "--", "--version"])
        .output()
        .expect("Failed to run musicctl-mcp --version");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");

    assert!(stdout.starts_with("musicctl-mcp "));
    assert!(stdout.contains("0.1."));
}
