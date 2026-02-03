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