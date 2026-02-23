use crate::mcp::music_chore_server::MusicChoreServer;
use rmcp::model::{GetPromptResult, PromptMessageContent};
use rmcp::{
    ErrorData as McpError,
    handler::server::{router::prompt::PromptRouter, wrapper::Parameters},
    model::{Content, PromptMessage, PromptMessageRole},
    prompt, prompt_router,
};
use serde::Deserialize;

/// Generic parameter for prompts that only need a library path.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LibraryPathParams {
    /// Path to the root of the music library directory.
    pub path: Option<String>,
}

/// Parameters for the artist deep-dive prompt.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ArtistDiveParams {
    /// Path to the root of the music library directory.
    pub path: Option<String>,
    /// Optional artist name to focus on. If omitted, the prompt will ask the
    /// AI to pick the most prominent artist in the library.
    pub artist_name: Option<String>,
}

/// Parameters for mood/activity-based playlist generation.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MoodPlaylistParams {
    /// Path to the root of the music library directory.
    pub path: Option<String>,
    /// The target mood or activity (e.g. "chill evening", "workout",
    /// "road trip", "dinner party", "focus work", "rainy day").
    pub mood: String,
    /// Maximum number of tracks in the playlist.
    pub max_tracks: Option<u32>,
}

/// Parameters for instrument recommendation.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct InstrumentParams {
    /// Path to the root of the music library directory.
    pub path: Option<String>,
    /// Self-reported experience level: "beginner", "intermediate", "advanced".
    pub experience_level: Option<String>,
}

/// Parameters for the album marathon prompt.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AlbumMarathonParams {
    /// Path to the root of the music library directory.
    pub path: Option<String>,
    /// Desired marathon length in hours. Defaults to 4.
    pub duration_hours: Option<u32>,
    /// Optional theme or constraint (e.g. "chronological", "genre journey",
    /// "energy arc").
    pub theme: Option<String>,
}

/// Parameters for the format/quality audit prompt.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FormatAuditParams {
    /// Path to the root of the music library directory.
    pub path: Option<String>,
    /// Whether to include upgrade recommendations for lossy files.
    pub suggest_upgrades: Option<bool>,
}

/// Parameters for the year-in-review prompt.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct YearReviewParams {
    /// Path to the root of the music library directory.
    pub path: Option<String>,
    /// The year to review. Defaults to the current year.
    pub year: Option<u32>,
}

/// Parameters for the concert setlist prompt.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SetlistParams {
    /// Path to the root of the music library directory.
    pub path: Option<String>,
    /// Setlist duration in minutes. Defaults to 90.
    pub duration_minutes: Option<u32>,
    /// Optional genre or vibe filter.
    pub vibe: Option<String>,
}

/// Parameters for a "what should I listen to right now?" decision.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListenNowParams {
    /// Path to the root of the music library directory.
    pub path: Option<String>,
    /// Available listening time in minutes. Defaults to 45.
    pub available_minutes: Option<u32>,
    /// Optional mood/activity hint (e.g. "focus", "night walk", "workout").
    pub mood: Option<String>,
    /// Discovery preference: "familiar", "balanced", or "adventurous".
    pub novelty_preference: Option<String>,
}

/// Parameters for picking one album for the current session.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AlbumTonightParams {
    /// Path to the root of the music library directory.
    pub path: Option<String>,
    /// Available listening time in minutes. Defaults to 60.
    pub available_minutes: Option<u32>,
}

/// Parameters for building a rediscovery rotation.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RediscoveryParams {
    /// Path to the root of the music library directory.
    pub path: Option<String>,
    /// Maximum tracks to include in the rotation. Defaults to 12.
    pub max_tracks: Option<u32>,
}

/// Parameters for choosing between two listening directions.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DecisionDuelParams {
    /// Path to the root of the music library directory.
    pub path: Option<String>,
    /// First listening direction (e.g. "calm acoustic").
    pub option_a: String,
    /// Second listening direction (e.g. "high-energy electronic").
    pub option_b: String,
    /// Maximum tracks to suggest per option. Defaults to 5.
    pub max_tracks_per_option: Option<u32>,
}

/// Parameters for finding high-fit web recommendations from library taste.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WebMatchParams {
    /// Path to the root of the music library directory.
    pub path: Option<String>,
    /// Optional mood constraint (e.g. "late night focus", "sunny walk").
    pub mood: Option<String>,
    /// Optional genre constraint.
    pub genre: Option<String>,
    /// Maximum recommendation count. Defaults to 10.
    pub max_results: Option<u32>,
}

/// Parameters for web recommendations constrained by genre.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WebGenreScoutParams {
    /// Path to the root of the music library directory.
    pub path: Option<String>,
    /// Target genre to scout online.
    pub genre: String,
    /// Maximum recommendation count. Defaults to 10.
    pub max_results: Option<u32>,
}

/// Parameters for web recommendations constrained by mood/activity.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WebMoodScoutParams {
    /// Path to the root of the music library directory.
    pub path: Option<String>,
    /// Target mood or activity.
    pub mood: String,
    /// Maximum recommendation count. Defaults to 10.
    pub max_results: Option<u32>,
}
