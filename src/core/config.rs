//! Application-wide configuration constants and settings.

/// Maximum allowed track number value
pub const MAX_TRACK_NUMBER: u32 = 999;

/// Maximum allowed disc number value
pub const MAX_DISC_NUMBER: u32 = 99;

/// Minimum valid year for music releases
pub const MIN_YEAR: u32 = 1000;

/// Maximum valid year for music releases
pub const MAX_YEAR: u32 = 3000;

/// Maximum duration in seconds (10 hours)
pub const MAX_DURATION_SECONDS: f64 = 36000.0;

/// Default confidence level for folder-inferred metadata
pub const FOLDER_INFERRED_CONFIDENCE: f32 = 0.3;

/// Default recursion depth for directory scanning
pub const DEFAULT_RECURSION_DEPTH: usize = 10;

/// Buffer size for file operations (8KB)
pub const FILE_BUFFER_SIZE: usize = 8192;

/// Maximum file size in MB that we'll process
pub const MAX_FILE_SIZE_MB: u64 = 100;

/// Default confidence level for CUE-inferred metadata
pub const CUE_INFERRED_CONFIDENCE: f32 = 1.0;

/// Default confidence level for embedded metadata
pub const EMBEDDED_METADATA_CONFIDENCE: f32 = 1.0;

/// Default confidence level for user-edited metadata
pub const USER_EDITED_METADATA_CONFIDENCE: f32 = 1.0;

/// Newtype wrapper for track titles to provide type safety
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrackTitle(pub String);

/// Newtype wrapper for artist names to provide type safety
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArtistName(pub String);

/// Newtype wrapper for album names to provide type safety
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AlbumName(pub String);

/// Newtype wrapper for file paths to provide type safety
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FilePath(pub std::path::PathBuf);

/// Newtype wrapper for track numbers to provide type safety
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TrackNumber(pub u32);

/// Newtype wrapper for disc numbers to provide type safety
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DiscNumber(pub u32);

/// Newtype wrapper for years to provide type safety
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Year(pub u32);

/// Newtype wrapper for durations to provide type safety
#[derive(Debug, Clone, PartialEq)]
pub struct Duration(pub f64);

/// Newtype wrapper for confidence values to provide type safety
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Confidence(pub f32);