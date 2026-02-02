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
    OperationResult,
};

#[derive(Debug, serde::Serialize)]
pub struct ValidationResult {
    pub total_issues: usize,
    pub warnings: Vec<ValidationIssue>,
    pub errors: Vec<ValidationIssue>,
    pub info: Vec<ValidationIssue>,
}

#[derive(Debug, serde::Serialize)]
pub struct ValidationIssue {
    pub severity: String,
    pub category: String,
    pub message: String,
    pub details: Option<String>,
    pub file_path: Option<String>,
}

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
        let library = build_library_hierarchy(tracks);
        let validation_results = validate_library_structure(&library);

        let result = if json_output {
            serde_json::to_string_pretty(&validation_results)
                .map_err(|e| McpError::invalid_params(format!("JSON serialization error: {}", e), None))?
        } else {
            format_validation_results(&validation_results)
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

/// Validate library structure and metadata consistency
pub fn validate_library_structure(library: &Library) -> ValidationResult {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    let mut info = Vec::new();

    for artist in &library.artists {
        // Check for empty artist name
        if artist.name.trim().is_empty() {
            errors.push(ValidationIssue {
                severity: "error".to_string(),
                category: "artist".to_string(),
                message: "Artist name is empty".to_string(),
                details: None,
                file_path: None,
            });
        }

        // Check for duplicate albums within artist
        let mut album_titles = std::collections::HashSet::new();
        for album in &artist.albums {
            if album.title.trim().is_empty() {
                errors.push(ValidationIssue {
                    severity: "error".to_string(),
                    category: "album".to_string(),
                    message: "Album title is empty".to_string(),
                    details: Some(format!("Artist: {}", artist.name)),
                    file_path: None,
                });
                continue;
            }

            if album_titles.contains(&album.title.to_lowercase()) {
                warnings.push(ValidationIssue {
                    severity: "warning".to_string(),
                    category: "album".to_string(),
                    message: "Duplicate album title found".to_string(),
                    details: Some(format!("Artist: {}, Album: {}", artist.name, album.title)),
                    file_path: None,
                });
            } else {
                album_titles.insert(album.title.to_lowercase());
            }

            // Check for missing year
            if album.year.is_none() {
                warnings.push(ValidationIssue {
                    severity: "warning".to_string(),
                    category: "album".to_string(),
                    message: "Album missing year".to_string(),
                    details: Some(format!("Artist: {}, Album: {}", artist.name, album.title)),
                    file_path: None,
                });
            }

            // Validate tracks
            let mut track_numbers = std::collections::HashSet::new();
            for track in &album.tracks {
                // Check for missing metadata
                if track.metadata.title.is_none() {
                    errors.push(ValidationIssue {
                        severity: "error".to_string(),
                        category: "track".to_string(),
                        message: "Track missing title".to_string(),
                        details: Some(format!("Artist: {}, Album: {}", artist.name, album.title)),
                        file_path: Some(track.file_path.to_string_lossy().to_string()),
                    });
                }

                if track.metadata.track_number.is_none() {
                    warnings.push(ValidationIssue {
                        severity: "warning".to_string(),
                        category: "track".to_string(),
                        message: "Track missing track number".to_string(),
                        details: Some(format!("Artist: {}, Album: {}, File: {}", 
                            artist.name, album.title, track.file_path.display())),
                        file_path: Some(track.file_path.to_string_lossy().to_string()),
                    });
                } else if let Some(track_num) = &track.metadata.track_number {
                    // Check for duplicate track numbers
                    if track_numbers.contains(&track_num.value) {
                        warnings.push(ValidationIssue {
                            severity: "warning".to_string(),
                            category: "track".to_string(),
                            message: "Duplicate track number".to_string(),
                            details: Some(format!("Artist: {}, Album: {}, Track: {}", 
                                artist.name, album.title, track_num.value)),
                            file_path: Some(track.file_path.to_string_lossy().to_string()),
                        });
                    } else {
                        track_numbers.insert(track_num.value);
                    }
                }

                if track.metadata.artist.is_none() {
                    warnings.push(ValidationIssue {
                        severity: "warning".to_string(),
                        category: "track".to_string(),
                        message: "Track missing artist metadata".to_string(),
                        details: Some(format!("Artist: {}, Album: {}, File: {}", 
                            artist.name, album.title, track.file_path.display())),
                        file_path: Some(track.file_path.to_string_lossy().to_string()),
                    });
                }

                if track.metadata.album.is_none() {
                    warnings.push(ValidationIssue {
                        severity: "warning".to_string(),
                        category: "track".to_string(),
                        message: "Track missing album metadata".to_string(),
                        details: Some(format!("Artist: {}, Album: {}, File: {}", 
                            artist.name, album.title, track.file_path.display())),
                        file_path: Some(track.file_path.to_string_lossy().to_string()),
                    });
                }

                // Check for unusually short or long tracks
                if let Some(duration) = &track.metadata.duration {
                    if duration.value < 10.0 {
                        warnings.push(ValidationIssue {
                            severity: "warning".to_string(),
                            category: "track".to_string(),
                            message: "Very short track (less than 10 seconds)".to_string(),
                            details: Some(format!("Duration: {}s, File: {}", 
                                duration.value, track.file_path.display())),
                            file_path: Some(track.file_path.to_string_lossy().to_string()),
                        });
                    } else if duration.value > 3600.0 {
                        warnings.push(ValidationIssue {
                            severity: "warning".to_string(),
                            category: "track".to_string(),
                            message: "Very long track (more than 1 hour)".to_string(),
                            details: Some(format!("Duration: {}s, File: {}", 
                                duration.value, track.file_path.display())),
                            file_path: Some(track.file_path.to_string_lossy().to_string()),
                        });
                    }
                }
            }

            // Check for empty albums
            if album.tracks.is_empty() {
                errors.push(ValidationIssue {
                    severity: "error".to_string(),
                    category: "album".to_string(),
                    message: "Album contains no tracks".to_string(),
                    details: Some(format!("Artist: {}, Album: {}", artist.name, album.title)),
                    file_path: None,
                });
            }
        }

        // Check for artists with no albums
        if artist.albums.is_empty() {
            warnings.push(ValidationIssue {
                severity: "warning".to_string(),
                category: "artist".to_string(),
                message: "Artist has no albums".to_string(),
                details: Some(format!("Artist: {}", artist.name)),
                file_path: None,
            });
        }
    }

    // Add summary info
    info.push(ValidationIssue {
        severity: "info".to_string(),
        category: "summary".to_string(),
        message: format!("Library validation complete: {} artists, {} albums, {} tracks", 
            library.total_artists, library.total_albums, library.total_tracks),
        details: None,
        file_path: None,
    });

    let total_issues = warnings.len() + errors.len();

    ValidationResult {
        total_issues,
        warnings,
        errors,
        info,
    }
}

/// Format validation results for display
pub fn format_validation_results(results: &ValidationResult) -> String {
    let mut output = String::new();

    output.push_str("=== MUSIC LIBRARY VALIDATION ===\n");
    output.push_str(&format!("Total Issues Found: {}\n\n", results.total_issues));

    if !results.errors.is_empty() {
        output.push_str(&format!("🔴 ERRORS ({}):\n", results.errors.len()));
        for error in &results.errors {
            output.push_str(&format!("  [{}] {}: {}\n", 
                error.category.to_uppercase(), error.message, 
                error.details.as_ref().unwrap_or(&String::new())));
            if let Some(file) = &error.file_path {
                output.push_str(&format!("    File: {}\n", file));
            }
        }
        output.push('\n');
    }

    if !results.warnings.is_empty() {
        output.push_str(&format!("🟡 WARNINGS ({}):\n", results.warnings.len()));
        for warning in &results.warnings {
            output.push_str(&format!("  [{}] {}: {}\n", 
                warning.category.to_uppercase(), warning.message,
                warning.details.as_ref().unwrap_or(&String::new())));
            if let Some(file) = &warning.file_path {
                output.push_str(&format!("    File: {}\n", file));
            }
        }
        output.push('\n');
    }

    if !results.info.is_empty() {
        output.push_str(&format!("ℹ️  INFO ({}):\n", results.info.len()));
        for info in &results.info {
            output.push_str(&format!("  {}\n", info.message));
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