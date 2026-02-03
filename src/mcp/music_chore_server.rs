use log;
use rmcp::{
    handler::server::{tool::ToolRouter, wrapper::Parameters, ServerHandler},
    model::{
        CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    tool,
    tool_handler, tool_router, ErrorData as McpError,
};
use std::path::PathBuf;
use crate::cli::commands::validate_path;
use crate::mcp::params::{
    EmitLibraryMetadataParams, FindDuplicatesParams, GetLibraryTreeParams, NormalizeTitlesParams,
    ReadFileMetadataParams, ScanDirectoryParams, ValidateLibraryParams,
};
use crate::services::duplicates::find_duplicates;
use crate::services::format_tree::emit_by_path;
use crate::services::library::build_library_hierarchy;
use crate::services::{formats::read_metadata, normalization::normalize, scanner::scan_dir};
use crate::services::scanner::scan_tracks;

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

        match scan_tracks(path, json_output){
            Ok(result) => Ok(CallToolResult::success(vec![Content::text(result)])),
            Err(error) => Ok(CallToolResult::error(vec![Content::text(error)]))
        }
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
                    McpError::internal_error(format!("JSON serialization error: {}", e), None)
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
        let dry_run = params.0.dry_run.unwrap_or(false);

        log::info!(
            "normalize_titles called with path: {}, dry_run: {}",
            path.display(),
            dry_run
        );

        match normalize(path, dry_run) {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(output)])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(description = "Emit library metadata in structured format")]
    async fn emit_library_metadata(
        &self,
        params: Parameters<EmitLibraryMetadataParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = PathBuf::from(params.0.path);
        let json_output = params.0.json_output.unwrap_or(false);

        log::info!("emit_library_metadata called with path: {}", path.display());

        match emit_by_path(&path, json_output) {
            Ok(result) => Ok(CallToolResult::success(vec![Content::text(result)])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(description = "Validate music library for common issues and inconsistencies")]
    async fn validate_library(
        &self,
        params: Parameters<ValidateLibraryParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = PathBuf::from(params.0.path);
        let json_output = params.0.json_output.unwrap_or(false);

        log::info!("validate_library called with path: {}", path.display());

        return match validate_path(&path, json_output) {
            Ok(result) => Ok(CallToolResult::success(vec![Content::text(result)])),
            Err(result) => Ok(CallToolResult::error(vec![Content::text(result)])),
        };
    }

    #[tool(description = "Find duplicate tracks by checksum")]
    async fn find_duplicates(
        &self,
        params: Parameters<FindDuplicatesParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = PathBuf::from(params.0.path);
        let json_output = params.0.json_output.unwrap_or(false);

        log::info!("find_duplicates called with path: {}", path.display());

        return match find_duplicates(&path, json_output) {
            Ok(result) => Ok(CallToolResult::success(vec![Content::text(result)])),
            Err(result) => Ok(CallToolResult::error(vec![Content::text(result)])),
        };
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
