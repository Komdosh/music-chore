use crate::mcp::config::Config;
use crate::mcp::params::{
    CueParams, EmitLibraryMetadataParams, FindDuplicatesParams, GetLibraryTreeParams,
    NormalizeParams, ReadFileMetadataParams, ScanDirectoryParams, ValidateLibraryParams,
};

use crate::adapters::audio_formats::read_metadata;
use crate::build_library_hierarchy;
use crate::core::services::duplicates::find_duplicates;
use crate::core::services::format_tree::emit_by_path;
use crate::core::services::normalization::normalize_and_format;
use crate::core::services::scanner::{
    format_track_name_for_scan_output, scan_dir, scan_dir_with_options,
};
use crate::mcp::call_tool_result::CallToolResultExt;
use crate::mcp::cue_helper_methods::{handle_cue_generate, handle_cue_parse, handle_cue_validate};
use crate::mcp::music_chore_server::MusicChoreServer;
use crate::mcp::prompt_handler_requests::{
    AlbumMarathonParams, ArtistDiveParams, FormatAuditParams, InstrumentParams, LibraryPathParams,
    MoodPlaylistParams, SetlistParams, YearReviewParams,
};
use crate::mcp::prompts::{
    album_marathon_prompt, artist_deep_dive_prompt, collection_story_prompt,
    concert_setlist_prompt, cue_sheet_assistant_prompt, decade_analysis_prompt,
    duplicate_resolution_prompt, format_quality_audit_prompt, genre_breakdown_prompt,
    hidden_gems_prompt, instrument_to_learn_prompt, library_health_check_prompt,
    metadata_cleanup_guide_prompt, mood_playlist_prompt, reorganization_plan_prompt,
    similar_artists_discovery_prompt, top_tracks_analysis_prompt, year_in_review_prompt,
};
use crate::presentation::cli::commands::validate_path;
use rmcp::model::PromptMessageContent;
use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, GetPromptResult, PromptMessage, PromptMessageRole},
    prompt, prompt_router, tool, tool_router, ErrorData as McpError, ErrorData,
};
use std::path::PathBuf;
// ─── Helper traits & functions ───────────────────────────────────────────────


/// Serialize `value` to pretty JSON, mapping errors to `McpError`.
pub(crate) fn to_json_pretty<T: serde::Serialize>(value: &T) -> Result<String, McpError> {
    serde_json::to_string_pretty(value)
        .map_err(|e| McpError::invalid_params(format!("JSON serialization error: {e}"), None))
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
                format!("Access denied: path '{}' is not in allowed paths", path.display()),
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
    fn resolve_path_for_tool(
        &self,
        path_param: Option<String>,
    ) -> Result<PathBuf, CallToolResult> {
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
            match serde_json::to_string_pretty(&tracks) {
                Ok(s) => Ok(CallToolResult::success_text(s)),
                Err(e) => Ok(CallToolResult::error_text(format!(
                    "Error serializing to JSON: {e}"
                ))),
            }
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
        let path = match self.resolve_path_for_tool(params.0.path) {
            Ok(p) => p,
            Err(e) => return Ok(e),
        };

        let tracks = scan_dir(&path, false);
        let library = build_library_hierarchy(tracks);
        let result = to_json_pretty(&library)?;

        Ok(CallToolResult::success_text(result))
    }

    #[tool(description = "Read metadata from a single music file")]
    async fn read_file_metadata(
        &self,
        params: Parameters<ReadFileMetadataParams>,
    ) -> Result<CallToolResult, McpError> {
        let file = match self.resolve_path_for_tool(params.0.path) {
            Ok(p) => p,
            Err(e) => return Ok(e),
        };

        match read_metadata(&file) {
            Ok(track) => {
                let result = to_json_pretty(&track)?;
                Ok(CallToolResult::success_text(result))
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
        match find_duplicates(&path, json_output) {
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
                handle_cue_generate(&path, params.0.output.map(PathBuf::from), dry_run, force)
                    .await
            }
            "parse" => handle_cue_parse(&path, json_output).await,
            "validate" => handle_cue_validate(&path, audio_dir, json_output).await,
            _ => Ok(CallToolResult::error_text(
                "Invalid operation. Must be 'generate', 'parse', or 'validate'",
            )),
        }
    }

    // ─── Prompts: Analysis & Insights ────────────────────────────────────

    #[prompt(
        name = "top-tracks-analysis",
        description = "Analyze your music library and predict which tracks you probably love the most based on collection patterns, metadata richness, and genre clustering"
    )]
    async fn top_tracks_analysis(
        &self,
        params: Parameters<LibraryPathParams>,
    ) -> Result<GetPromptResult, McpError> {
        let (_buf, path) = self
            .resolve_path_str_for_prompt(params.0.path)
            .map_err(|_| McpError::invalid_params("Path resolution failed", None))?;

        Ok(user_prompt(top_tracks_analysis_prompt(path)))
    }

    #[prompt(
        name = "genre-breakdown",
        description = "Break down your music taste by genre distribution, discover your listening identity, and visualize how genres interconnect in your library"
    )]
    async fn genre_breakdown(
        &self,
        params: Parameters<LibraryPathParams>,
    ) -> Result<GetPromptResult, McpError> {
        let (_buf, path) = self
            .resolve_path_str_for_prompt(params.0.path)
            .map_err(|_| McpError::invalid_params("Path resolution failed", None))?;

        Ok(user_prompt(genre_breakdown_prompt(path)))
    }

    #[prompt(
        name = "decade-analysis",
        description = "Discover which decades dominate your music collection and what that reveals about your musical influences and nostalgia patterns"
    )]
    async fn decade_analysis(
        &self,
        params: Parameters<LibraryPathParams>,
    ) -> Result<GetPromptResult, McpError> {
        let (_buf, path) = self
            .resolve_path_str_for_prompt(params.0.path)
            .map_err(|_| McpError::invalid_params("Path resolution failed", None))?;

        Ok(user_prompt(decade_analysis_prompt(path)))
    }

    #[prompt(
        name = "collection-story",
        description = "Generate a narrative story about your music collection — its themes, diversity, emotional arc, and what it reveals about you as a listener"
    )]
    async fn collection_story(
        &self,
        params: Parameters<LibraryPathParams>,
    ) -> Result<GetPromptResult, McpError> {
        let (_buf, path) = self
            .resolve_path_str_for_prompt(params.0.path)
            .map_err(|_| McpError::invalid_params("Path resolution failed", None))?;

        Ok(user_prompt(collection_story_prompt(path)))
    }

    #[prompt(
        name = "artist-deep-dive",
        description = "Deep dive into a specific artist's presence in your library — their discography coverage, standout tracks, and how they fit into your broader taste"
    )]
    async fn artist_deep_dive(
        &self,
        params: Parameters<ArtistDiveParams>,
    ) -> Result<GetPromptResult, McpError> {
        let (_buf, path) = self
            .resolve_path_str_for_prompt(params.0.path)
            .map_err(|_| McpError::invalid_params("Path resolution failed", None))?;

        Ok(user_prompt(artist_deep_dive_prompt(path, params.0.artist_name)))
    }

    // ─── Prompts: Recommendations & Discovery ────────────────────────────

    #[prompt(
        name = "instrument-to-learn",
        description = "Based on your music library's genres and artists, recommend the best instrument to learn so you can play your favorite songs"
    )]
    async fn instrument_to_learn(
        &self,
        params: Parameters<InstrumentParams>,
    ) -> Result<GetPromptResult, McpError> {
        let (_buf, path) = self
            .resolve_path_str_for_prompt(params.0.path)
            .map_err(|_| McpError::invalid_params("Path resolution failed", None))?;

        let level = params.0.experience_level.as_deref().unwrap_or("beginner");

        Ok(user_prompt(instrument_to_learn_prompt(path, level)))
    }

    #[prompt(
        name = "similar-artists-discovery",
        description = "Discover new artists you'll love based on patterns in your existing music library"
    )]
    async fn similar_artists_discovery(
        &self,
        params: Parameters<LibraryPathParams>,
    ) -> Result<GetPromptResult, McpError> {
        let (_buf, path) = self
            .resolve_path_str_for_prompt(params.0.path)
            .map_err(|_| McpError::invalid_params("Path resolution failed", None))?;

        Ok(user_prompt(similar_artists_discovery_prompt(path)))
    }

    #[prompt(
        name = "mood-playlist",
        description = "Create a perfectly curated playlist from your library matched to a specific mood, activity, or moment"
    )]
    async fn mood_playlist(
        &self,
        params: Parameters<MoodPlaylistParams>,
    ) -> Result<GetPromptResult, McpError> {
        let (_buf, path) = self
            .resolve_path_str_for_prompt(params.0.path)
            .map_err(|_| McpError::invalid_params("Path resolution failed", None))?;

        let mood = &params.0.mood;
        let max = params.0.max_tracks.unwrap_or(20);

        Ok(user_prompt(mood_playlist_prompt(path, mood, max)))
    }

    #[prompt(
        name = "hidden-gems",
        description = "Uncover overlooked and underappreciated tracks in your library that deserve more attention"
    )]
    async fn hidden_gems(
        &self,
        params: Parameters<LibraryPathParams>,
    ) -> Result<GetPromptResult, McpError> {
        let (_buf, path) = self
            .resolve_path_str_for_prompt(params.0.path)
            .map_err(|_| McpError::invalid_params("Path resolution failed", None))?;

        Ok(user_prompt(hidden_gems_prompt(path)))
    }

    #[prompt(
        name = "album-marathon",
        description = "Design the perfect album listening marathon from your collection with themed sequencing and pacing"
    )]
    async fn album_marathon(
        &self,
        params: Parameters<AlbumMarathonParams>,
    ) -> Result<GetPromptResult, McpError> {
        let (_buf, path) = self
            .resolve_path_str_for_prompt(params.0.path)
            .map_err(|_| McpError::invalid_params("Path resolution failed", None))?;

        let hours = params.0.duration_hours.unwrap_or(4);

        Ok(user_prompt(album_marathon_prompt(path, hours, params.0.theme)))
    }

    #[prompt(
        name = "concert-setlist",
        description = "Build a dream concert setlist from your library — sequenced like a real live show with encores and crowd moments"
    )]
    async fn concert_setlist(
        &self,
        params: Parameters<SetlistParams>,
    ) -> Result<GetPromptResult, McpError> {
        let (_buf, path) = self
            .resolve_path_str_for_prompt(params.0.path)
            .map_err(|_| McpError::invalid_params("Path resolution failed", None))?;

        let minutes = params.0.duration_minutes.unwrap_or(90);

        Ok(user_prompt(concert_setlist_prompt(path, minutes, params.0.vibe)))
    }

    // ─── Prompts: Library Maintenance & Quality ──────────────────────────

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
        name = "reorganization-plan",
        description = "Get a strategic plan to reorganize your music library's folder structure for optimal browsing, consistency, and tool compatibility"
    )]
    async fn reorganization_plan(
        &self,
        params: Parameters<LibraryPathParams>,
    ) -> Result<GetPromptResult, McpError> {
        let (_buf, path) = self
            .resolve_path_str_for_prompt(params.0.path)
            .map_err(|_| McpError::invalid_params("Path resolution failed", None))?;

        Ok(user_prompt(reorganization_plan_prompt(path)))
    }

    #[prompt(
        name = "format-quality-audit",
        description = "Audit the audio formats and quality levels across your library, identify lossy files, and plan quality upgrades"
    )]
    async fn format_quality_audit(
        &self,
        params: Parameters<FormatAuditParams>,
    ) -> Result<GetPromptResult, McpError> {
        let (_buf, path) = self
            .resolve_path_str_for_prompt(params.0.path)
            .map_err(|_| McpError::invalid_params("Path resolution failed", None))?;

        let suggest_upgrades = params.0.suggest_upgrades.unwrap_or(true);

        Ok(user_prompt(format_quality_audit_prompt(path, suggest_upgrades)))
    }

    #[prompt(
        name = "year-in-review",
        description = "Generate a music collection year-in-review summary highlighting additions, patterns, and collection milestones"
    )]
    async fn year_in_review(
        &self,
        params: Parameters<YearReviewParams>,
    ) -> Result<GetPromptResult, McpError> {
        let (_buf, path) = self
            .resolve_path_str_for_prompt(params.0.path)
            .map_err(|_| McpError::invalid_params("Path resolution failed", None))?;

        let year = params.0.year.unwrap_or(2024);

        Ok(user_prompt(year_in_review_prompt(path, year)))
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
    use crate::mcp::prompt_handler_requests::LibraryPathParams;
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
        let json = to_json_pretty(&test).unwrap();
        assert!(json.contains("\"name\": \"test\""));
        assert!(json.contains("\"value\": 123"));
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

        assert!(server.validate_path(&PathBuf::from("/allowed/path")).is_ok());
        assert!(server.validate_path(&PathBuf::from("/not/allowed")).is_err());
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
        let path = server.resolve_and_validate_path(Some("/other/path".to_string())).unwrap();
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
        assert!(json.is_array());
        assert_eq!(json.as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_top_tracks_analysis_prompt() {
        let server = MusicChoreServer::new();
        let params = LibraryPathParams {
            path: Some("/allowed/path".to_string()),
        };
        
        let res = server.top_tracks_analysis(Parameters(params)).await.unwrap();
        assert_eq!(res.messages.len(), 1);
        let text = match &res.messages[0].content {
            PromptMessageContent::Text { text } => text,
            _ => panic!("Expected text"),
        };
        assert!(text.contains("/allowed/path"));
        assert!(text.contains("top 10 favorite tracks"));
    }
}
