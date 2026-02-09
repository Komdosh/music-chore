use crate::core::services::cue::{
    CueGenerationError, CueValidationResult, format_cue_validation_result, generate_cue_for_path,
    parse_cue_file, validate_cue_consistency,
};
use rmcp::{ErrorData as McpError, model::CallToolResult};
use std::path::{Path, PathBuf};
use crate::mcp::call_tool_result::CallToolResultExt;

pub(crate) async fn handle_cue_generate(
    path: &Path,
    output: Option<PathBuf>,
    dry_run: bool,
    force: bool,
) -> Result<CallToolResult, McpError> {
    match generate_cue_for_path(path, output) {
        Ok(result) => {
            if !dry_run && result.output_path.exists() && !force {
                return Ok(CallToolResult::error_text(format!(
                    "Cue file already exists at '{}'. Use force=true to overwrite.",
                    result.output_path.display()
                )));
            }

            if dry_run {
                Ok(CallToolResult::success_text(format!(
                    "Would write to: {}\n\n{}",
                    result.output_path.display(),
                    result.cue_content
                )))
            } else {
                match std::fs::write(&result.output_path, &result.cue_content) {
                    Ok(_) => Ok(CallToolResult::success_text(format!(
                        "Cue file written to: {}",
                        result.output_path.display()
                    ))),
                    Err(e) => Ok(CallToolResult::error_text(format!(
                        "Error writing cue file: {e}"
                    ))),
                }
            }
        }
        Err(CueGenerationError::NoMusicFiles) => Ok(CallToolResult::error_text(
            "No music files found in directory (checked only immediate files, not subdirectories)",
        )),
        Err(CueGenerationError::NoReadableFiles) => Ok(CallToolResult::error_text(
            "No readable music files found in directory",
        )),
        Err(CueGenerationError::FileReadError(msg)) => Ok(CallToolResult::error_text(msg)),
    }
}

pub(crate) async fn handle_cue_parse(
    path: &Path,
    json_output: bool,
) -> Result<CallToolResult, McpError> {
    match parse_cue_file(path) {
        Ok(cue_file) => {
            if json_output {
                let result = crate::mcp::music_chore_server_impl::to_json_pretty(&cue_file)?;
                Ok(CallToolResult::success_text(result))
            } else {
                let mut output = format!("Cue File: {}\n", path.display());
                if let Some(performer) = &cue_file.performer {
                    output.push_str(&format!("  Performer: {performer}\n"));
                }
                if let Some(title) = &cue_file.title {
                    output.push_str(&format!("  Title: {title}\n"));
                }
                if !cue_file.files.is_empty() {
                    output.push_str("  Files:\n");
                    for file in &cue_file.files {
                        output.push_str(&format!("    - {file}\n"));
                    }
                }
                output.push_str(&format!("  Tracks: {}\n", cue_file.tracks.len()));
                for track in &cue_file.tracks {
                    let file_info = track
                        .file
                        .as_ref()
                        .map(|f| format!(" [{f}]"))
                        .unwrap_or_default();
                    output.push_str(&format!(
                        "    Track {:02}: {}{}\n",
                        track.number,
                        track.title.as_deref().unwrap_or("(no title)"),
                        file_info
                    ));
                }
                Ok(CallToolResult::success_text(output))
            }
        }
        Err(e) => Ok(CallToolResult::error_text(format!(
            "Error parsing cue file: {e}"
        ))),
    }
}

pub(crate) async fn handle_cue_validate(
    path: &Path,
    audio_dir: Option<PathBuf>,
    json_output: bool,
) -> Result<CallToolResult, McpError> {
    let audio_directory = audio_dir.unwrap_or_else(|| {
        path.parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    });

    let audio_files: Vec<PathBuf> = match std::fs::read_dir(&audio_directory) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            .filter(|e| {
                !e.path()
                    .extension()
                    .map(|ext| ext == "cue")
                    .unwrap_or(false)
            })
            .map(|e| e.path())
            .collect(),
        Err(_) => {
            let result = CueValidationResult {
                is_valid: false,
                file_missing: true,
                ..Default::default()
            };
            let result_str = if json_output {
                crate::mcp::music_chore_server_impl::to_json_pretty(&result)?
            } else {
                format_cue_validation_result(&result)
            };
            return Ok(CallToolResult::success_text(result_str));
        }
    };

    let audio_files_refs: Vec<&Path> = audio_files.iter().map(|p| p.as_path()).collect();
    let result = validate_cue_consistency(path, &audio_files_refs);

    let output = if json_output {
        crate::mcp::music_chore_server_impl::to_json_pretty(&result)?
    } else {
        format_cue_validation_result(&result)
    };
    Ok(CallToolResult::success_text(output))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_handle_cue_generate_dry_run() {
        let temp_dir = TempDir::new().unwrap();
        let album_dir = temp_dir.path().join("Album");
        fs::create_dir_all(&album_dir).unwrap();
        
        // Need some audio files for generate to work
        let track1 = album_dir.join("01. Track 1.flac");
        fs::copy("tests/fixtures/flac/simple/track1.flac", &track1).unwrap();

        let result = handle_cue_generate(&album_dir, None, true, false).await.expect("Should succeed");
        assert!(!result.is_error.unwrap_or(false));
        let text = result.content[0].raw.as_text().unwrap().text.as_str();
        assert!(text.contains("Would write to:"));
        assert!(text.contains("PERFORMER"));
        assert!(text.contains("TRACK 01"));
    }

    #[tokio::test]
    async fn test_handle_cue_generate_actual() {
        let temp_dir = TempDir::new().unwrap();
        let album_dir = temp_dir.path().join("Album");
        fs::create_dir_all(&album_dir).unwrap();
        
        let track1 = album_dir.join("01. Track 1.flac");
        fs::copy("tests/fixtures/flac/simple/track1.flac", &track1).unwrap();

        let result = handle_cue_generate(&album_dir, None, false, false).await.expect("Should succeed");
        assert!(!result.is_error.unwrap_or(false));
        let text = result.content[0].raw.as_text().unwrap().text.as_str();
        assert!(text.contains("Cue file written to:"));
        
        let cue_path = album_dir.join("Album.cue");
        assert!(cue_path.exists());
        let content = fs::read_to_string(cue_path).unwrap();
        assert!(content.contains("PERFORMER"));
        assert!(content.contains("TRACK 01"));
    }

    #[tokio::test]
    async fn test_handle_cue_generate_no_music_files() {
        let temp_dir = TempDir::new().unwrap();
        let empty_dir = temp_dir.path().join("Empty");
        fs::create_dir_all(&empty_dir).unwrap();

        let result = handle_cue_generate(&empty_dir, None, false, false).await.expect("Should return error Result");
        assert!(result.is_error.unwrap_or(false));
        let text = result.content[0].raw.as_text().unwrap().text.as_str();
        assert!(text.contains("No music files found"));
    }

    #[tokio::test]
    async fn test_handle_cue_parse_json() {
        let cue_path = Path::new("tests/fixtures/cue/album.cue");
        let result = handle_cue_parse(cue_path, true).await.expect("Should succeed");
        assert!(!result.is_error.unwrap_or(false));
        let json_text = result.content[0].raw.as_text().unwrap().text.as_str();
        let json: serde_json::Value = serde_json::from_str(json_text).unwrap();
        assert_eq!(json["performer"], "Test Artist");
        assert_eq!(json["title"], "Test Album");
    }

    #[tokio::test]
    async fn test_handle_cue_parse_text() {
        let cue_path = Path::new("tests/fixtures/cue/album.cue");
        let result = handle_cue_parse(cue_path, false).await.expect("Should succeed");
        assert!(!result.is_error.unwrap_or(false));
        let text = result.content[0].raw.as_text().unwrap().text.as_str();
        assert!(text.contains("Cue File: tests/fixtures/cue/album.cue"));
        assert!(text.contains("Performer: Test Artist"));
        assert!(text.contains("Title: Test Album"));
        assert!(text.contains("Track 01: First Track"));
    }

    #[tokio::test]
    async fn test_handle_cue_validate_valid() {
        let temp_dir = TempDir::new().unwrap();
        let cue_path = temp_dir.path().join("test.cue");
        let audio_path = temp_dir.path().join("track1.flac");
        
        fs::write(&cue_path, "FILE \"track1.flac\" WAVE\n  TRACK 01 AUDIO\n    INDEX 01 00:00:00").unwrap();
        fs::write(&audio_path, b"dummy").unwrap();

        let result = handle_cue_validate(&cue_path, None, false).await.expect("Should succeed");
        assert!(!result.is_error.unwrap_or(false));
        let text = result.content[0].raw.as_text().unwrap().text.as_str();
        assert!(text.contains("CUE file is valid"));
    }

    #[tokio::test]
    async fn test_handle_cue_validate_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let cue_path = temp_dir.path().join("test.cue");
        
        fs::write(&cue_path, "FILE \"missing.flac\" WAVE\n  TRACK 01 AUDIO\n    INDEX 01 00:00:00").unwrap();

        let result = handle_cue_validate(&cue_path, None, true).await.expect("Should succeed");
        assert!(!result.is_error.unwrap_or(false));
        let json_text = result.content[0].raw.as_text().unwrap().text.as_str();
        let json: serde_json::Value = serde_json::from_str(json_text).unwrap();
        assert_eq!(json["is_valid"], false);
        assert_eq!(json["file_missing"], true);
    }
}
