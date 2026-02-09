#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ScanDirectoryParams {
    pub(crate) path: Option<String>,
    pub(crate) json_output: Option<bool>,
    pub(crate) skip_metadata: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetLibraryTreeParams {
    pub(crate) path: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ReadFileMetadataParams {
    pub(crate) path: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NormalizeParams {
    pub(crate) path: Option<String>,
    pub(crate) json_output: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EmitLibraryMetadataParams {
    pub(crate) path: Option<String>,
    pub(crate) json_output: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ValidateLibraryParams {
    pub(crate) path: Option<String>,
    pub(crate) json_output: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FindDuplicatesParams {
    pub(crate) path: Option<String>,
    pub(crate) json_output: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CueParams {
    pub(crate) path: Option<String>,
    pub(crate) operation: String,
    pub(crate) output: Option<String>,
    pub(crate) dry_run: Option<bool>,
    pub(crate) force: Option<bool>,
    pub(crate) audio_dir: Option<String>,
    pub(crate) json_output: Option<bool>,
}
