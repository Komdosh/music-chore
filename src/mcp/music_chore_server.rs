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
    EmitLibraryMetadataParams, FindDuplicatesParams, GetLibraryTreeParams, NormalizeTitlesParams, ReadFileMetadataParams,
    ScanDirectoryParams, ValidateLibraryParams,
};
use crate::services::{formats::read_metadata, scanner::{scan_dir, scan_with_duplicates}};
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

    #[tool(description = "Find duplicate tracks by checksum")]
    async fn find_duplicates(
        &self,
        params: Parameters<FindDuplicatesParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = PathBuf::from(params.0.path);
        let json_output = params.0.json_output.unwrap_or(false);

        log::info!("find_duplicates called with path: {}", path.display());

        let (tracks, duplicates) = scan_with_duplicates(&path);

        if tracks.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "No music files found in directory: {}",
                path.display()
            ))]));
        }

        let result = if duplicates.is_empty() {
            "No duplicate tracks found.".to_string()
        } else if json_output {
            serde_json::to_string_pretty(&duplicates).map_err(|e| {
                McpError::invalid_params(format!("JSON serialization error: {}", e), None)
            })?
        } else {
            let mut output = format!("Found {} duplicate groups:\n\n", duplicates.len());
            
            for (i, duplicate_group) in duplicates.iter().enumerate() {
                output.push_str(&format!("Duplicate Group {} ({} files):\n", i + 1, duplicate_group.len()));
                for track in duplicate_group {
                    output.push_str(&format!("  {}\n", track.file_path.display()));
                }
                output.push('\n');
            }
            
            output
        };

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
