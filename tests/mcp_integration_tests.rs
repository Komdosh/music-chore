//! Refactored Integration tests for MCP Server functionality
//! DRY helpers, shared setup, and consistent assertions

use anyhow::Result;
use rmcp::model::JsonObject;
use rmcp::service::RunningService;
use rmcp::ServiceError::McpError;
use rmcp::{
    model::{CallToolRequestParams, ErrorCode}, object, transport::TokioChildProcess,
    RmcpError,
    RoleClient,
    ServiceExt,
};
use std::borrow::Cow;
use tokio::process::Command;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/* ----------------------------- Shared helpers ----------------------------- */

fn init_tracing() {
    let _ = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .try_init();
}

async fn spawn_client() -> Result<RunningService<RoleClient, ()>> {
    init_tracing();
    let cmd = Command::new(env!("CARGO_BIN_EXE_musicctl-mcp"));
    let child =
        TokioChildProcess::new(cmd).map_err(RmcpError::transport_creation::<TokioChildProcess>)?;
    let client = ().serve(child).await?;

    Ok(client)
}

async fn call_tool(
    client: &RunningService<RoleClient, ()>,
    name: &str,
    args: JsonObject,
) -> Result<rmcp::model::CallToolResult> {
    Ok(client
        .call_tool(CallToolRequestParams {
            meta: None,
            name: Cow::from(name.to_string()),
            arguments: Some(args),
            task: None,
        })
        .await?)
}

fn text_content(result: &rmcp::model::CallToolResult) -> &str {
    result
        .content
        .first()
        .expect("No content")
        .raw
        .as_text()
        .unwrap()
        .text
        .as_str()
}

fn assert_ok(result: &rmcp::model::CallToolResult) {
    assert_eq!(result.is_error.unwrap_or(false), false);
}

fn assert_err(result: &rmcp::model::CallToolResult) {
    assert_eq!(result.is_error.unwrap_or(false), true);
}

async fn shutdown(client: RunningService<RoleClient, ()>) -> Result<()> {
    client.cancel().await?;
    Ok(())
}

/* --------------------------------- Tests -------------------------------- */

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_server_initialization() -> Result<()> {
    let client = spawn_client().await?;

    let info = client.peer_info().expect("No peer info");
    assert_eq!(info.server_info.name, "music-chore");
    assert!(info.server_info.version.starts_with("0.2."));

    shutdown(client).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_tools_list() -> Result<()> {
    let client = spawn_client().await?;

    let tools = client.list_all_tools().await?;
    assert_eq!(tools.len(), 7);

    let names: Vec<_> = tools.iter().map(|t| t.name.to_string()).collect();
    for expected in [
        "scan_directory",
        "get_library_tree",
        "read_file_metadata",
        "normalize_titles",
        "emit_library_metadata",
        "validate_library",
        "find_duplicates",
    ] {
        assert!(names.contains(&expected.to_string()));
    }

    shutdown(client).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_scan_directory() -> Result<()> {
    let client = spawn_client().await?;

    let result = call_tool(
        &client,
        "scan_directory",
        object!({
            "path": "tests/fixtures/flac/simple",
            "json_output": true
        }),
    )
    .await?;

    assert_ok(&result);

    let json: serde_json::Value = serde_json::from_str(text_content(&result))?;
    assert_eq!(json.as_array().unwrap().len(), 2);

    shutdown(client).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_library_tree() -> Result<()> {
    let client = spawn_client().await?;

    let result = call_tool(
        &client,
        "get_library_tree",
        object!({
            "path": "tests/fixtures/flac/nested",
            "json_output": false
        }),
    )
    .await?;

    assert_ok(&result);

    let json: serde_json::Value = serde_json::from_str(text_content(&result))?;
    for key in ["total_artists", "total_albums", "total_tracks", "artists"] {
        assert!(json.get(key).is_some());
    }

    shutdown(client).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_read_file_metadata() -> Result<()> {
    let client = spawn_client().await?;

    let result = call_tool(
        &client,
        "read_file_metadata",
        object!({
            "file_path": "tests/fixtures/flac/simple/track1.flac"
        }),
    )
    .await?;

    assert_ok(&result);

    let json: serde_json::Value = serde_json::from_str(text_content(&result))?;
    assert_eq!(
        json.get("file_path").unwrap().as_str().unwrap(),
        "tests/fixtures/flac/simple/track1.flac"
    );
    assert!(json.get("metadata").is_some());

    shutdown(client).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_normalize_titles() -> Result<()> {
    let client = spawn_client().await?;

    let result = call_tool(
        &client,
        "normalize_titles",
        object!({
            "path": "tests/fixtures/normalization",
            "dry_run": true
        }),
    )
    .await?;

    assert_ok(&result);

    let text = text_content(&result);
    assert!(text.contains("NORMALIZED:") || text.contains("NO CHANGE:") || text.contains("ERROR:"));

    shutdown(client).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_emit_library_metadata_text() -> Result<()> {
    let client = spawn_client().await?;

    let result = call_tool(
        &client,
        "emit_library_metadata",
        object!({
            "path": "tests/fixtures/flac/simple",
            "json_output": false
        }),
    )
    .await?;

    assert_ok(&result);

    let text = text_content(&result);
    println!("{}", text);
    for expected in [
        "=== MUSIC LIBRARY METADATA ===",
        "Total Artists: 1",
        "Total Albums: 1",
        "Total Tracks: 2",
        "ARTIST: flac",
        "ALBUM: simple",
        "TRACK: \"[Unknown Title]\" | Duration: 0:00 | File: tests/fixtures/flac/simple/track1.flac",
        "TRACK: \"[Unknown Title]\" | Duration: 0:00 | File: tests/fixtures/flac/simple/track2.flac"
    ] {
        assert!(text.contains(expected));
    }

    shutdown(client).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_emit_library_metadata_json() -> Result<()> {
    let client = spawn_client().await?;

    let result = call_tool(
        &client,
        "emit_library_metadata",
        object!({
            "path": "tests/fixtures/flac/simple",
            "json_output": true
        }),
    )
    .await?;

    assert_ok(&result);

    let json: serde_json::Value = serde_json::from_str(text_content(&result))?;
    for key in ["total_artists", "total_albums", "total_tracks", "artists"] {
        assert!(json.get(key).is_some());
    }

    shutdown(client).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_error_invalid_tool() -> Result<()> {
    let client = spawn_client().await?;

    let err = client
        .call_tool(CallToolRequestParams {
            meta: None,
            name: "nonexistent_tool".into(),
            arguments: Some(object!({})),
            task: None,
        })
        .await
        .expect_err("Expected error");

    let code = match err {
        McpError(e) => e.code,
        _ => panic!("Expected MCP error"),
    };

    assert_eq!(code, ErrorCode::INVALID_PARAMS);

    shutdown(client).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_parameter_validation() -> Result<()> {
    let client = spawn_client().await?;

    let result = call_tool(
        &client,
        "scan_directory",
        object!({
            "json_output": true
        }),
    )
    .await;

    assert!(result.is_err() || result.unwrap().is_error.unwrap_or(true));

    shutdown(client).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_nonexistent_directory() -> Result<()> {
    let client = spawn_client().await?;

    let result = call_tool(
        &client,
        "scan_directory",
        object!({
            "path": "/nonexistent/path",
            "json_output": true
        }),
    )
    .await?;

    assert_err(&result);

    let text = text_content(&result);
    assert!(text.contains("No music files found"));

    shutdown(client).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_validate_library_text() -> Result<()> {
    let client = spawn_client().await?;

    let result = call_tool(
        &client,
        "validate_library",
        object!({
            "path": "tests/fixtures/flac/simple",
            "json_output": false
        }),
    )
    .await?;

    assert_ok(&result);

    let text = text_content(&result);
    println!("{}", text);
    for expected in [
        "=== METADATA VALIDATION RESULTS ===",
        "📊 Summary:",
        "  Total files: 1",
        "  Valid files: 1",
        "  Files with errors: 0",
        "  Files with warnings: 0",
    ] {
        assert!(text.contains(expected));
    }

    shutdown(client).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_validate_library_json() -> Result<()> {
    let client = spawn_client().await?;

    let result = call_tool(
        &client,
        "validate_library",
        object!({
            "path": "tests/fixtures/flac/simple",
            "json_output": true
        }),
    )
    .await?;

    assert_ok(&result);

    let json: serde_json::Value = serde_json::from_str(text_content(&result))?;
    for key in ["valid", "errors", "warnings", "summary"] {
        assert!(json.get(key).is_some());
    }

    shutdown(client).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_validate_empty_directory() -> Result<()> {
    let client = spawn_client().await?;

    let result = call_tool(
        &client,
        "validate_library",
        object!({
            "path": "/nonexistent/path",
            "json_output": false
        }),
    )
    .await?;

    assert_err(&result);

    let text = text_content(&result);
    assert!(text.contains("No music files found to validate."));

    shutdown(client).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_validate_nested_directory() -> Result<()> {
    let client = spawn_client().await?;

    let result = client
        .call_tool(CallToolRequestParams {
            meta: None,
            name: "validate_library".into(),
            arguments: Some(object!({
                "path": "tests/fixtures/flac/nested",
                "json_output": false
            })),
            task: None,
        })
        .await?;

    assert_err(&result);

    let text = text_content(&result);
    assert!(text.contains("Unable to read metadata from any files for validation"));

    shutdown(client).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_find_duplicates() -> Result<()> {
    let client = spawn_client().await?;

    // // Test with duplicates
    let result = call_tool(
        &client,
        "find_duplicates",
        object!({
            "path": "tests/fixtures/duplicates",
            "json_output": false
        }),
    )
    .await?;

    assert_ok(&result);

    let text = text_content(&result);
    assert!(text.contains("Found") && text.contains("duplicate groups"));
    assert!(text.contains("Duplicate Group 1"));
    assert!(text.contains("track1.flac") || text.contains("track2.flac"));

    // Test JSON output
    let result = call_tool(
        &client,
        "find_duplicates",
        object!({
            "path": "tests/fixtures/duplicates",
            "json_output": true
        }),
    )
    .await?;

    assert_ok(&result);

    let json_text = text_content(&result);
    assert!(json_text.starts_with("["));
    assert!(json_text.contains("checksum"));

    // Test with no duplicates
    let result = call_tool(
        &client,
        "find_duplicates",
        object!({
            "path": "tests/fixtures/flac/simple",
            "json_output": false
        }),
    )
    .await?;

    assert_err(&result);
    let text = text_content(&result);
    assert!(text.contains("No duplicate tracks found"));

    shutdown(client).await
}

/* -------------------------- Binary CLI smoke tests ------------------------- */

#[test]
fn test_binary_help() {
    use std::process::Command;

    let output = Command::new(env!("CARGO_BIN_EXE_musicctl-mcp"))
        .args(["--help"])
        .output()
        .expect("Failed to run --help");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("MCP server for Music Chore CLI tool"));
    assert!(stdout.contains("verbose"));
}

#[test]
fn test_binary_version() {
    use std::process::Command;

    let output = Command::new(env!("CARGO_BIN_EXE_musicctl-mcp"))
        .args(["--version"])
        .output()
        .expect("Failed to run --version");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.starts_with("musicctl-mcp "));
    assert!(stdout.contains("0.2."));
}
