use serde::Deserialize;

/// Generic parameter for prompts that only need a library path.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LibraryPathParams {
    /// Path to the root of the music library directory.
    pub path: Option<String>,
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
