#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ScanDirectoryParams {
    pub(crate) path: String,
    pub(crate) json_output: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetLibraryTreeParams {
    pub(crate) path: String,
    pub(crate) json_output: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ReadFileMetadataParams {
    pub(crate) file_path: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NormalizeTitlesParams {
    pub(crate) path: String,
    pub(crate) dry_run: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EmitLibraryMetadataParams {
    pub(crate) path: String,
    pub(crate) json_output: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ValidateLibraryParams {
    pub(crate) path: String,
    pub(crate) json_output: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FindDuplicatesParams {
    pub(crate) path: String,
    pub(crate) json_output: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GenerateCueParams {
    pub(crate) path: String,
    pub(crate) output: Option<String>,
    pub(crate) dry_run: Option<bool>,
    pub(crate) force: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ParseCueParams {
    pub(crate) path: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ValidateCueParams {
    pub(crate) path: String,
    pub(crate) audio_dir: Option<String>,
    pub(crate) json_output: Option<bool>,
}
