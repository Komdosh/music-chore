//! Integration tests for MCP Server functionality
//!
//! Tests the MCP server using the rmcp crate by running it as a subprocess
//! and communicating via the MCP protocol. This provides end-to-end testing
//! of the complete MCP protocol implementation.

use anyhow::Result;
use rmcp::ServiceError::McpError;
use rmcp::{
    RmcpError, ServiceError, ServiceExt, model::CallToolRequestParams, object,
    transport::TokioChildProcess,
};
use rmcp::model::ErrorCode;
use tokio::process::Command;
use tracing::error;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_mcp_server_initialization() -> Result<()> {
    // Enable logging in tests (RUST_LOG=debug cargo test)
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Spawn MCP server via cargo
    let cmd = Command::new(env!("CARGO_BIN_EXE_musicctl-mcp"));

    let child = TokioChildProcess::new(cmd)?;
    let client = ().serve(child).await?;
    // Check server info
    let info = client.peer_info().expect("No peer info");
    assert_eq!(info.server_info.name, "music-chore");
    assert!(info.server_info.version.starts_with("0.1."));
    // Gracefully shut down
    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_mcp_server_tools_list() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cmd = Command::new(env!("CARGO_BIN_EXE_musicctl-mcp"));

    let child = TokioChildProcess::new(cmd)?;
    let client = ().serve(child).await?;

    // List tools
    let tools = client.list_all_tools().await?;

    // Should have exactly 5 tools
    assert_eq!(tools.len(), 5);

    // Check that expected tools are present
    let tool_names: Vec<String> = tools.iter().map(|t| t.name.to_string()).collect();
    assert!(tool_names.contains(&"scan_directory".to_string()));
    assert!(tool_names.contains(&"get_library_tree".to_string()));
    assert!(tool_names.contains(&"read_file_metadata".to_string()));
    assert!(tool_names.contains(&"normalize_titles".to_string()));
    assert!(tool_names.contains(&"emit_library_metadata".to_string()));

    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_mcp_server_scan_directory_tool() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cmd = Command::new(env!("CARGO_BIN_EXE_musicctl-mcp"));

    let child = TokioChildProcess::new(cmd)?;
    let client = ().serve(child).await?;

    // Test scan_directory with JSON output
    let result = client
        .call_tool(CallToolRequestParams {
            meta: None,
            name: "scan_directory".into(),
            arguments: Some(object!({
                "path": "tests/fixtures/flac/simple",
                "json_output": true
            })),
            task: None,
        })
        .await?;

    assert_eq!(result.is_error.unwrap_or(false), false);

    // Parse the JSON response
    let content = result.content.first().expect("No content");
    let text = &*content.raw.as_text().unwrap().text;
    let scan_result: serde_json::Value = serde_json::from_str(text)?;

    // Should find 2 tracks in the simple fixture
    assert_eq!(scan_result.as_array().unwrap().len(), 2);

    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_mcp_server_get_library_tree_tool() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cmd = Command::new(env!("CARGO_BIN_EXE_musicctl-mcp"));

    let child = TokioChildProcess::new(cmd)?;
    let client = ().serve(child).await?;

    // Test get_library_tree
    let result = client
        .call_tool(CallToolRequestParams {
            meta: None,
            name: "get_library_tree".into(),
            arguments: Some(object!({
                "path": "tests/fixtures/flac/nested",
                "json_output": false
            })),
            task: None,
        })
        .await?;

    assert_eq!(result.is_error.unwrap_or(false), false);

    // Parse the JSON response (it returns JSON regardless of the json_output param)
    let content = result.content.first().expect("No content");
    let s = &*content.raw.as_text().unwrap().text;
    let tree_result: serde_json::Value = serde_json::from_str(s)?;

    // Check basic structure
    assert!(tree_result.get("total_artists").is_some());
    assert!(tree_result.get("total_albums").is_some());
    assert!(tree_result.get("total_tracks").is_some());
    assert!(tree_result.get("artists").is_some());

    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_mcp_server_read_file_metadata_tool() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cmd = Command::new(env!("CARGO_BIN_EXE_musicctl-mcp"));

    let child = TokioChildProcess::new(cmd)?;
    let client = ().serve(child).await?;

    // Test read_file_metadata
    let result = client
        .call_tool(CallToolRequestParams {
            meta: None,
            name: "read_file_metadata".into(),
            arguments: Some(object!({
                "file_path": "tests/fixtures/flac/simple/track1.flac"
            })),
            task: None,
        })
        .await?;

    assert_eq!(result.is_error.unwrap_or(false), false);

    // Parse the JSON response
    let content = result.content.first().expect("No content");
    let s = &*content.raw.as_text().unwrap().text;
    let metadata: serde_json::Value = serde_json::from_str(s)?;

    // Should contain file path and metadata structure
    assert!(metadata.get("file_path").is_some());
    assert!(metadata.get("metadata").is_some());
    assert_eq!(
        metadata.get("file_path").unwrap().as_str().unwrap(),
        "tests/fixtures/flac/simple/track1.flac"
    );

    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_mcp_server_normalize_titles_tool() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cmd = Command::new(env!("CARGO_BIN_EXE_musicctl-mcp"));

    let child = TokioChildProcess::new(cmd)?;
    let client = ().serve(child).await?;

    // Test normalize_titles with dry run
    let result = client
        .call_tool(CallToolRequestParams {
            meta: None,
            name: "normalize_titles".into(),
            arguments: Some(object!({
                "path": "tests/fixtures/normalization",
                "dry_run": true
            })),
            task: None,
        })
        .await?;

    assert_eq!(result.is_error.unwrap_or(false), false);

    let content = result.content.first().expect("No content");
    let text = &*content.raw.as_text().unwrap().text;

    // Should contain operation results
    assert!(text.contains("NORMALIZED:") || text.contains("NO CHANGE:") || text.contains("ERROR:"));

    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_mcp_server_emit_library_metadata_tool() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cmd = Command::new(env!("CARGO_BIN_EXE_musicctl-mcp"));

    let child = TokioChildProcess::new(cmd)?;
    let client = ().serve(child).await?;

    // Test emit_library_metadata with text format
    let result = client
        .call_tool(CallToolRequestParams {
            meta: None,
            name: "emit_library_metadata".into(),
            arguments: Some(object!({
                "path": "tests/fixtures/flac/simple",
                "json_output": false
            })),
            task: None,
        })
        .await?;

    assert_eq!(result.is_error.unwrap_or(false), false);

    let content = result.content.first().expect("No content");
    let text = &*content.raw.as_text().unwrap().text;

    // Should contain structured metadata output
    assert!(text.contains("=== MUSIC LIBRARY METADATA ==="));
    assert!(text.contains("Total Artists: 1"));
    assert!(text.contains("Total Albums: 1"));
    assert!(text.contains("Total Tracks: 2"));
    assert!(text.contains("ARTIST: flac"));
    assert!(text.contains("ALBUM: simple"));

    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_mcp_server_error_handling() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cmd = Command::new(env!("CARGO_BIN_EXE_musicctl-mcp"));

    let child = TokioChildProcess::new(cmd)?;
    let client = ().serve(child).await?;

    // Test error handling with invalid tool name - this should return an error
    let result = client
        .call_tool(CallToolRequestParams {
            meta: None,
            name: "nonexistent_tool".into(),
            arguments: Some(object!({})),
            task: None,
        })
        .await
        .expect_err("Should return an error");

    // This should result in an error since the tool doesn't exist
    let code = match result {
        McpError(e) => e.code,
        _ => panic!("Should return MCP Error"),
    };

    assert_eq!(code, ErrorCode::INVALID_PARAMS);

    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_mcp_server_tool_parameter_validation() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cmd = Command::new(env!("CARGO_BIN_EXE_musicctl-mcp"));

    let child =
        TokioChildProcess::new(cmd).map_err(RmcpError::transport_creation::<TokioChildProcess>)?;
    let client = ().serve(child).await?;

    // Test parameter validation with missing required parameter
    let result = client.call_tool(CallToolRequestParams {
        meta: None,
        name: "scan_directory".into(),
        arguments: Some(object!({
            "json_output": true
            // Missing required "path" parameter
        })),
        task: None,
    });

    let is_error: Result<bool, bool> = match result.await {
        Ok(result) => Ok(result.is_error.unwrap_or(false)),
        Err(_d) => Ok(true),
    };

    // Should return an error due to missing required parameter
    assert_eq!(is_error.unwrap(), true);

    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_mcp_server_nonexistent_directory() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cmd = Command::new(env!("CARGO_BIN_EXE_musicctl-mcp"));

    let child = TokioChildProcess::new(cmd)?;
    let client = ().serve(child).await?;

    // Test handling of non-existent directory
    let result = client
        .call_tool(CallToolRequestParams {
            meta: None,
            name: "scan_directory".into(),
            arguments: Some(object!({
                "path": "/nonexistent/path",
                "json_output": true
            })),
            task: None,
        })
        .await?;

    // Should return success (no error) but indicate no files found
    assert_eq!(result.is_error.unwrap_or(false), false);

    let content = result.content.first().expect("No content");
    let text = &*content.raw.as_text().unwrap().text;

    // Should indicate no files found
    assert!(text.contains("No music files found"));

    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_mcp_server_emit_library_metadata_json() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cmd = Command::new(env!("CARGO_BIN_EXE_musicctl-mcp"));

    let child = TokioChildProcess::new(cmd)?;
    let client = ().serve(child).await?;

    // Test emit_library_metadata with JSON format
    let result = client
        .call_tool(CallToolRequestParams {
            meta: None,
            name: "emit_library_metadata".into(),
            arguments: Some(object!({
                "path": "tests/fixtures/flac/simple",
                "json_output": true
            })),
            task: None,
        })
        .await?;

    assert_eq!(result.is_error.unwrap_or(false), false);

    let content = result.content.first().expect("No content");
    let text = &*content.raw.as_text().unwrap().text;
    let metadata: serde_json::Value = serde_json::from_str(text)?;

    // Should contain JSON structure with library metadata
    assert!(metadata.get("total_artists").is_some());
    assert!(metadata.get("total_albums").is_some());
    assert!(metadata.get("total_tracks").is_some());
    assert!(metadata.get("artists").is_some());

    client.cancel().await?;
    Ok(())
}

#[test]
fn test_mcp_server_binary_help() {
    use std::process::Command;

    // Test that the MCP server binary responds to --help
    let output = Command::new(env!("CARGO_BIN_EXE_musicctl-mcp"))
        .args(&["--help"])
        .output()
        .expect("Failed to run musicctl-mcp --help");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");

    assert!(stdout.contains("MCP server for Music Chore CLI tool"));
    assert!(stdout.contains("verbose"));
}

#[test]
fn test_mcp_server_binary_version() {
    use std::process::Command;

    // Test that the MCP server binary responds to --version
    let output = Command::new(env!("CARGO_BIN_EXE_musicctl-mcp"))
        .args(&["--version"])
        .output()
        .expect("Failed to run musicctl-mcp --version");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");

    assert!(stdout.starts_with("musicctl-mcp "));
    assert!(stdout.contains("0.1."));
}
