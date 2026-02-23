use crate::mcp::config::Config;
use crate::mcp::params::{
    CueParams, EmitLibraryMetadataParams, FindDuplicatesParams, GetLibraryTreeParams,
    NormalizeParams, ReadFileMetadataParams, ScanDirectoryParams, ScanDirectoryResponse,
    ValidateLibraryParams,
};

use crate::adapters::audio_formats::read_metadata;
use crate::build_library_hierarchy;
use crate::core::services::duplicates::find_duplicates;
use crate::core::services::format_tree::{emit_by_path, format_library_output};
use crate::core::services::normalization::normalize_and_format;
use crate::core::services::scanner::{
    format_track_name_for_scan_output, scan_dir, scan_dir_with_options,
};
use crate::mcp::call_tool_result::CallToolResultExt;
use crate::mcp::cue_helper_methods::{handle_cue_generate, handle_cue_parse, handle_cue_validate};
use crate::mcp::music_chore_server::MusicChoreServer;
use crate::mcp::prompt_handler_requests::{LibraryPathParams, ListenNowParams, WebMatchParams};
use crate::mcp::prompts::{
    cue_sheet_assistant_prompt, duplicate_resolution_prompt, library_health_check_prompt,
    listen_now_prompt, metadata_cleanup_guide_prompt, web_perfect_match_prompt,
};
use crate::presentation::cli::commands::validate_path;
use rmcp::model::PromptMessageContent;
use rmcp::{
    ErrorData as McpError, ErrorData,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, GetPromptResult, PromptMessage, PromptMessageRole},
    prompt, prompt_router, tool, tool_router,
};
use serde_json::to_string_pretty;
use std::path::PathBuf;
// ─── Helper traits & functions ───────────────────────────────────────────────

/// Serialize `value` to pretty JSON, mapping errors to `McpError`.
pub(crate) fn to_json_call_response<T: serde::Serialize>(
    value: &T,
) -> Result<CallToolResult, McpError> {
    serde_json::to_value(value)
        .map(|v| CallToolResult::structured(v))
        .map_err(|e| McpError::internal_error(format!("{e}"), None))
}

/// Build a single-message `GetPromptResult` addressed to the user.
fn user_prompt(text: String) -> GetPromptResult {
    GetPromptResult {
        description: None,
        messages: vec![PromptMessage {
            role: PromptMessageRole::User,
            content: PromptMessageContent::Text { text },
        }],
    }
}

/// Build an error `GetPromptResult` addressed from the assistant.
fn error_prompt(e: ErrorData) -> Result<GetPromptResult, ErrorData> {
    Ok(GetPromptResult {
        description: None,
        messages: vec![PromptMessage {
            role: PromptMessageRole::Assistant,
            content: PromptMessageContent::Text {
                text: e.to_string(),
            },
        }],
    })
}

// ─── Core implementation ─────────────────────────────────────────────────────

#[tool_router]
#[prompt_router]
impl MusicChoreServer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_config(config: Config) -> Self {
        Self {
            tool_router: Self::tool_router(),
            prompt_router: Self::prompt_router(),
            config,
        }
    }

    /// Validate that a path is allowed by the security configuration.
    fn validate_path(&self, path: &PathBuf) -> Result<(), McpError> {
        if !self.config.is_path_allowed(path) {
            return Err(McpError::invalid_params(
                format!(
                    "Access denied: path '{}' is not in allowed paths",
                    path.display()
                ),
                None,
            ));
        }
        Ok(())
    }

    /// Resolves a path parameter (using default if empty) and validates it.
    fn resolve_and_validate_path(&self, path_param: Option<String>) -> Result<PathBuf, McpError> {
        let raw = path_param.unwrap_or_default();
        let path = if raw.is_empty() {
            self.config
                .require_default_library_path()
                .map(|p| p.to_path_buf())
                .map_err(|e| McpError::invalid_params(e, None))?
        } else {
            PathBuf::from(raw)
        };
        self.validate_path(&path)?;
        Ok(path)
    }

    /// Resolve a path and convert to a `&str`, returning a tool-level error on failure.
    /// Suitable for tool handlers that want `Ok(CallToolResult::error(...))` on bad paths.
    fn resolve_path_for_tool(&self, path_param: Option<String>) -> Result<PathBuf, CallToolResult> {
        self.resolve_and_validate_path(path_param)
            .map_err(|e| CallToolResult::error_text(e.to_string()))
    }

    /// Resolve a path for a prompt handler, returning both the `PathBuf` and its `&str` form.
    fn resolve_path_str_for_prompt(
        &self,
        path_param: Option<String>,
    ) -> Result<(PathBuf, String), GetPromptResult> {
        let path_buf = self.resolve_and_validate_path(path_param).map_err(|e| {
            // Unwrap the Ok — `error_prompt` never fails.
            error_prompt(e).unwrap()
        })?;
        let path_str = path_buf
            .to_str()
            .ok_or_else(|| {
                let e = McpError::invalid_params("Invalid path string".to_string(), None);
                error_prompt(e).unwrap()
            })?
            .to_owned();
        Ok((path_buf, path_str))
    }

    // ─── Tools ───────────────────────────────────────────────────────────

    #[tool(description = "Recursively scan a directory for music files")]
    async fn scan_directory(
        &self,
        params: Parameters<ScanDirectoryParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = match self.resolve_path_for_tool(params.0.path) {
            Ok(p) => p,
            Err(e) => return Ok(e),
        };

        let json_output = params.0.json_output.unwrap_or(false);
        let skip_metadata = params.0.skip_metadata.unwrap_or(false);

        let tracks = scan_dir_with_options(&path, None, false, Vec::new(), skip_metadata);

        if tracks.is_empty() {
            return Ok(CallToolResult::error_text(format!(
                "No music files found in directory: {}",
                path.display()
            )));
        }

        if json_output {
            to_json_call_response(&ScanDirectoryResponse { tracks })
        } else {
            let out: String = tracks
                .iter()
                .map(|track| {
                    format!(
                        "{} [{}]\n",
                        track.file_path.display(),
                        format_track_name_for_scan_output(track)
                    )
                })
                .collect();
            Ok(CallToolResult::success_text(out))
        }
    }

    #[tool(description = "Get hierarchical library tree view")]
    async fn get_library_tree(
        &self,
        params: Parameters<GetLibraryTreeParams>,
    ) -> Result<CallToolResult, McpError> {
        let json_output = params.0.json_output.unwrap_or(false);
        let path = match self.resolve_path_for_tool(params.0.path) {
            Ok(p) => p,
            Err(e) => return Ok(e),
        };

        let tracks = scan_dir(&path, false);
        let library = build_library_hierarchy(tracks);

        if json_output {
            to_json_call_response(&library)
        } else {
            Ok(CallToolResult::success_text(format_library_output(
                &library,
            )))
        }
    }

    #[tool(description = "Read metadata from a single music file")]
    async fn read_file_metadata(
        &self,
        params: Parameters<ReadFileMetadataParams>,
    ) -> Result<CallToolResult, McpError> {
        let json_output = params.0.json_output.unwrap_or(false);
        let file = match self.resolve_path_for_tool(params.0.path) {
            Ok(p) => p,
            Err(e) => return Ok(e),
        };

        match read_metadata(&file) {
            Ok(track) => {
                if json_output {
                    to_json_call_response(&track)
                } else {
                    Ok(CallToolResult::success_text(
                        to_string_pretty(&track)
                            .unwrap_or("Fail to read_metadata".parse().unwrap()),
                    ))
                }
            }
            Err(e) => Ok(CallToolResult::error_text(format!(
                "Error reading metadata: {e}"
            ))),
        }
    }

    #[tool(description = "Normalize track titles and genres")]
    async fn normalize(
        &self,
        params: Parameters<NormalizeParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = match self.resolve_path_for_tool(params.0.path) {
            Ok(p) => p,
            Err(e) => return Ok(e),
        };

        let json_output = params.0.json_output.unwrap_or(false);
        match normalize_and_format(path.into(), json_output) {
            Ok(output) => Ok(CallToolResult::success_text(output)),
            Err(e) => Ok(CallToolResult::error_text(e)),
        }
    }

    #[tool(description = "Emit library metadata in structured format")]
    async fn emit_library_metadata(
        &self,
        params: Parameters<EmitLibraryMetadataParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = match self.resolve_path_for_tool(params.0.path) {
            Ok(p) => p,
            Err(e) => return Ok(e),
        };

        let json_output = params.0.json_output.unwrap_or(false);
        match emit_by_path(&path, json_output) {
            Ok(result) => Ok(CallToolResult::success_text(result)),
            Err(e) => Ok(CallToolResult::error_text(e)),
        }
    }

    #[tool(description = "Validate music library for common issues and inconsistencies")]
    async fn validate_library(
        &self,
        params: Parameters<ValidateLibraryParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = match self.resolve_path_for_tool(params.0.path) {
            Ok(p) => p,
            Err(e) => return Ok(e),
        };

        let json_output = params.0.json_output.unwrap_or(false);
        match validate_path(&path, json_output) {
            Ok(result) => Ok(CallToolResult::success_text(result)),
            Err(result) => Ok(CallToolResult::error_text(result)),
        }
    }

    #[tool(description = "Find duplicate tracks by checksum")]
    async fn find_duplicates(
        &self,
        params: Parameters<FindDuplicatesParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = match self.resolve_path_for_tool(params.0.path) {
            Ok(p) => p,
            Err(e) => return Ok(e),
        };

        let json_output = params.0.json_output.unwrap_or(false);
        let verbose = params.0.verbose.unwrap_or(false);
        let parallel = params.0.parallel;
        match find_duplicates(&path, json_output, verbose, parallel) {
            Ok(result) => Ok(CallToolResult::success_text(result)),
            Err(result) => Ok(CallToolResult::error_text(result)),
        }
    }

    #[tool(description = "Generate, parse, or validate .cue files")]
    async fn cue_file(&self, params: Parameters<CueParams>) -> Result<CallToolResult, McpError> {
        let path = match self.resolve_path_for_tool(params.0.path) {
            Ok(p) => p,
            Err(e) => return Ok(e),
        };

        let operation = params.0.operation.to_lowercase();
        let dry_run = params.0.dry_run.unwrap_or(false);
        let force = params.0.force.unwrap_or(false);
        let audio_dir = params.0.audio_dir.map(PathBuf::from);
        let json_output = params.0.json_output.unwrap_or(false);

        // Validate audio directory path if provided
        if let Some(ref audio_path) = audio_dir {
            if let Err(e) = self.validate_path(audio_path) {
                return Ok(CallToolResult::error_text(e.to_string()));
            }
        }

        match operation.as_str() {
            "generate" => {
                handle_cue_generate(&path, params.0.output.map(PathBuf::from), dry_run, force).await
            }
            "parse" => handle_cue_parse(&path, json_output).await,
            "validate" => handle_cue_validate(&path, audio_dir, json_output).await,
            _ => Ok(CallToolResult::error_text(
                "Invalid operation. Must be 'generate', 'parse', or 'validate'",
            )),
        }
    }

    // ─── Prompts: Core Listening + Essential Maintenance ─────────────────

    #[prompt(
        name = "listen-now",
        description = "Resolve listening indecision right now with one clear pick plus a short fallback queue based on your time, mood, and novelty preference"
    )]
    async fn listen_now(
        &self,
        params: Parameters<ListenNowParams>,
    ) -> Result<GetPromptResult, McpError> {
        let (_buf, path) = self
            .resolve_path_str_for_prompt(params.0.path)
            .map_err(|_| McpError::invalid_params("Path resolution failed", None))?;

        let minutes = params.0.available_minutes.unwrap_or(45);
        let mood = params.0.mood.unwrap_or_else(|| "any".to_string());
        let novelty = params
            .0
            .novelty_preference
            .unwrap_or_else(|| "balanced".to_string());

        Ok(user_prompt(listen_now_prompt(
            path, minutes, &mood, &novelty,
        )))
    }

    #[prompt(
        name = "web-perfect-match",
        description = "Find highest-fit web music recommendations based on your local library fingerprint with explicit scoring"
    )]
    async fn web_perfect_match(
        &self,
        params: Parameters<WebMatchParams>,
    ) -> Result<GetPromptResult, McpError> {
        let (_buf, path) = self
            .resolve_path_str_for_prompt(params.0.path)
            .map_err(|_| McpError::invalid_params("Path resolution failed", None))?;

        let max_results = params.0.max_results.unwrap_or(10);
        Ok(user_prompt(web_perfect_match_prompt(
            path,
            params.0.mood.as_deref(),
            params.0.genre.as_deref(),
            max_results,
        )))
    }

    #[prompt(
        name = "library-health-check",
        description = "Comprehensive health assessment of your music library covering metadata quality, organization issues, duplicates, and actionable cleanup steps"
    )]
    async fn library_health_check(
        &self,
        params: Parameters<LibraryPathParams>,
    ) -> Result<GetPromptResult, McpError> {
        let (_buf, path) = self
            .resolve_path_str_for_prompt(params.0.path)
            .map_err(|_| McpError::invalid_params("Path resolution failed", None))?;

        Ok(user_prompt(library_health_check_prompt(path)))
    }

    #[prompt(
        name = "metadata-cleanup-guide",
        description = "Identify metadata issues in your library and get a step-by-step guide to fix them using musicctl normalization tools"
    )]
    async fn metadata_cleanup_guide(
        &self,
        params: Parameters<LibraryPathParams>,
    ) -> Result<GetPromptResult, McpError> {
        let (_buf, path) = self
            .resolve_path_str_for_prompt(params.0.path)
            .map_err(|_| McpError::invalid_params("Path resolution failed", None))?;

        Ok(user_prompt(metadata_cleanup_guide_prompt(path)))
    }

    #[prompt(
        name = "duplicate-resolution",
        description = "Find duplicate tracks in your library and get intelligent recommendations for which copies to keep based on quality and metadata"
    )]
    async fn duplicate_resolution(
        &self,
        params: Parameters<LibraryPathParams>,
    ) -> Result<GetPromptResult, McpError> {
        let (_buf, path) = self
            .resolve_path_str_for_prompt(params.0.path)
            .map_err(|_| McpError::invalid_params("Path resolution failed", None))?;

        Ok(user_prompt(duplicate_resolution_prompt(path)))
    }

    #[prompt(
        name = "cue-sheet-assistant",
        description = "Analyze, generate, or troubleshoot CUE sheets in your library with expert guidance on proper formatting and validation"
    )]
    async fn cue_sheet_assistant(
        &self,
        params: Parameters<LibraryPathParams>,
    ) -> Result<GetPromptResult, McpError> {
        let (_buf, path) = self
            .resolve_path_str_for_prompt(params.0.path)
            .map_err(|_| McpError::invalid_params("Path resolution failed", None))?;

        Ok(user_prompt(cue_sheet_assistant_prompt(path)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::params::ScanDirectoryParams;
    use crate::mcp::prompt_handler_requests::{ListenNowParams, WebMatchParams};
    use rmcp::handler::server::wrapper::Parameters;
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestStruct {
        name: String,
        value: i32,
    }

    #[test]
    fn test_to_json_pretty() {
        let test = TestStruct {
            name: "test".to_string(),
            value: 123,
        };
        let answer = to_json_call_response(&test).unwrap();

        let json = &answer.content.get(0).unwrap().as_text().unwrap().text;
        assert_eq!(
            "{\"name\":\"test\",\"value\":123}", json,
            "Expected \"name\":\"test\",\"value\":123\", but was: {}",
            json
        );
    }

    #[test]
    fn test_user_prompt() {
        let prompt = user_prompt("hello".to_string());
        assert!(prompt.description.is_none());
        assert_eq!(prompt.messages.len(), 1);
        assert_eq!(prompt.messages[0].role, PromptMessageRole::User);
        match &prompt.messages[0].content {
            PromptMessageContent::Text { text } => assert_eq!(text, "hello"),
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_error_prompt() {
        let err = McpError::invalid_params("error msg", None);
        let prompt = error_prompt(err).unwrap();
        assert_eq!(prompt.messages.len(), 1);
        assert_eq!(prompt.messages[0].role, PromptMessageRole::Assistant);
        match &prompt.messages[0].content {
            PromptMessageContent::Text { text } => assert!(text.contains("error msg")),
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_validate_path() {
        let mut config = Config::default();
        config.allowed_paths = vec![PathBuf::from("/allowed")];
        let server = MusicChoreServer::new_with_config(config);

        assert!(
            server
                .validate_path(&PathBuf::from("/allowed/path"))
                .is_ok()
        );
        assert!(
            server
                .validate_path(&PathBuf::from("/not/allowed"))
                .is_err()
        );
    }

    #[test]
    fn test_resolve_and_validate_path() {
        let mut config = Config::default();
        config.default_library_path = Some(PathBuf::from("/default"));
        config.allowed_paths = vec![PathBuf::from("/default"), PathBuf::from("/other")];
        let server = MusicChoreServer::new_with_config(config);

        // Test default path
        let path = server.resolve_and_validate_path(None).unwrap();
        assert_eq!(path, PathBuf::from("/default"));

        // Test explicit path
        let path = server
            .resolve_and_validate_path(Some("/other/path".to_string()))
            .unwrap();
        assert_eq!(path, PathBuf::from("/other/path"));

        // Test denied path
        let res = server.resolve_and_validate_path(Some("/denied".to_string()));
        assert!(res.is_err());
    }

    #[test]
    fn test_resolve_path_for_tool() {
        let mut config = Config::default();
        config.allowed_paths = vec![PathBuf::from("/allowed")];
        let server = MusicChoreServer::new_with_config(config);

        let res = server.resolve_path_for_tool(Some("/allowed/path".to_string()));
        assert!(res.is_ok());

        let res = server.resolve_path_for_tool(Some("/denied".to_string()));
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.is_error.unwrap());
    }

    #[test]
    fn test_resolve_path_str_for_prompt() {
        let mut config = Config::default();
        config.allowed_paths = vec![PathBuf::from("/allowed")];
        let server = MusicChoreServer::new_with_config(config);

        let res = server.resolve_path_str_for_prompt(Some("/allowed/path".to_string()));
        assert!(res.is_ok());
        let (buf, s) = res.unwrap();
        assert_eq!(buf, PathBuf::from("/allowed/path"));
        assert_eq!(s, "/allowed/path");

        let res = server.resolve_path_str_for_prompt(Some("/denied".to_string()));
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_scan_directory_tool_no_files() {
        let server = MusicChoreServer::new();
        // Use a definitely empty directory or a nonexistent one
        let params = ScanDirectoryParams {
            path: Some("/tmp/nonexistent_music_dir_xyz".to_string()),
            json_output: None,
            skip_metadata: None,
        };

        let res = server.scan_directory(Parameters(params)).await.unwrap();
        assert!(res.is_error.unwrap());
        let text = res.content[0].raw.as_text().unwrap().text.as_str();
        assert!(text.contains("No music files found"));
    }

    #[tokio::test]
    async fn test_scan_directory_tool_success() {
        let server = MusicChoreServer::new();
        let params = ScanDirectoryParams {
            path: Some("tests/fixtures/flac/simple".to_string()),
            json_output: Some(true),
            skip_metadata: None,
        };

        let res = server.scan_directory(Parameters(params)).await.unwrap();
        assert!(!res.is_error.unwrap_or(false));
        let text = res.content[0].raw.as_text().unwrap().text.as_str();
        let json: serde_json::Value = serde_json::from_str(text).unwrap();
        assert!(json.is_object());
        assert_eq!(
            json["tracks"]
                .as_array()
                .expect("tracks should be array")
                .len(),
            2
        );
    }

    #[tokio::test]
    async fn test_listen_now_prompt() {
        let server = MusicChoreServer::new();
        let params = ListenNowParams {
            path: Some("/allowed/path".to_string()),
            available_minutes: Some(30),
            mood: Some("focus".to_string()),
            novelty_preference: Some("balanced".to_string()),
        };

        let res = server.listen_now(Parameters(params)).await.unwrap();
        assert_eq!(res.messages.len(), 1);
        let text = match &res.messages[0].content {
            PromptMessageContent::Text { text } => text,
            _ => panic!("Expected text"),
        };
        assert!(text.contains("/allowed/path"));
        assert!(text.contains("Available time: 30 minutes"));
        assert!(text.contains("Mood/activity: \"focus\""));
    }

    #[tokio::test]
    async fn test_web_perfect_match_prompt() {
        let server = MusicChoreServer::new();
        let params = WebMatchParams {
            path: Some("/allowed/path".to_string()),
            mood: Some("night drive".to_string()),
            genre: Some("ambient".to_string()),
            max_results: Some(7),
        };

        let res = server.web_perfect_match(Parameters(params)).await.unwrap();
        assert_eq!(res.messages.len(), 1);
        let text = match &res.messages[0].content {
            PromptMessageContent::Text { text } => text,
            _ => panic!("Expected text"),
        };
        assert!(text.contains("/allowed/path"));
        assert!(text.contains("Mood filter: \"night drive\""));
        assert!(text.contains("Genre filter: \"ambient\""));
        assert!(text.contains("Max results: 7"));
    }
}
