use crate::mcp::params::{
    CueParams, EmitLibraryMetadataParams, FindDuplicatesParams, GetLibraryTreeParams,
    NormalizeTitlesParams, ReadFileMetadataParams, ScanDirectoryParams, ValidateLibraryParams,
};

use log;
use rmcp::{
    ErrorData as McpError,
    handler::server::{ServerHandler, tool::ToolRouter, wrapper::Parameters},
    model::{
        CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    tool, tool_handler, tool_router,
};
use std::path::{Path, PathBuf};
use crate::adapters::audio_formats::read_metadata;
use crate::build_library_hierarchy;
use crate::core::services::cue::{format_cue_validation_result, generate_cue_for_path, parse_cue_file, validate_cue_consistency, CueGenerationError, CueValidationResult};
use crate::core::services::duplicates::find_duplicates;
use crate::core::services::format_tree::emit_by_path;
use crate::core::services::normalization::normalize;
use crate::core::services::scanner::{scan_dir, scan_tracks};
use crate::presentation::cli::commands::validate_path;

#[derive(Clone)]
pub struct MusicChoreServer {
    tool_router: ToolRouter<Self>,
}

impl Default for MusicChoreServer {
    fn default() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl MusicChoreServer {
    pub fn new() -> Self {
        Self::default()
    }

    #[tool(description = "Recursively scan a directory for music files")]
    async fn scan_directory(
        &self,
        params: Parameters<ScanDirectoryParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = PathBuf::from(params.0.path);
        let json_output = params.0.json_output.unwrap_or(false);

        match scan_tracks(path, json_output) {
            Ok(result) => Ok(CallToolResult::success(vec![Content::text(result)])),
            Err(error) => Ok(CallToolResult::error(vec![Content::text(error)])),
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

    #[tool(description = "Generate, parse, or validate .cue files")]
    async fn cue_file(&self, params: Parameters<CueParams>) -> Result<CallToolResult, McpError> {
        let path = PathBuf::from(params.0.path);
        let operation = params.0.operation.to_lowercase();
        let dry_run = params.0.dry_run.unwrap_or(false);
        let force = params.0.force.unwrap_or(false);
        let audio_dir = params.0.audio_dir.map(PathBuf::from);
        let json_output = params.0.json_output.unwrap_or(false);

        log::info!(
            "cue_file called with path: {}, operation: {}, dry_run: {}, force: {}",
            path.display(),
            operation,
            dry_run,
            force
        );

        match operation.as_str() {
            "generate" => {
                handle_cue_generate(&path, params.0.output.map(PathBuf::from), dry_run, force).await
            }
            "parse" => handle_cue_parse(&path, json_output).await,
            "validate" => handle_cue_validate(&path, audio_dir, json_output).await,
            _ => Ok(CallToolResult::error(vec![Content::text(
                "Invalid operation. Must be 'generate', 'parse', or 'validate'".to_string(),
            )])),
        }
    }
}

async fn handle_cue_generate(
    path: &Path,
    output: Option<PathBuf>,
    dry_run: bool,
    force: bool,
) -> Result<CallToolResult, McpError> {
    match generate_cue_for_path(path, output) {
        Ok(result) => {
            if !dry_run && result.output_path.exists() && !force {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Cue file already exists at '{}'. Use force=true to overwrite.",
                    result.output_path.display()
                ))]));
            }

            if dry_run {
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Would write to: {}\n\n{}",
                    result.output_path.display(),
                    result.cue_content
                ))]))
            } else {
                match std::fs::write(&result.output_path, &result.cue_content) {
                    Ok(_) => Ok(CallToolResult::success(vec![Content::text(format!(
                        "Cue file written to: {}",
                        result.output_path.display()
                    ))])),
                    Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                        "Error writing cue file: {}",
                        e
                    ))])),
                }
            }
        }
        Err(CueGenerationError::NoMusicFiles) => Ok(CallToolResult::error(vec![Content::text(
            "No music files found in directory (checked only immediate files, not subdirectories)"
                .to_string(),
        )])),
        Err(CueGenerationError::NoReadableFiles) => Ok(CallToolResult::error(vec![Content::text(
            "No readable music files found in directory".to_string(),
        )])),
        Err(CueGenerationError::FileReadError(msg)) => {
            Ok(CallToolResult::error(vec![Content::text(msg)]))
        }
    }
}

async fn handle_cue_parse(path: &Path, json_output: bool) -> Result<CallToolResult, McpError> {
    match parse_cue_file(path) {
        Ok(cue_file) => {
            if json_output {
                let result = serde_json::to_string_pretty(&cue_file).map_err(|e| {
                    McpError::invalid_params(format!("JSON serialization error: {}", e), None)
                })?;
                Ok(CallToolResult::success(vec![Content::text(result)]))
            } else {
                let mut output = format!("Cue File: {}\n", path.display());
                if let Some(performer) = &cue_file.performer {
                    output.push_str(&format!("  Performer: {}\n", performer));
                }
                if let Some(title) = &cue_file.title {
                    output.push_str(&format!("  Title: {}\n", title));
                }
                if !cue_file.files.is_empty() {
                    output.push_str("  Files:\n");
                    for file in &cue_file.files {
                        output.push_str(&format!("    - {}\n", file));
                    }
                }
                output.push_str(&format!("  Tracks: {}\n", cue_file.tracks.len()));
                for track in &cue_file.tracks {
                    let file_info = track
                        .file
                        .as_ref()
                        .map(|f| format!(" [{}]", f))
                        .unwrap_or_default();
                    output.push_str(&format!(
                        "    Track {:02}: {}{}\n",
                        track.number,
                        track.title.as_deref().unwrap_or("(no title)"),
                        file_info
                    ));
                }
                Ok(CallToolResult::success(vec![Content::text(output)]))
            }
        }
        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
            "Error parsing cue file: {}",
            e
        ))])),
    }
}

async fn handle_cue_validate(
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
            let mut result = CueValidationResult::default();
            result.is_valid = false;
            result.file_missing = true;
            let result_str = if json_output {
                serde_json::to_string_pretty(&result).map_err(|e| {
                    McpError::invalid_params(format!("JSON serialization error: {}", e), None)
                })?
            } else {
                format_cue_validation_result(&result)
            };
            return Ok(CallToolResult::success(vec![Content::text(result_str)]));
        }
    };

    let audio_files_refs: Vec<&Path> = audio_files.iter().map(|p| p.as_path()).collect();
    let result = validate_cue_consistency(path, &audio_files_refs);

    if json_output {
        let result_str = serde_json::to_string_pretty(&result).map_err(|e| {
            McpError::invalid_params(format!("JSON serialization error: {}", e), None)
        })?;
        Ok(CallToolResult::success(vec![Content::text(result_str)]))
    } else {
        let output = format_cue_validation_result(&result);
        Ok(CallToolResult::success(vec![Content::text(output)]))
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
        }
    }
}
