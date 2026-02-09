//! Refactored Integration tests for MCP Server functionality
//! DRY helpers, shared setup, and consistent assertions

use anyhow::Result;
use music_chore::core::services::normalization::CombinedNormalizationReport;
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
use tempfile::TempDir;

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
    assert!(!result.is_error.unwrap_or(false));
}

fn assert_err(result: &rmcp::model::CallToolResult) {
    assert!(result.is_error.unwrap_or(false));
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
    assert!(info.server_info.version.starts_with("0.3."));

    shutdown(client).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_tools_list() -> Result<()> {
    let client = spawn_client().await?;

    let tools = client.list_all_tools().await?;
    assert_eq!(tools.len(), 8); // Updated count

    let names: Vec<_> = tools.iter().map(|t| t.name.to_string()).collect();
    for expected in [
        "scan_directory",
        "get_library_tree",
        "read_file_metadata",
        "normalize", // Changed from normalize_titles
        "emit_library_metadata",
        "validate_library",
        "find_duplicates",
        "cue_file",
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
async fn test_normalize_human_output() -> Result<()> { // Renamed
    let client = spawn_client().await?;
    let temp_dir = TempDir::new()?; // Use a truly empty temporary directory

    let result = call_tool(
        &client,
        "normalize", // Changed tool name
        object!({
            "path": temp_dir.path().to_string_lossy(),
            "json_output": false
        }),
    )
    .await?;

    assert_ok(&result);

    let text = text_content(&result);
    assert!(
        text.contains("Title Summary: 0 normalized, 0 no change, 0 errors"), // Updated assertion
        "Expected title summary for no audio files, got: {}",
        text
    );
    assert!(
        text.contains("Genre Summary: 0 normalized, 0 no change, 0 errors"), // Updated assertion
        "Expected genre summary for no audio files, got: {}",
        text
    );

    shutdown(client).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_normalize_json_output() -> Result<()> { // Renamed
    let client = spawn_client().await?;
    let temp_dir = TempDir::new()?; // Use a truly empty temporary directory

    let result = call_tool(
        &client,
        "normalize", // Changed tool name
        object!({
            "path": temp_dir.path().to_string_lossy(),
            "json_output": true
        }),
    )
    .await?;

    assert_ok(&result);

    let json_text = text_content(&result);
    // Expecting an empty CombinedNormalizationReport if no files are found
    let combined_report: CombinedNormalizationReport = serde_json::from_str(json_text)?;
    assert!(combined_report.title_reports.is_empty());
    assert!(combined_report.genre_reports.is_empty());
    assert_eq!(combined_report.summary, "Combined normalization report".to_string());

    shutdown(client).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_cue_file_validate_valid() -> Result<()> {
    let client = spawn_client().await?;

    let temp_dir = tempfile::Builder::new().tempdir()?;
    let cue_path = temp_dir.path().join("test.cue");
    let audio_path1 = temp_dir.path().join("track1.flac");
    let audio_path2 = temp_dir.path().join("track2.flac");

    std::fs::write(
        &cue_path,
        r#"PERFORMER "Artist"
TITLE "Album"
FILE "track1.flac" WAVE
  TRACK 01 AUDIO
    TITLE "Track 1"
    INDEX 01 00:00:00
FILE "track2.flac" WAVE
  TRACK 02 AUDIO
    TITLE "Track 2"
    INDEX 01 00:02:00
"#,
    )?;
    std::fs::write(&audio_path1, b"dummy audio")?;
    std::fs::write(&audio_path2, b"dummy audio")?;

    let result = call_tool(
        &client,
        "cue_file",
        object!({
            "path": cue_path.to_string_lossy(),
            "operation": "validate",
            "json_output": true
        }),
    )
    .await?;

    assert_ok(&result);

    let json_text = text_content(&result);
    let json: serde_json::Value = serde_json::from_str(&json_text)?;
    assert_eq!(json.get("is_valid").unwrap().as_bool().unwrap(), true);
    assert_eq!(json.get("parsing_error").unwrap().as_bool().unwrap(), false);
    assert_eq!(json.get("file_missing").unwrap().as_bool().unwrap(), false);
    assert_eq!(
        json.get("track_count_mismatch").unwrap().as_bool().unwrap(),
        false
    );

    shutdown(client).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_cue_file_validate_missing_file() -> Result<()> {
    let client = spawn_client().await?;

    let temp_dir = tempfile::Builder::new().tempdir()?;
    let cue_path = temp_dir.path().join("test.cue");
    let audio_path = temp_dir.path().join("existing.flac");

    std::fs::write(
        &cue_path,
        r#"PERFORMER "Artist"
TITLE "Album"
FILE "missing.flac" WAVE
  TRACK 01 AUDIO
    TITLE "Track"
    INDEX 01 00:00:00
"#,
    )?;
    std::fs::write(&audio_path, b"dummy audio")?;

    let result = call_tool(
        &client,
        "cue_file",
        object!({
            "path": cue_path.to_string_lossy(),
            "operation": "validate",
            "json_output": false
        }),
    )
    .await?;

    assert_ok(&result);

    let text = text_content(&result);
    assert!(text.contains("validation failed"));
    assert!(text.contains("missing"));

    shutdown(client).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_cue_file_validate_track_mismatch() -> Result<()> {
    let client = spawn_client().await?;

    let temp_dir = tempfile::Builder::new().tempdir()?;
    let cue_path = temp_dir.path().join("test.cue");
    let audio_path1 = temp_dir.path().join("track1.flac");
    let audio_path2 = temp_dir.path().join("track2.flac");
    let audio_path3 = temp_dir.path().join("track3.flac");

    std::fs::write(
        &cue_path,
        r#"PERFORMER "Artist"
TITLE "Album"
FILE "track1.flac" WAVE
  TRACK 01 AUDIO
    TITLE "Track 1"
    INDEX 01 00:00:00
FILE "track2.flac" WAVE
  TRACK 02 AUDIO
    TITLE "Track 2"
    INDEX 01 00:02:00
"#,
    )?;
    std::fs::write(&audio_path1, b"dummy audio")?;
    std::fs::write(&audio_path2, b"dummy audio")?;
    std::fs::write(&audio_path3, b"dummy audio")?;

    let result = call_tool(
        &client,
        "cue_file",
        object!({
            "path": cue_path.to_string_lossy(),
            "operation": "validate",
            "json_output": false
        }),
    )
    .await?;

    assert_ok(&result);

    let text = text_content(&result);
    assert!(text.contains("validation failed"));
    assert!(text.contains("Track count mismatch"));

    shutdown(client).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_cue_file_parse_nonexistent() -> Result<()> {
    let client = spawn_client().await?;

    let result = call_tool(
        &client,
        "cue_file",
        object!({
            "path": "/nonexistent/path/nonexistent.cue",
            "operation": "parse"
        }),
    )
    .await?;

    assert_err(&result);

    let text = text_content(&result);
    assert!(text.contains("Error parsing cue file"));

    shutdown(client).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_cue_file_validate_nonexistent() -> Result<()> {
    let client = spawn_client().await?;

    let result = call_tool(
        &client,
        "cue_file",
        object!({
            "path": "/nonexistent/path/nonexistent.cue",
            "operation": "validate"
        }),
    )
    .await?;

    assert_ok(&result);

    let text = text_content(&result);
    assert!(text.contains("validation failed"));

    shutdown(client).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_cue_file_validate_with_custom_audio_dir() -> Result<()> {
    let client = spawn_client().await?;

    let temp_dir = tempfile::Builder::new().tempdir()?;
    let audio_dir = temp_dir.path().join("audio");
    std::fs::create_dir_all(&audio_dir)?;

    let cue_path = temp_dir.path().join("test.cue");
    let audio_path = audio_dir.join("track.flac");

    std::fs::write(
        &cue_path,
        r#"PERFORMER "Artist"
TITLE "Album"
FILE "track.flac" WAVE
  TRACK 01 AUDIO
    TITLE "Track"
    INDEX 01 00:00:00
"#,
    )?;
    std::fs::write(&audio_path, b"dummy audio")?;

    let result = call_tool(
        &client,
        "cue_file",
        object!({
            "path": cue_path.to_string_lossy(),
            "operation": "validate",
            "audio_dir": audio_dir.to_string_lossy(),
            "json_output": false
        }),
    )
    .await?;

    assert_ok(&result);

    let text = text_content(&result);
    assert!(text.contains("CUE file is valid"));

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
    for expected in [
        "=== MUSIC LIBRARY METADATA ===",
        "Total Artists: 2",
        "Total Albums: 2",
        "Total Files: 2",
        "Total Tracks: 2",
        "ARTIST: flac",
        "ARTIST: Test Artist",
        "ALBUM: simple",
        "ALBUM: Test Album (2023)",
        "TRACK: \"Test Apply Behavior\" | Duration: 0:01 | File: tests/fixtures/flac/simple/track1.flac",
        "TRACK: \"[Unknown Title]\" | Duration: 0:01 | File: tests/fixtures/flac/simple/track2.flac",
    ] {
        assert!(text.contains(expected), "Expected text to contain: {}", expected);
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
    for expected in [
        "=== METADATA VALIDATION RESULTS ===",
        "📊 Summary:",
        "Total files: 2",
        "Valid files: 1",
        "Files with errors: 1",
        "Files with warnings: 1",
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

    assert_ok(&result);

    let text = text_content(&result);
    assert!(text.contains("=== METADATA VALIDATION RESULTS ==="));
    assert!(text.contains("Total files: 2"));
    assert!(text.contains("Files with errors: 0"));
    assert!(text.contains("Files with warnings: 0"));
    assert!(text.contains("✅ All files passed validation!"));

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
    assert!(stdout.contains("0.3."));
}

/* -------------------------- CUE file tests ------------------------- */

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_cue_file_generate_dry_run() -> Result<()> {
    let client = spawn_client().await?;

    let temp_dir = TempDir::new()?;
    let album_dir = temp_dir.path().join("Test Album");
    std::fs::create_dir_all(&album_dir)?;

    let track1 = album_dir.join("01. Track One.flac");
    let track2 = album_dir.join("02. Some.flac"); // Corrected extension
    std::fs::copy("tests/fixtures/flac/simple/track1.flac", &track1)?;
    std::fs::copy("tests/fixtures/flac/simple/track2.flac", &track2)?;

    let result = call_tool(
        &client,
        "cue_file",
        object!({
            "path": album_dir.to_string_lossy(),
            "operation": "generate",
            "dry_run": true,
            "force": false
        }),
    )
    .await?;

    assert_ok(&result);

    let text = text_content(&result);
    assert!(text.contains("Would write to:"));
    assert!(text.contains(".cue"));
    assert!(text.contains("PERFORMER"));
    assert!(text.contains("TRACK 01"));
    assert!(text.contains("TRACK 02"));

    shutdown(client).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_cue_file_generate_actual() -> Result<()> {
    let client = spawn_client().await?;

    let temp_dir = TempDir::new()?;
    let album_dir = temp_dir.path().join("Actual Album");
    std::fs::create_dir_all(&album_dir)?;

    let track1 = album_dir.join("01. Track.flac");
    std::fs::copy("tests/fixtures/flac/simple/track1.flac", &track1)?;

    let result = call_tool(
        &client,
        "cue_file",
        object!({
            "path": album_dir.to_string_lossy(),
            "operation": "generate",
            "dry_run": false,
            "force": true
        }),
    )
    .await?;

    assert_ok(&result);

    let text = text_content(&result);
    assert!(text.contains("Cue file written to:"));

    let cue_path = album_dir.join("Actual Album.cue");
    assert!(cue_path.exists());

    let cue_content = std::fs::read_to_string(&cue_path)?;
    assert!(cue_content.contains("PERFORMER"));
    assert!(cue_content.contains("TRACK 01 AUDIO"));

    shutdown(client).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_cue_file_parse() -> Result<()> {
    let client = spawn_client().await?;

    let result = call_tool(
        &client,
        "cue_file",
        object!({
            "path": "tests/fixtures/cue/album.cue",
            "operation": "parse",
            "json_output": true
        }),
    )
    .await?;

    assert_ok(&result);

    let json_text = text_content(&result);
    let json: serde_json::Value = serde_json::from_str(&json_text)?;

    assert_eq!(
        json.get("performer").unwrap().as_str().unwrap(),
        "Test Artist"
    );
    assert_eq!(json.get("title").unwrap().as_str().unwrap(), "Test Album");

    let files = json.get("files").unwrap().as_array().unwrap();
    assert_eq!(files.len(), 2);
    assert_eq!(files[0].as_str().unwrap(), "01. First Track.flac");
    assert_eq!(files[1].as_str().unwrap(), "02. Second Track.flac");

    let tracks = json.get("tracks").unwrap().as_array().unwrap();
    assert_eq!(tracks.len(), 2);
    assert_eq!(tracks[0].get("number").unwrap().as_u64().unwrap(), 1);
    assert_eq!(
        tracks[0].get("title").unwrap().as_str().unwrap(),
        "First Track"
    );
    assert_eq!(tracks[1].get("number").unwrap().as_u64().unwrap(), 2);
    assert_eq!(
        tracks[1].get("title").unwrap().as_str().unwrap(),
        "Second Track"
    );

    shutdown(client).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_generate_cue_file_no_files() -> Result<()> {
    let client = spawn_client().await?;

    let temp_dir = TempDir::new()?;
    let empty_dir = temp_dir.path().join("Empty Album");
    std::fs::create_dir_all(&empty_dir)?;

    let result = call_tool(
        &client,
        "cue_file",
        object!({
            "path": empty_dir.to_string_lossy(),
            "operation": "generate",
            "dry_run": false,
            "force": false
        }),
    )
    .await?;

    assert_err(&result);

    let text = text_content(&result);
    assert!(text.contains("No music files found"));

    shutdown(client).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_mcp_cue_file_parse_text_output() -> Result<()> {
    let client = spawn_client().await?;

    let temp_dir = TempDir::new()?;
    let cue_path = temp_dir.path().join("test_parse_text.cue");

    let cue_content = r#"TITLE "Example Album"
PERFORMER "Example Artist"
FILE "audio.flac" WAVE
  TRACK 01 AUDIO
    TITLE "Track One"
    PERFORMER "Example Artist"
    INDEX 01 00:00:00
  TRACK 02 AUDIO
    TITLE "Track Two"
    PERFORMER "Another Artist"
    INDEX 01 03:00:00"#;
    std::fs::write(&cue_path, cue_content)?;

    let result = call_tool(
        &client,
        "cue_file",
        object!({
            "path": cue_path.to_string_lossy(),
            "operation": "parse",
            "json_output": false
        }),
    )
    .await?;

    assert_ok(&result);

    let text = text_content(&result);
    assert!(text.contains("Cue File:"));
    assert!(text.contains("Performer: Example Artist"));
    assert!(text.contains("Title: Example Album"));
    assert!(text.contains("Track 01: Track One"));
    assert!(text.contains("Track 02: Track Two"));

    shutdown(client).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_mcp_cue_file_parse_invalid_syntax() -> Result<()> {
    let client = spawn_client().await?;

    let temp_dir = TempDir::new()?;
    let cue_path = temp_dir.path().join("invalid_syntax.cue");

    // Malformed CUE content (missing quote)
    let cue_content = r#"TITLE "Example Album"
PERFORMER Example Artist" // Missing quote
FILE "audio.flac" WAVE
  TRACK 01 AUDIO
    TITLE "Track One"
    INDEX 01 00:00:00"#;
    std::fs::write(&cue_path, cue_content)?;

    let result = call_tool(
        &client,
        "cue_file",
        object!({
            "path": cue_path.to_string_lossy(),
            "operation": "parse",
            "json_output": false
        }),
    )
    .await?;

    assert_err(&result);

    let text = text_content(&result);
    assert!(text.contains("Error parsing cue file"));
    assert!(text.contains("Malformed PERFORMER at line"));

    shutdown(client).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_mcp_cue_file_generate_exists_no_force() -> Result<()> {
    let client = spawn_client().await?;

    let temp_dir = TempDir::new()?;
    let album_dir = temp_dir.path().join("Existing Album");
    std::fs::create_dir_all(&album_dir)?;

    let track1 = album_dir.join("01. Song.flac");
    std::fs::copy("tests/fixtures/flac/simple/track1.flac", &track1)?;

    let cue_path = album_dir.join("Existing Album.cue");
    std::fs::write(&cue_path, "dummy cue content")?; // Pre-create the CUE file

    let result = call_tool(
        &client,
        "cue_file",
        object!({
            "path": album_dir.to_string_lossy(),
            "operation": "generate",
            "dry_run": false,
            "force": false
        }),
    )
    .await?;

    assert_err(&result);

    let text = text_content(&result);
    assert!(text.contains("Cue file already exists"));
    assert!(text.contains("Use force=true to overwrite"));

    shutdown(client).await
}