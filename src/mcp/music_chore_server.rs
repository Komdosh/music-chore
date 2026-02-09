use crate::mcp::params::{
    CueParams, EmitLibraryMetadataParams, FindDuplicatesParams, GetLibraryTreeParams,
    NormalizeParams, ReadFileMetadataParams, ScanDirectoryParams, ValidateLibraryParams,
};
use crate::mcp::config::Config;

use crate::adapters::audio_formats::read_metadata;
use crate::build_library_hierarchy;
use crate::core::services::cue::{
    format_cue_validation_result, generate_cue_for_path, parse_cue_file, validate_cue_consistency,
    CueGenerationError, CueValidationResult,
};
use crate::core::services::duplicates::find_duplicates;
use crate::core::services::format_tree::emit_by_path;
use crate::core::services::normalization::normalize_and_format;
use crate::core::services::scanner::{
    format_track_name_for_scan_output, scan_dir, scan_dir_with_options,
};
use crate::mcp::prompt_handler_requests::{
    AlbumMarathonParams, ArtistDiveParams, FormatAuditParams, InstrumentParams, LibraryPathParams,
    MoodPlaylistParams, SetlistParams, YearReviewParams,
};
use crate::presentation::cli::commands::validate_path;
use rmcp::handler::server::router::prompt::PromptRouter;
use rmcp::model::PromptMessageContent;
use rmcp::{RoleServer, handler::server::{tool::ToolRouter, wrapper::Parameters, ServerHandler}, model::{
    CallToolResult, Content, GetPromptRequestParams, GetPromptResult,
    Implementation, ListPromptsResult, PaginatedRequestParams, PromptMessage,
    PromptMessageRole, ProtocolVersion, ServerCapabilities, ServerInfo,
}, prompt, prompt_handler, prompt_router, service::RequestContext, tool, tool_handler, tool_router, ErrorData as McpError, ErrorData};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct MusicChoreServer {
    tool_router: ToolRouter<Self>,
    prompt_router: PromptRouter<Self>,
    config: Config,
}

impl Default for MusicChoreServer {
    fn default() -> Self {
        Self::new_with_config(Config::default())
    }
}

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

    /// Validate that a path is allowed by the security configuration
    fn validate_path(&self, path: &PathBuf) -> Result<(), McpError> {
        if !self.config.is_path_allowed(path) {
            return Err(McpError::invalid_params(format!(
                "Access denied: path '{}' is not in allowed paths",
                path.display()
            ), None));
        }
        Ok(())
    }

    /// Resolves a path parameter (using default if empty) and validates it
    fn resolve_and_validate_path(&self, path_param: Option<String>) -> Result<PathBuf, McpError> {
        let path_option = path_param.unwrap_or("".to_string());
        let path = if path_option.is_empty() {
            self.config
                .require_default_library_path()
                .map(|p| p.to_path_buf())
                .map_err(|e| McpError::invalid_params(e, None))?
        } else {
            PathBuf::from(path_option)
        };

        self.validate_path(&path)?;
        Ok(path)
    }

    #[tool(description = "Recursively scan a directory for music files")]
    async fn scan_directory(
        &self,
        params: Parameters<ScanDirectoryParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = match self.resolve_and_validate_path(params.0.path) {
            Ok(p) => p,
            Err(e) => return Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        };
        
        let json_output = params.0.json_output.unwrap_or(false);
        let skip_metadata = params.0.skip_metadata.unwrap_or(false);

        let tracks = scan_dir_with_options(
            &path,
            None,       // max_depth
            false,      // follow_symlinks
            Vec::new(), // exclude_patterns
            skip_metadata,
        );

        if tracks.is_empty() {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "No music files found in directory: {}",
                path.display()
            ))]));
        }

        if json_output {
            match serde_json::to_string_pretty(&tracks) {
                Ok(s) => Ok(CallToolResult::success(vec![Content::text(s)])),
                Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                    "Error serializing to JSON: {}",
                    e
                ))])),
            }
        } else {
            let mut out = String::new();
            for track in tracks {
                let track_name_for_display = format_track_name_for_scan_output(&track);
                out.push_str(&format!(
                    "{} [{}]\n",
                    track.file_path.display(),
                    track_name_for_display
                ));
            }
            Ok(CallToolResult::success(vec![Content::text(out)]))
        }
    }
    #[tool(description = "Get hierarchical library tree view")]
    async fn get_library_tree(
        &self,
        params: Parameters<GetLibraryTreeParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = match self.resolve_and_validate_path(params.0.path) {
            Ok(p) => p,
            Err(e) => return Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        };
        
        let _json_output = params.0.json_output.unwrap_or(false);
        let tracks = scan_dir(&path, false);
        let library = build_library_hierarchy(tracks);

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
        let file = match self.resolve_and_validate_path(params.0.path) {
            Ok(p) => p,
            Err(e) => return Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        };

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

    #[tool(description = "Normalize track titles and genres")]
    async fn normalize(
        &self,
        params: Parameters<NormalizeParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = match self.resolve_and_validate_path(params.0.path) {
            Ok(p) => p,
            Err(e) => return Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        };
        
        let json_output = params.0.json_output.unwrap_or(false);

        match normalize_and_format(path.into(), json_output) {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(output)])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(description = "Emit library metadata in structured format")]
    async fn emit_library_metadata(
        &self,
        params: Parameters<EmitLibraryMetadataParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = match self.resolve_and_validate_path(params.0.path) {
            Ok(p) => p,
            Err(e) => return Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        };
        
        let json_output = params.0.json_output.unwrap_or(false);

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
        let path = match self.resolve_and_validate_path(params.0.path) {
            Ok(p) => p,
            Err(e) => return Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        };
        
        let json_output = params.0.json_output.unwrap_or(false);

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
        let path = match self.resolve_and_validate_path(params.0.path) {
            Ok(p) => p,
            Err(e) => return Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        };
        
        let json_output = params.0.json_output.unwrap_or(false);

        return match find_duplicates(&path, json_output) {
            Ok(result) => Ok(CallToolResult::success(vec![Content::text(result)])),
            Err(result) => Ok(CallToolResult::error(vec![Content::text(result)])),
        };
    }

    #[tool(description = "Generate, parse, or validate .cue files")]
    async fn cue_file(&self, params: Parameters<CueParams>) -> Result<CallToolResult, McpError> {
        let path = match self.resolve_and_validate_path(params.0.path) {
            Ok(p) => p,
            Err(e) => return Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        };
        
        let operation = params.0.operation.to_lowercase();
        let dry_run = params.0.dry_run.unwrap_or(false);
        let force = params.0.force.unwrap_or(false);
        let audio_dir = params.0.audio_dir.map(PathBuf::from);
        
        // Validate audio directory path if provided
        if let Some(ref audio_path) = audio_dir {
            if let Err(e) = self.validate_path(audio_path) {
                return Ok(CallToolResult::error(vec![Content::text(e.to_string())]));
            }
        }
        
        let json_output = params.0.json_output.unwrap_or(false);

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

    /////
    // ── Analysis & Insights ──────────────────────────────────────────────

    #[prompt(
        name = "top-tracks-analysis",
        description = "Analyze your music library and predict which tracks you probably love the most based on collection patterns, metadata richness, and genre clustering"
    )]
    async fn top_tracks_analysis(
        &self,
        params: Parameters<LibraryPathParams>,
    ) -> Result<GetPromptResult, McpError> {
        let path_buf = match self.resolve_and_validate_path(params.0.path) {
            Ok(p) => p,
            Err(e) => {
                return Self::get_error_in_prompt(e);
            }
        };
        let path = path_buf
            .to_str()
            .ok_or_else(|| McpError::invalid_params("Invalid path string".to_string(), None))?;

        let messages = vec![PromptMessage {
            role: PromptMessageRole::User,
            content: PromptMessageContent::Text {
                text: format!(
                    r#"I want you to analyze my music library at "{path}" and predict my top 10 favorite tracks.

Here's how to approach this:

1. First, use the `scan_directory` tool with `json_output: true` to get all tracks.
2. Then use `get_library_tree` to understand the library hierarchy.
3. Analyze the results looking for these signals of a "loved" track:
   - **Artist frequency**: Artists with many albums/tracks likely indicate strong preference.
   - **Metadata completeness**: Well-tagged tracks suggest the owner cares about them.
   - **Genre patterns**: Identify the dominant genres — tracks in these genres are more likely favorites.
   - **Album completeness**: Complete albums (vs. scattered singles) suggest intentional collection.

Present your findings as:
- A ranked list of 10 tracks with reasoning for each pick.
- A brief "taste profile" summary explaining my overall music personality.
- 3 honorable mentions that almost made the list."#
                ),
            },
        }];

        Ok(GetPromptResult {
            description: None,
            messages,
        })
    }

    #[prompt(
        name = "genre-breakdown",
        description = "Break down your music taste by genre distribution, discover your listening identity, and visualize how genres interconnect in your library"
    )]
    async fn genre_breakdown(
        &self,
        params: Parameters<LibraryPathParams>,
    ) -> Result<GetPromptResult, McpError> {
        let path_buf = match self.resolve_and_validate_path(params.0.path) {
            Ok(p) => p,
            Err(e) => {
                return Self::get_error_in_prompt(e);
            }
        };
        let path = path_buf
            .to_str()
            .ok_or_else(|| McpError::invalid_params("Invalid path string".to_string(), None))?;

        let messages = vec![PromptMessage {
            role: PromptMessageRole::User,
            content: PromptMessageContent::Text {
                text: format!(
                    r#"Analyze the genre distribution across my music library at "{path}".

Steps:
1. Use `scan_directory` with `json_output: true` to get all tracks and their metadata.
2. Use `emit_library_metadata` for structured genre data.

Provide:
- **Genre Distribution**: A percentage breakdown of all genres in my library, sorted by prevalence.
- **Genre Clusters**: Group related genres (e.g., "Rock → Alternative Rock → Indie Rock") and show how they connect.
- **Listening Identity**: Give me a creative 2-3 word "listener archetype" name (e.g., "Melancholic Indie Explorer" or "Rhythmic Jazz Purist").
- **Genre Gaps**: Identify genres that are notably absent given my existing taste profile.
- **Genre Evolution Hints**: Based on my collection, suggest which genres I might naturally gravitate toward next.
- **Cross-Genre Bridges**: Identify tracks or artists in my library that bridge between my top genres."#
                ),
            },
        }];

        Ok(GetPromptResult {
            description: None,
            messages,
        })
    }

    #[prompt(
        name = "decade-analysis",
        description = "Discover which decades dominate your music collection and what that reveals about your musical influences and nostalgia patterns"
    )]
    async fn decade_analysis(
        &self,

        params: Parameters<LibraryPathParams>,
    ) -> Result<GetPromptResult, McpError> {
        let path_buf = match self.resolve_and_validate_path(params.0.path) {
            Ok(p) => p,
            Err(e) => {
                return Self::get_error_in_prompt(e);
            }
        };
        let path = path_buf
            .to_str()
            .ok_or_else(|| McpError::invalid_params("Invalid path string".to_string(), None))?;

        let messages = vec![PromptMessage {
            role: PromptMessageRole::User,
            content: PromptMessageContent::Text {
                text: format!(
                    r#"Perform a temporal analysis of my music library at "{path}".

Steps:
1. Use `scan_directory` with `json_output: true` to extract track metadata including year/date tags.
2. Use `get_library_tree` to see the full library structure.

Analyze and present:
- **Decade Distribution**: A breakdown showing how many tracks fall in each decade (1950s–2020s).
- **Peak Decade**: Which decade dominates and what that says about my influences.
- **Nostalgia Index**: Am I primarily a "retrospective listener" or "contemporary explorer"?
- **Decade-Genre Map**: Show how genres shift across decades in my collection.
- **Time Travel Playlist**: Pick one standout track per decade that best represents my taste in that era.
- **Missing Eras**: If any decades are underrepresented, suggest what I'm missing out on."#
                ),
            },
        }];

        Ok(GetPromptResult {
            description: None,
            messages,
        })
    }

    #[prompt(
        name = "collection-story",
        description = "Generate a narrative story about your music collection — its themes, diversity, emotional arc, and what it reveals about you as a listener"
    )]
    async fn collection_story(
        &self,

        params: Parameters<LibraryPathParams>,
    ) -> Result<GetPromptResult, McpError> {
        let path_buf = match self.resolve_and_validate_path(params.0.path) {
            Ok(p) => p,
            Err(e) => {
                return Self::get_error_in_prompt(e);
            }
        };
        let path = path_buf
            .to_str()
            .ok_or_else(|| McpError::invalid_params("Invalid path string".to_string(), None))?;

        let messages = vec![PromptMessage {
            role: PromptMessageRole::User,
            content: PromptMessageContent::Text {
                text: format!(
                    r#"Tell me the story of my music collection at "{path}".

Steps:
1. Use `scan_directory` with `json_output: true` to get all tracks.
2. Use `get_library_tree` for the hierarchical view.
3. Use `emit_library_metadata` for the full structured metadata.

Write a compelling narrative that covers:
- **The Opening Chapter**: How does the collection begin? What's the foundational artist/genre?
- **Themes & Motifs**: What recurring patterns or themes emerge (genres, moods, eras)?
- **Diversity Score**: Rate the collection's diversity on a 1-10 scale with explanation.
- **Emotional Landscape**: Map the emotional range — is this a collection of joy, melancholy, energy, contemplation?
- **The Collector's Portrait**: Paint a picture of who owns this library. What can you infer about their personality, life stage, and experiences?
- **The Missing Chapter**: What's conspicuously absent that would complete the story?

Write this in an engaging, literary style — as if reviewing a curated exhibition."#
                ),
            },
        }];

        Ok(GetPromptResult {
            description: None,
            messages,
        })
    }

    #[prompt(
        name = "artist-deep-dive",
        description = "Deep dive into a specific artist's presence in your library — their discography coverage, standout tracks, and how they fit into your broader taste"
    )]
    async fn artist_deep_dive(
        &self,

        params: Parameters<ArtistDiveParams>,
    ) -> Result<GetPromptResult, McpError> {
        let path_buf = match self.resolve_and_validate_path(params.0.path) {
            Ok(p) => p,
            Err(e) => {
                return Self::get_error_in_prompt(e);
            }
        };
        let path = path_buf
            .to_str()
            .ok_or_else(|| McpError::invalid_params("Invalid path string".to_string(), None))?;

        let artist_clause = match &params.0.artist_name {
            Some(name) => format!("Focus specifically on the artist \"{}\".", name),
            None => "Identify the most prominent artist in my library and focus on them.".into(),
        };
        let messages = vec![PromptMessage {
            role: PromptMessageRole::User,
            content: PromptMessageContent::Text {
                text: format!(
                    r#"Perform a deep dive into an artist from my music library at "{path}".

{artist_clause}

Steps:
1. Use `scan_directory` with `json_output: true` to get all tracks.
2. Use `get_library_tree` to understand the library hierarchy.

Provide:
- **Discography Coverage**: Which albums/tracks do I have? What am I missing?
- **Collection Completeness**: Rate as percentage with specific gaps identified.
- **Standout Tracks**: Which tracks by this artist are likely my favorites and why?
- **Genre/Style Evolution**: How does this artist's style evolve across the albums I own?
- **Library Context**: How does this artist connect to my other artists and genres?
- **Recommendations**: Suggest 3-5 missing albums/tracks I should add.
- **Similar Artists in Library**: Which other artists in my collection share DNA with this one?"#
                ),
            },
        }];

        Ok(GetPromptResult {
            description: None,
            messages,
        })
    }

    // ── Recommendations & Discovery ──────────────────────────────────────

    #[prompt(
        name = "instrument-to-learn",
        description = "Based on your music library's genres and artists, recommend the best instrument to learn so you can play your favorite songs"
    )]
    async fn instrument_to_learn(
        &self,

        params: Parameters<InstrumentParams>,
    ) -> Result<GetPromptResult, McpError> {
        let path_buf = match self.resolve_and_validate_path(params.0.path) {
            Ok(p) => p,
            Err(e) => {
                return Self::get_error_in_prompt(e);
            }
        };
        let path = path_buf
            .to_str()
            .ok_or_else(|| McpError::invalid_params("Invalid path string".to_string(), None))?;

        let level = params.0.experience_level.as_deref().unwrap_or("beginner");
        let messages = vec![PromptMessage {
            role: PromptMessageRole::User,
            content: PromptMessageContent::Text {
                text: format!(
                    r#"Based on my music library at "{path}", recommend what instrument I should learn.
My experience level: {level}.

Steps:
1. Use `scan_directory` with `json_output: true` to analyze my music.
2. Use `emit_library_metadata` for detailed metadata.

Provide:
- **Top 3 Instrument Recommendations**: Ranked by how well they match my music taste.
- For each instrument:
  - Why it matches my library's genres and artists.
  - 5 songs from my library I could learn (ordered from easiest to hardest).
  - Estimated time to play my first song at my experience level.
  - Recommended learning resources specific to my genres.
- **Genre-Instrument Map**: Show which instruments dominate each of my top genres.
- **The "Fun Factor" Pick**: Which instrument would let me jam along to the most tracks?
- **The "Surprise" Pick**: An unconventional instrument that would give my favorites a fresh twist."#
                ),
            },
        }];

        Ok(GetPromptResult {
            description: None,
            messages,
        })
    }

    #[prompt(
        name = "similar-artists-discovery",
        description = "Discover new artists you'll love based on patterns in your existing music library"
    )]
    async fn similar_artists_discovery(
        &self,

        params: Parameters<LibraryPathParams>,
    ) -> Result<GetPromptResult, McpError> {
        let path_buf = match self.resolve_and_validate_path(params.0.path) {
            Ok(p) => p,
            Err(e) => {
                return Self::get_error_in_prompt(e);
            }
        };
        let path = path_buf
            .to_str()
            .ok_or_else(|| McpError::invalid_params("Invalid path string".to_string(), None))?;

        let messages = vec![PromptMessage {
            role: PromptMessageRole::User,
            content: PromptMessageContent::Text {
                text: format!(
                    r#"Analyze my music library at "{path}" and suggest new artists I should discover.

Steps:
1. Use `scan_directory` with `json_output: true` to get all tracks.
2. Use `get_library_tree` for the full hierarchy.
3. Use `emit_library_metadata` for structured data.

Provide:
- **Taste DNA**: First summarize my core musical preferences (genres, eras, moods).
- **10 Artist Recommendations** organized into:
  - 🎯 **Safe Bets** (3 artists): Very similar to what I already love.
  - 🔀 **Lateral Moves** (4 artists): Same vibe but different genres/eras.
  - 🚀 **Stretch Picks** (3 artists): Outside my comfort zone but with hooks I'd appreciate.
- For each artist provide:
  - Why they match my taste.
  - Best album to start with.
  - One track to sample first.
  - Which artist in my library they're most similar to.
- **The Rabbit Hole**: One deep-cut artist that could become an obsession."#
                ),
            },
        }];

        Ok(GetPromptResult {
            description: None,
            messages,
        })
    }

    #[prompt(
        name = "mood-playlist",
        description = "Create a perfectly curated playlist from your library matched to a specific mood, activity, or moment"
    )]
    async fn mood_playlist(
        &self,

        params: Parameters<MoodPlaylistParams>,
    ) -> Result<GetPromptResult, McpError> {
        let path_buf = match self.resolve_and_validate_path(params.0.path) {
            Ok(p) => p,
            Err(e) => {
                return Self::get_error_in_prompt(e);
            }
        };
        let path = path_buf
            .to_str()
            .ok_or_else(|| McpError::invalid_params("Invalid path string".to_string(), None))?;

        let mood = &params.0.mood;
        let max = params.0.max_tracks.unwrap_or(20);
        let messages = vec![PromptMessage {
            role: PromptMessageRole::User,
            content: PromptMessageContent::Text {
                text: format!(r#"Create a playlist from my music library at "{path}" for this mood/activity: "{mood}".
Maximum tracks: {max}.

Steps:
1. Use `scan_directory` with `json_output: true` to get all tracks with metadata.
2. Use `emit_library_metadata` for detailed track information.

Build the playlist considering:
- **Track Selection**: Pick tracks whose genre, tempo (inferred from genre/style), and mood align with "{mood}".
- **Flow & Sequencing**: Order tracks for natural energy flow — consider openers, builders, peaks, and cool-downs.
- **Variety**: Mix artists and albums while maintaining mood coherence.

Present:
- **Playlist Name**: A creative, evocative name for this playlist.
- **Track List**: Numbered list with Artist – Title – Album for each track.
- **Playlist Arc**: Brief description of the emotional journey the playlist creates.
- **Transition Notes**: For key transitions, explain why one track flows into the next.
- **Mood Match Score**: Rate how well my library serves this mood (1-10) and identify any gaps."#
                ),
            },
        }];

        Ok(GetPromptResult {
            description: None,
            messages,
        })
    }

    #[prompt(
        name = "hidden-gems",
        description = "Uncover overlooked and underappreciated tracks in your library that deserve more attention"
    )]
    async fn hidden_gems(
        &self,

        params: Parameters<LibraryPathParams>,
    ) -> Result<GetPromptResult, McpError> {
        let path_buf = match self.resolve_and_validate_path(params.0.path) {
            Ok(p) => p,
            Err(e) => {
                return Self::get_error_in_prompt(e);
            }
        };
        let path = path_buf
            .to_str()
            .ok_or_else(|| McpError::invalid_params("Invalid path string".to_string(), None))?;

        let messages = vec![PromptMessage {
            role: PromptMessageRole::User,
            content: PromptMessageContent::Text {
                text: format!(
                    r#"Search my music library at "{path}" for hidden gems — tracks and albums I might be overlooking.

Steps:
1. Use `scan_directory` with `json_output: true` to scan the full library.
2. Use `get_library_tree` for structure analysis.
3. Use `emit_library_metadata` for metadata details.

Look for:
- **Deep Cuts**: Non-single tracks from well-known artists that are critically acclaimed.
- **Solo Artists**: Tracks by artists with very few entries (1-2 tracks) — they were added for a reason.
- **Genre Outliers**: Tracks in genres that are rare in my collection — they stand out as intentional picks.
- **Album Closers/Openers**: Often the most carefully crafted tracks on an album.

Present:
- **10 Hidden Gems** with:
  - Track name, artist, album.
  - Why this qualifies as a hidden gem.
  - What makes it special or noteworthy.
  - When to listen to it (mood/setting recommendation).
- **The One You Forgot**: One track that's probably been sitting in your library unplayed.
- **Rediscovery Playlist**: Arrange all gems in an optimal listening order."#
                ),
            },
        }];

        Ok(GetPromptResult {
            description: None,
            messages,
        })
    }

    #[prompt(
        name = "album-marathon",
        description = "Design the perfect album listening marathon from your collection with themed sequencing and pacing"
    )]
    async fn album_marathon(
        &self,
        params: Parameters<AlbumMarathonParams>,
    ) -> Result<GetPromptResult, McpError> {
        let path_buf = match self.resolve_and_validate_path(params.0.path) {
            Ok(p) => p,
            Err(e) => {
                return Self::get_error_in_prompt(e);
            }
        };
        let path = path_buf
            .to_str()
            .ok_or_else(|| McpError::invalid_params("Invalid path string".to_string(), None))?;

        let hours = params.0.duration_hours.unwrap_or(4);
        let theme_clause = match &params.0.theme {
            Some(theme) => format!("Theme/approach: \"{theme}\"."),
            None => "Choose the most interesting sequencing approach for my collection.".into(),
        };
        let messages = vec![PromptMessage {
            role: PromptMessageRole::User,
            content: PromptMessageContent::Text {
                text: format!(
                    r#"Design a {hours}-hour album listening marathon from my library at "{path}".
{theme_clause}

Steps:
1. Use `scan_directory` with `json_output: true` to get all tracks.
2. Use `get_library_tree` to see album completeness.
3. Use `emit_library_metadata` for full album details.

Create a marathon plan with:
- **Album Selection**: Choose complete albums (or near-complete) that fit within {hours} hours.
- **Sequencing Strategy**: Explain the logic behind the album order (chronological, genre flow, energy arc, etc.).
- **Schedule**: Approximate timestamps for when each album starts/ends.
- **Intermissions**: Suggest 1-2 break points with palate-cleansing single tracks.
- **Listening Notes**: For each album, provide a 1-2 sentence "what to listen for" guide.
- **The Finale**: Which album closes the marathon and why it's the perfect ending.
- **Snack Pairings**: Fun bonus — suggest a drink/snack that matches each album's vibe!"#
                ),
            },
        }];

        Ok(GetPromptResult {
            description: None,
            messages,
        })
    }

    #[prompt(
        name = "concert-setlist",
        description = "Build a dream concert setlist from your library — sequenced like a real live show with encores and crowd moments"
    )]
    async fn concert_setlist(
        &self,

        params: Parameters<SetlistParams>,
    ) -> Result<GetPromptResult, McpError> {
        let path_buf = match self.resolve_and_validate_path(params.0.path) {
            Ok(p) => p,
            Err(e) => {
                return Self::get_error_in_prompt(e);
            }
        };
        let path = path_buf
            .to_str()
            .ok_or_else(|| McpError::invalid_params("Invalid path string".to_string(), None))?;

        let minutes = params.0.duration_minutes.unwrap_or(90);
        let vibe_clause = match &params.0.vibe {
            Some(v) => format!("Vibe/genre focus: \"{v}\"."),
            None => "Pull from all genres in my library for maximum variety.".into(),
        };
        let messages = vec![PromptMessage {
            role: PromptMessageRole::User,
            content: PromptMessageContent::Text {
                text: format!(
                    r#"Build a dream concert setlist from my library at "{path}".
Target duration: {minutes} minutes.
{vibe_clause}

Steps:
1. Use `scan_directory` with `json_output: true` for the full track catalog.
2. Use `emit_library_metadata` for detailed metadata.

Structure the setlist like a real concert:
- **Opener** (1-2 tracks): High energy to grab attention.
- **Early Set** (3-4 tracks): Establish the vibe, crowd-pleasers.
- **Deep Cut Segment** (2-3 tracks): Reward the dedicated fans.
- **Acoustic/Chill Break** (1-2 tracks): Emotional breather.
- **Build-Up** (3-4 tracks): Rising energy toward the climax.
- **Main Set Closer** (1 track): The biggest anthem in my collection.
- **Encore** (2-3 tracks): One intimate track, then go out with a bang.

For each track include:
- Position, artist, title.
- Why it fits this slot.
- Crowd energy level (1-10).
- Imagined crowd reaction.

End with a "setlist poster" — formatted like a real concert poster with the show name, venue vibe, and lineup."#
                ),
            },
        }];

        Ok(GetPromptResult {
            description: None,
            messages,
        })
    }

    // ── Library Maintenance & Quality ────────────────────────────────────

    #[prompt(
        name = "library-health-check",
        description = "Comprehensive health assessment of your music library covering metadata quality, organization issues, duplicates, and actionable cleanup steps"
    )]
    async fn library_health_check(
        &self,

        params: Parameters<LibraryPathParams>,
    ) -> Result<GetPromptResult, McpError> {
        let path_buf = match self.resolve_and_validate_path(params.0.path) {
            Ok(p) => p,
            Err(e) => {
                return Self::get_error_in_prompt(e);
            }
        };
        let path = path_buf
            .to_str()
            .ok_or_else(|| McpError::invalid_params("Invalid path string".to_string(), None))?;

        let messages = vec![PromptMessage {
            role: PromptMessageRole::User,
            content: PromptMessageContent::Text {
                text: format!(
                    r#"Perform a comprehensive health check on my music library at "{path}".

Run these tools in sequence:
1. `scan_directory` with `json_output: true` — get full track inventory.
2. `validate_library` with `json_output: true` — identify metadata issues.
3. `find_duplicates` with `json_output: true` — detect duplicate files.
4. `get_library_tree` — check organizational structure.

Compile a **Library Health Report** with:

📊 **Overall Health Score**: X/100 with letter grade.

🏗️ **Structure Assessment**:
  - Folder hierarchy consistency.
  - Naming convention adherence.
  - Orphaned or misplaced files.

🏷️ **Metadata Quality**:
  - Percentage of tracks with complete metadata.
  - Most common missing fields.
  - Inconsistencies (e.g., same artist spelled differently).

🔁 **Duplicate Report**:
  - Number of duplicate groups found.
  - Total wasted space.
  - Recommended deletions.

⚠️ **Critical Issues** (fix immediately):
  - List top 5 most urgent problems.

🔧 **Improvement Plan** (prioritized):
  - Step-by-step actions ordered by impact.
  - Estimated effort for each step.
  - Which `musicctl` commands to run.

Present the report in a clean, professional format."#
                ),
            },
        }];

        Ok(GetPromptResult {
            description: None,
            messages,
        })
    }

    #[prompt(
        name = "metadata-cleanup-guide",
        description = "Identify metadata issues in your library and get a step-by-step guide to fix them using musicctl normalization tools"
    )]
    async fn metadata_cleanup_guide(
        &self,

        params: Parameters<LibraryPathParams>,
    ) -> Result<GetPromptResult, McpError> {
        let path_buf = match self.resolve_and_validate_path(params.0.path) {
            Ok(p) => p,
            Err(e) => {
                return Self::get_error_in_prompt(e);
            }
        };
        let path = path_buf
            .to_str()
            .ok_or_else(|| McpError::invalid_params("Invalid path string".to_string(), None))?;

        let messages = vec![PromptMessage {
            role: PromptMessageRole::User,
            content: PromptMessageContent::Text {
                text: format!(
                    r#"Guide me through cleaning up metadata in my music library at "{path}".

Steps:
1. Use `validate_library` with `json_output: true` to find all issues.
2. Use `normalize` with `json_output: true` to preview normalization changes.
3. Use `scan_directory` with `json_output: true` for the full picture.

Create a **Metadata Cleanup Guide**:

📋 **Issue Inventory**:
  - Categorize all found issues by type (missing title, bad genre, inconsistent artist name, etc.).
  - Count how many tracks are affected by each issue type.

🎯 **Quick Wins** (automated fixes):
  - Issues that `normalize` can fix automatically.
  - Show before/after previews for each normalization.
  - Provide the exact command to apply: `musicctl normalize "{path}" --apply`.

✏️ **Manual Fixes Required**:
  - Issues that need human judgment (e.g., ambiguous artist names).
  - For each, suggest the most likely correct value.

📐 **Consistency Check**:
  - Artist name variations (e.g., "The Beatles" vs "Beatles").
  - Genre standardization opportunities.
  - Title formatting inconsistencies.

🚀 **Action Plan**:
  - Ordered steps to resolve everything.
  - Start with automated fixes, then manual ones.
  - Checkpoint after each step to verify."#
                ),
            },
        }];

        Ok(GetPromptResult {
            description: None,
            messages,
        })
    }

    #[prompt(
        name = "duplicate-resolution",
        description = "Find duplicate tracks in your library and get intelligent recommendations for which copies to keep based on quality and metadata"
    )]
    async fn duplicate_resolution(
        &self,

        params: Parameters<LibraryPathParams>,
    ) -> Result<GetPromptResult, McpError> {
        let path_buf = match self.resolve_and_validate_path(params.0.path) {
            Ok(p) => p,
            Err(e) => {
                return Self::get_error_in_prompt(e);
            }
        };
        let path = path_buf
            .to_str()
            .ok_or_else(|| McpError::invalid_params("Invalid path string".to_string(), None))?;

        let messages = vec![PromptMessage {
            role: PromptMessageRole::User,
            content: PromptMessageContent::Text {
                text: format!(
                    r#"Help me resolve duplicate files in my music library at "{path}".

Steps:
1. Use `find_duplicates` with `json_output: true` to identify all duplicate groups.
2. Use `scan_directory` with `json_output: true` for full metadata on affected files.
3. For key duplicates, use `read_file_metadata` on individual files for detailed comparison.

Provide a **Duplicate Resolution Report**:

📊 **Summary**:
  - Total duplicate groups found.
  - Total redundant files.
  - Estimated space savings.

🔍 **For each duplicate group**:
  - List all copies with their paths.
  - Compare: format, bitrate/quality, metadata completeness, file size.
  - **Recommendation**: Which copy to KEEP and why (prefer: higher quality → better metadata → better file path).
  - Mark as: ✅ Keep, ❌ Remove.

⚡ **Batch Actions**:
  - Group deletions by confidence level (high/medium/low).
  - High confidence: exact checksums, clear quality winner.
  - Low confidence: different versions, remasters, etc.

⚠️ **Caution List**: Duplicates that might actually be intentional (e.g., album version vs. compilation, remastered vs. original).

📝 **Cleanup Script**: Provide a summary of recommended file removals."#
                ),
            },
        }];

        Ok(GetPromptResult {
            description: None,
            messages,
        })
    }

    #[prompt(
        name = "reorganization-plan",
        description = "Get a strategic plan to reorganize your music library's folder structure for optimal browsing, consistency, and tool compatibility"
    )]
    async fn reorganization_plan(
        &self,

        params: Parameters<LibraryPathParams>,
    ) -> Result<GetPromptResult, McpError> {
        let path_buf = match self.resolve_and_validate_path(params.0.path) {
            Ok(p) => p,
            Err(e) => {
                return Self::get_error_in_prompt(e);
            }
        };
        let path = path_buf
            .to_str()
            .ok_or_else(|| McpError::invalid_params("Invalid path string".to_string(), None))?;

        let messages = vec![PromptMessage {
            role: PromptMessageRole::User,
            content: PromptMessageContent::Text {
                text: format!(
                    r#"Analyze my music library at "{path}" and suggest an optimal reorganization strategy.

Steps:
1. Use `get_library_tree` to see the current folder structure.
2. Use `scan_directory` with `json_output: true` for all tracks.
3. Use `validate_library` with `json_output: true` for structural issues.

Create a **Reorganization Plan**:

📁 **Current Structure Analysis**:
  - Describe the existing folder hierarchy pattern.
  - Identify inconsistencies and deviations.
  - Rate current organization (1-10).

🏗️ **Recommended Structure**:
  - Propose an `Artist/Album/Track` hierarchy.
  - Naming convention: `Artist Name/[Year] Album Name/## - Track Title.ext`.
  - Handle edge cases: compilations, soundtracks, singles, multi-disc albums.

🔀 **Migration Plan**:
  - List specific files/folders that need to move.
  - Show before → after paths for each.
  - Group moves by priority (worst offenders first).

📏 **Naming Standards**:
  - Capitalization rules.
  - Character handling (special chars, unicode).
  - Track number formatting (01, 02 vs 1, 2).

🛡️ **Safety Steps**:
  - Backup recommendations before restructuring.
  - How to verify nothing was lost after reorganization.
  - Metadata preservation during moves."#
                ),
            },
        }];

        Ok(GetPromptResult {
            description: None,
            messages,
        })
    }

    #[prompt(
        name = "format-quality-audit",
        description = "Audit the audio formats and quality levels across your library, identify lossy files, and plan quality upgrades"
    )]
    async fn format_quality_audit(
        &self,
        params: Parameters<FormatAuditParams>,
    ) -> Result<GetPromptResult, McpError> {
        let path_buf = match self.resolve_and_validate_path(params.0.path) {
            Ok(p) => p,
            Err(e) => {
                return Self::get_error_in_prompt(e);
            }
        };
        let path = path_buf
            .to_str()
            .ok_or_else(|| McpError::invalid_params("Invalid path string".to_string(), None))?;

        let suggest = params.0.suggest_upgrades.unwrap_or(true);
        let upgrade_clause = if suggest {
            "Include specific upgrade recommendations for lossy files."
        } else {
            "Focus on the audit only, skip upgrade recommendations."
        };
        let messages = vec![PromptMessage {
            role: PromptMessageRole::User,
            content: PromptMessageContent::Text {
                text: format!(
                    r#"Audit the audio quality and formats in my music library at "{path}".
{upgrade_clause}

Steps:
1. Use `scan_directory` with `json_output: true` to get all tracks with format details.
2. Use `emit_library_metadata` for structured metadata including format info.
3. Use `get_library_tree` for the full library view.

Create a **Format & Quality Audit Report**:

📊 **Format Distribution**:
  - Breakdown: FLAC, MP3, WAV, DSF, WavPack, and any others.
  - Percentage and file count for each format.

🎧 **Quality Tiers**:
  - 🥇 Lossless Hi-Res (24-bit FLAC, DSF, etc.)
  - 🥈 Lossless Standard (16-bit FLAC, WavPack)
  - 🥉 High-Quality Lossy (MP3 320kbps, etc.)
  - ⚠️ Low-Quality Lossy (MP3 <192kbps)

💾 **Storage Analysis**:
  - Total library size estimate by format.
  - Space savings if lossy were used vs. space cost of going full lossless.

🔄 **Upgrade Recommendations** (if requested):
  - Priority list of albums/tracks to upgrade from lossy to lossless.
  - Prioritize: favorite artists first, then albums with mixed formats.

🏆 **Audiophile Score**: Rate my library's overall quality (1-10) with explanation."#
                ),
            },
        }];

        Ok(GetPromptResult {
            description: None,
            messages,
        })
    }

    #[prompt(
        name = "year-in-review",
        description = "Generate a music collection year-in-review summary highlighting additions, patterns, and collection milestones"
    )]
    async fn year_in_review(
        &self,

        params: Parameters<YearReviewParams>,
    ) -> Result<GetPromptResult, McpError> {
        let path_buf = match self.resolve_and_validate_path(params.0.path) {
            Ok(p) => p,
            Err(e) => {
                return Self::get_error_in_prompt(e);
            }
        };
        let path = path_buf
            .to_str()
            .ok_or_else(|| McpError::invalid_params("Invalid path string".to_string(), None))?;

        let year = params.0.year.unwrap_or(2024);
        let messages = vec![PromptMessage {
            role: PromptMessageRole::User,
            content: PromptMessageContent::Text {
                text: format!(
                    r#"Create a "{year} Year in Review" for my music library at "{path}".

Steps:
1. Use `scan_directory` with `json_output: true` to get all tracks.
2. Use `get_library_tree` for the library overview.
3. Use `emit_library_metadata` for detailed track data.

Generate a **{year} Music Collection Review**:

📈 **Collection Stats**:
  - Total tracks, albums, and artists in the library.
  - Tracks from {year} specifically.
  - Library growth indicators.

🏆 **Top of {year}**:
  - Top 5 artists by track count.
  - Top 5 albums (most complete).
  - Genre of the year.

🎵 **Highlights**:
  - Most interesting additions from {year}.
  - Genre diversification — any new genres added?
  - The oldest and newest recordings in the collection.

📊 **By the Numbers**:
  - Average tracks per album.
  - Most prolific artist.
  - Format breakdown for {year} additions.

🔮 **{} Preview**:
  - Based on trends, predict what genres/artists I might add next.
  - Suggest 5 albums from {} that would complement my collection.

Present this like a premium music streaming service's annual wrap-up!"#,
                    year + 1,
                    year + 1
                ),
            },
        }];

        Ok(GetPromptResult {
            description: None,
            messages,
        })
    }

    #[prompt(
        name = "cue-sheet-assistant",
        description = "Analyze, generate, or troubleshoot CUE sheets in your library with expert guidance on proper formatting and validation"
    )]
    async fn cue_sheet_assistant(
        &self,
        params: Parameters<LibraryPathParams>,
    ) -> Result<GetPromptResult, McpError> {
        let path_buf = match self.resolve_and_validate_path(params.0.path) {
            Ok(p) => p,
            Err(e) => {
                return Self::get_error_in_prompt(e);
            }
        };
        let path = path_buf
            .to_str()
            .ok_or_else(|| McpError::invalid_params("Invalid path string".to_string(), None))?;

        let messages = vec![PromptMessage {
            role: PromptMessageRole::User,
            content: PromptMessageContent::Text {
                text: format!(
                    r#"Help me manage CUE sheets in my music library at "{path}".

Steps:
1. Use `scan_directory` with `json_output: true` to find all music files.
2. For any existing .cue files found, use `cue_file` with operation "validate" to check them.
3. For directories with album tracks but no .cue file, use `cue_file` with operation "generate" and `dry_run: true` to preview.

Provide a **CUE Sheet Management Report**:

📋 **Existing CUE Files**:
  - List all found CUE files and their validation status.
  - For invalid CUE files: describe the specific issues.

🆕 **Missing CUE Files**:
  - Albums that would benefit from a CUE sheet.
  - Preview the generated CUE content.

🔧 **Fix Recommendations**:
  - For each issue, provide the fix.
  - Suggest exact commands to regenerate problematic CUE files.

📖 **CUE Sheet Best Practices** specific to my library's formats."#
                ),
            },
        }];

        Ok(GetPromptResult {
            description: None,
            messages,
        })
    }

    fn get_error_in_prompt(e: ErrorData) -> Result<GetPromptResult, ErrorData> {
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
#[prompt_handler]
impl ServerHandler for MusicChoreServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .build(),
            server_info: Implementation {
                name: "music-chore".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                ..Default::default()
            },
            instructions: Some("Music Chore CLI - Music library metadata management tool".into()),
        }
    }
}
