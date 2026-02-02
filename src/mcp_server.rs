//! MCP Server for Music Chore CLI Tool
//! 
//! Provides Model Context Protocol interface for AI agents to interact
//! with the music library management functionality.

use rmcp::{
    ServiceExt,
    handler::server::{tool::ToolRouter, ServerHandler, wrapper::Parameters},
    model::{ServerInfo, ServerCapabilities, Implementation, ProtocolVersion, CallToolResult, Content},
    tool, tool_handler, tool_router,
    transport::stdio,
    ErrorData as McpError,
};
use log;
use std::path::PathBuf;

use crate::{
    build_library_hierarchy, normalize_track_titles, read_metadata, scan_dir, Library,
    OperationResult, cli::commands::validate_tracks,
};

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ScanDirectoryParams {
    path: String,
    json_output: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetLibraryTreeParams {
    path: String,
    json_output: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ReadFileMetadataParams {
    file_path: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NormalizeTitlesParams {
    path: String,
    dry_run: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EmitLibraryMetadataParams {
    path: String,
    json_output: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ValidateLibraryParams {
    path: String,
    json_output: Option<bool>,
}

#[derive(Clone)]
pub struct MusicChoreServer {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl MusicChoreServer {
    fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Recursively scan a directory for music files")]
    async fn scan_directory(&self, params: Parameters<ScanDirectoryParams>) -> Result<CallToolResult, McpError> {
        let path = PathBuf::from(params.0.path);
        let json_output = params.0.json_output.unwrap_or(false);
        let tracks = scan_dir(&path);

        if tracks.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(
                format!("No music files found in directory: {}", path.display())
            )]));
        }

        let result = if json_output {
            serde_json::to_string_pretty(&tracks)
                .map_err(|e| McpError::invalid_params(format!("JSON serialization error: {}", e), None))?
        } else {
            tracks.iter()
                .map(|track| track.file_path.display().to_string())
                .collect::<Vec<_>>()
                .join("\n")
        };

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Get hierarchical library tree view")]
    async fn get_library_tree(&self, params: Parameters<GetLibraryTreeParams>) -> Result<CallToolResult, McpError> {
        let path = PathBuf::from(params.0.path);
        let _json_output = params.0.json_output.unwrap_or(false);
        let tracks = scan_dir(&path);
        let library = build_library_hierarchy(tracks);
        
        log::info!("get_library_tree called with path: {}", path.display());

        let result = serde_json::to_string_pretty(&library)
            .map_err(|e| McpError::invalid_params(format!("JSON serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Read metadata from a single music file")]
    async fn read_file_metadata(&self, params: Parameters<ReadFileMetadataParams>) -> Result<CallToolResult, McpError> {
        let file = PathBuf::from(params.0.file_path);
        
        log::info!("read_file_metadata called with file_path: {}", file.display());

        match read_metadata(&file) {
            Ok(track) => {
                let result = serde_json::to_string_pretty(&track)
                    .map_err(|e| McpError::invalid_params(format!("JSON serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(result)]))
            }
            Err(e) => {
                Ok(CallToolResult::error(vec![Content::text(format!("Error reading metadata: {}", e))]))
            }
        }
    }

    #[tool(description = "Normalize track titles to title case")]
    async fn normalize_titles(&self, params: Parameters<NormalizeTitlesParams>) -> Result<CallToolResult, McpError> {
        let path = PathBuf::from(params.0.path);
        let _dry_run = params.0.dry_run.unwrap_or(false);
        
        log::info!("normalize_titles called with path: {}, dry_run: {}", path.display(), _dry_run);

        match normalize_track_titles(&path) {
            Ok(results) => {
                let mut output = Vec::new();

                for result in results {
                    match result {
                        OperationResult::Updated {
                            track,
                            old_title,
                            new_title,
                        } => {
                            output.push(format!(
                                "NORMALIZED: '{}' -> '{}' in {}",
                                track.file_path.display(),
                                old_title,
                                new_title
                            ));
                        }
                        OperationResult::NoChange { track } => {
                            output.push(format!(
                                "NO CHANGE: Title already title case in {}",
                                track.file_path.display()
                            ));
                        }
                        OperationResult::Error { track, error } => {
                            output.push(format!("ERROR: {} in {}", error, track.file_path.display()));
                        }
                    }
                }

                Ok(CallToolResult::success(vec![Content::text(output.join("\n"))]))
            }
            Err(e) => {
                Ok(CallToolResult::error(vec![Content::text(format!("Error normalizing titles: {}", e))]))
            }
        }
    }

    #[tool(description = "Emit library metadata in structured format")]
    async fn emit_library_metadata(&self, params: Parameters<EmitLibraryMetadataParams>) -> Result<CallToolResult, McpError> {
        let path = PathBuf::from(params.0.path);
        let json_output = params.0.json_output.unwrap_or(false);
        let tracks = scan_dir(&path);
        
        log::info!("emit_library_metadata called with path: {}", path.display());
        let library = build_library_hierarchy(tracks);

        let result = if json_output {
            serde_json::to_string_pretty(&library)
                .map_err(|e| McpError::invalid_params(format!("JSON serialization error: {}", e), None))?
        } else {
            format_structured_metadata(&library)
        };

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Validate music library for common issues and inconsistencies")]
    async fn validate_library(&self, params: Parameters<ValidateLibraryParams>) -> Result<CallToolResult, McpError> {
        let path = PathBuf::from(params.0.path);
        let json_output = params.0.json_output.unwrap_or(false);
        
        log::info!("validate_library called with path: {}", path.display());
        
        let tracks = scan_dir(&path);
        let total_scanned = tracks.len();

        if tracks.is_empty() {
            let result = if json_output {
                "{\"valid\": true, \"errors\": [], \"warnings\": [], \"summary\": {\"total_files\": 0, \"valid_files\": 0, \"files_with_errors\": 0, \"files_with_warnings\": 0}}"
            } else {
                "No music files found to validate."
            };
            return Ok(CallToolResult::success(vec![Content::text(result.to_string())]));
        }

        // For validation, we need to read actual metadata from each file
        let tracks_with_metadata: Vec<crate::Track> = tracks
            .into_iter()
            .filter_map(|track| {
                match read_metadata(&track.file_path) {
                    Ok(track_with_metadata) => Some(track_with_metadata),
                    Err(_) => None, // Skip files that can't be read
                }
            })
            .collect();

        if tracks_with_metadata.is_empty() {
            let result = if json_output {
                format!("{{\"valid\": false, \"errors\": [], \"warnings\": [], \"summary\": {{\"total_files\": {}, \"valid_files\": 0, \"files_with_errors\": {}, \"files_with_warnings\": 0}}}}", 
                    total_scanned, total_scanned)
            } else {
                "Unable to read metadata from any files for validation.".to_string()
            };
            return Ok(CallToolResult::success(vec![Content::text(result)]));
        }

        let validation_results = validate_tracks(tracks_with_metadata);

        let result = if json_output {
            serde_json::to_string_pretty(&validation_results)
                .map_err(|e| McpError::invalid_params(format!("JSON serialization error: {}", e), None))?
        } else {
            format_cli_validation_results(&validation_results)
        };

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
}

/// Format structured metadata for AI agents
fn format_structured_metadata(library: &Library) -> String {
    let mut output = String::new();

    output.push_str("=== MUSIC LIBRARY METADATA ===\n");
    output.push_str(&format!("Total Artists: {}\n", library.total_artists));
    output.push_str(&format!("Total Albums: {}\n", library.total_albums));
    output.push_str(&format!("Total Tracks: {}\n\n", library.total_tracks));

    for artist in &library.artists {
        output.push_str(&format!("ARTIST: {}\n", artist.name));

        for album in &artist.albums {
            let year_str = album.year.map(|y| format!(" ({})", y)).unwrap_or_default();
            output.push_str(&format!("  ALBUM: {}{}\n", album.title, year_str));

            for track in &album.tracks {
                let title = track
                    .metadata
                    .title
                    .as_ref()
                    .map(|t| t.value.as_str())
                    .unwrap_or("[Unknown Title]");
                let duration = track
                    .metadata
                    .duration
                    .as_ref()
                    .map(|d| {
                        let total_seconds = d.value as u64;
                        let minutes = total_seconds / 60;
                        let seconds = total_seconds % 60;
                        format!("{}:{:02}", minutes, seconds)
                    })
                    .unwrap_or_else(|| "0:00".to_string());
                let file_path = track.file_path.to_string_lossy();

                output.push_str(&format!(
                    "    TRACK: \"{}\" | Duration: {} | File: {}\n",
                    title, duration, file_path
                ));
            }
        }
        output.push('\n');
    }

    output.push_str("=== END METADATA ===\n");
    output
}

/// Format CLI validation results for MCP output (compatible with existing CLI format)
fn format_cli_validation_results(results: &crate::cli::commands::ValidationResult) -> String {
    let mut output = String::new();

    output.push_str("=== MUSIC LIBRARY VALIDATION ===\n");
    output.push_str(&format!("📊 Summary:\n"));
    output.push_str(&format!("  Total files: {}\n", results.summary.total_files));
    output.push_str(&format!("  Valid files: {}\n", results.summary.valid_files));
    output.push_str(&format!("  Files with errors: {}\n", results.summary.files_with_errors));
    output.push_str(&format!("  Files with warnings: {}\n\n", results.summary.files_with_warnings));

    if results.valid {
        output.push_str("✅ All files passed validation!\n");
    } else {
        output.push_str(&format!("❌ Validation failed with {} errors\n\n", results.errors.len()));
    }

    if !results.errors.is_empty() {
        output.push_str("🔴 ERRORS:\n");
        for error in &results.errors {
            output.push_str(&format!("  File: {}\n", error.file_path));
            output.push_str(&format!("  Field: {}\n", error.field));
            output.push_str(&format!("  Issue: {}\n\n", error.message));
        }
    }

    if !results.warnings.is_empty() {
        output.push_str("🟡 WARNINGS:\n");
        for warning in &results.warnings {
            output.push_str(&format!("  File: {}\n", warning.file_path));
            output.push_str(&format!("  Field: {}\n", warning.field));
            output.push_str(&format!("  Issue: {}\n\n", warning.message));
        }
    }

    output.push_str("=== END VALIDATION ===\n");
    output
}

#[tool_handler]
impl ServerHandler for MusicChoreServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation {
                name: "music-chore".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                ..Default::default()
            },
            instructions: Some("Music Chore CLI - Music library metadata management tool".into()),
            ..Default::default()
        }
    }
}

/// Start MCP server with stdio transport
pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
    let server = MusicChoreServer::new();
    
    // Run the server with stdio transport
    let service = server.serve(stdio()).await.inspect_err(|e| {
        println!("Error starting server: {}", e);
    })?;
    service.waiting().await?;
    
    Ok(())
}