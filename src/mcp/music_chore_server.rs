use log;
use rmcp::{
    ErrorData as McpError,
    handler::server::{ServerHandler, tool::ToolRouter, wrapper::Parameters},
    model::{
        CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    tool, tool_handler, tool_router,
};
use std::path::PathBuf;

use crate::cli::commands::{format_tree_output, handle_validate};
use crate::mcp::params::{
    EmitLibraryMetadataParams, GetLibraryTreeParams, NormalizeTitlesParams, ReadFileMetadataParams,
    ScanDirectoryParams, ValidateLibraryParams,
};
use crate::services::{formats::read_metadata, scanner::scan_dir};
use crate::{OperationResult, build_library_hierarchy, normalize_track_titles};

#[derive(Clone)]
pub struct MusicChoreServer {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl MusicChoreServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Recursively scan a directory for music files")]
    async fn scan_directory(
        &self,
        params: Parameters<ScanDirectoryParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = PathBuf::from(params.0.path);
        let json_output = params.0.json_output.unwrap_or(false);
        let tracks = scan_dir(&path);

        if tracks.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "No music files found in directory: {}",
                path.display()
            ))]));
        }

        let result = if json_output {
            serde_json::to_string_pretty(&tracks).map_err(|e| {
                McpError::invalid_params(format!("JSON serialization error: {}", e), None)
            })?
        } else {
            tracks
                .iter()
                .map(|track| track.file_path.display().to_string())
                .collect::<Vec<_>>()
                .join("\n")
        };

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Get hierarchical library tree view")]
    async fn get_library_tree(
        &self,
        params: Parameters<GetLibraryTreeParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = PathBuf::from(params.0.path);
        let _json_output = params.0.json_output.unwrap_or(false);
        let tracks = scan_dir(&path);
        let library = build_library_hierarchy(tracks);

        log::info!("get_library_tree called with path: {}", path.display());

        let result = serde_json::to_string_pretty(&library).map_err(|e| {
            McpError::invalid_params(format!("JSON serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Read metadata from a single music file")]
    async fn read_file_metadata(
        &self,
        params: Parameters<ReadFileMetadataParams>,
    ) -> Result<CallToolResult, McpError> {
        let file = PathBuf::from(params.0.file_path);

        log::info!(
            "read_file_metadata called with file_path: {}",
            file.display()
        );

        match read_metadata(&file) {
            Ok(track) => {
                let result = serde_json::to_string_pretty(&track).map_err(|e| {
                    McpError::invalid_params(format!("JSON serialization error: {}", e), None)
                })?;
                Ok(CallToolResult::success(vec![Content::text(result)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Error reading metadata: {}",
                e
            ))])),
        }
    }

    #[tool(description = "Normalize track titles to title case")]
    async fn normalize_titles(
        &self,
        params: Parameters<NormalizeTitlesParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = PathBuf::from(params.0.path);
        let _dry_run = params.0.dry_run.unwrap_or(false);

        log::info!(
            "normalize_titles called with path: {}, dry_run: {}",
            path.display(),
            _dry_run
        );

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
                            output.push(format!(
                                "ERROR: {} in {}",
                                error,
                                track.file_path.display()
                            ));
                        }
                    }
                }

                Ok(CallToolResult::success(vec![Content::text(
                    output.join("\n"),
                )]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Error normalizing titles: {}",
                e
            ))])),
        }
    }

    #[tool(description = "Emit library metadata in structured format")]
    async fn emit_library_metadata(
        &self,
        params: Parameters<EmitLibraryMetadataParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = PathBuf::from(params.0.path);
        let json_output = params.0.json_output.unwrap_or(false);
        let tracks = scan_dir(&path);

        log::info!("emit_library_metadata called with path: {}", path.display());
        let library = build_library_hierarchy(tracks);

        let result = if json_output {
            serde_json::to_string_pretty(&library).map_err(|e| {
                McpError::invalid_params(format!("JSON serialization error: {}", e), None)
            })?
        } else {
            // Use MCP tree formatting logic
            format_tree_output(&library)
        };

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Validate music library for common issues and inconsistencies")]
    async fn validate_library(
        &self,
        params: Parameters<ValidateLibraryParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = PathBuf::from(params.0.path);
        let json_output = params.0.json_output.unwrap_or(false);

        log::info!("validate_library called with path: {}", path.display());

        let result = handle_validate(path, json_output);

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
}

#[tool_handler]
impl ServerHandler for MusicChoreServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
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
