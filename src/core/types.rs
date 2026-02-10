//! Type-safe wrappers for semantic types in the music chore application.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Newtype wrapper for track titles to provide type safety
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TrackTitle(pub String);

impl From<String> for TrackTitle {
    fn from(s: String) -> Self {
        TrackTitle(s)
    }
}

impl From<&str> for TrackTitle {
    fn from(s: &str) -> Self {
        TrackTitle(s.to_string())
    }
}

impl AsRef<str> for TrackTitle {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Newtype wrapper for artist names to provide type safety
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArtistName(pub String);

impl From<String> for ArtistName {
    fn from(s: String) -> Self {
        ArtistName(s)
    }
}

impl From<&str> for ArtistName {
    fn from(s: &str) -> Self {
        ArtistName(s.to_string())
    }
}

impl AsRef<str> for ArtistName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Newtype wrapper for album names to provide type safety
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AlbumName(pub String);

impl From<String> for AlbumName {
    fn from(s: String) -> Self {
        AlbumName(s)
    }
}

impl From<&str> for AlbumName {
    fn from(s: &str) -> Self {
        AlbumName(s.to_string())
    }
}

impl AsRef<str> for AlbumName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Newtype wrapper for file paths to provide type safety
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FilePath(pub PathBuf);

impl From<PathBuf> for FilePath {
    fn from(p: PathBuf) -> Self {
        FilePath(p)
    }
}

impl From<&PathBuf> for FilePath {
    fn from(p: &PathBuf) -> Self {
        FilePath(p.clone())
    }
}

impl AsRef<PathBuf> for FilePath {
    fn as_ref(&self) -> &PathBuf {
        &self.0
    }
}

/// Newtype wrapper for track numbers to provide type safety
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TrackNumber(pub u32);

impl From<u32> for TrackNumber {
    fn from(n: u32) -> Self {
        TrackNumber(n)
    }
}

impl From<&u32> for TrackNumber {
    fn from(n: &u32) -> Self {
        TrackNumber(*n)
    }
}

impl AsRef<u32> for TrackNumber {
    fn as_ref(&self) -> &u32 {
        &self.0
    }
}

/// Newtype wrapper for disc numbers to provide type safety
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DiscNumber(pub u32);

impl From<u32> for DiscNumber {
    fn from(n: u32) -> Self {
        DiscNumber(n)
    }
}

impl From<&u32> for DiscNumber {
    fn from(n: &u32) -> Self {
        DiscNumber(*n)
    }
}

impl AsRef<u32> for DiscNumber {
    fn as_ref(&self) -> &u32 {
        &self.0
    }
}

/// Newtype wrapper for years to provide type safety
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Year(pub u32);

impl From<u32> for Year {
    fn from(n: u32) -> Self {
        Year(n)
    }
}

impl From<&u32> for Year {
    fn from(n: &u32) -> Self {
        Year(*n)
    }
}

impl AsRef<u32> for Year {
    fn as_ref(&self) -> &u32 {
        &self.0
    }
}

/// Newtype wrapper for durations to provide type safety
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Duration(pub f64);

impl From<f64> for Duration {
    fn from(n: f64) -> Self {
        Duration(n)
    }
}

impl From<&f64> for Duration {
    fn from(n: &f64) -> Self {
        Duration(*n)
    }
}

impl AsRef<f64> for Duration {
    fn as_ref(&self) -> &f64 {
        &self.0
    }
}

/// Newtype wrapper for confidence values to provide type safety
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Confidence(pub f32);

impl From<f32> for Confidence {
    fn from(n: f32) -> Self {
        Confidence(n)
    }
}

impl From<&f32> for Confidence {
    fn from(n: &f32) -> Self {
        Confidence(*n)
    }
}

impl AsRef<f32> for Confidence {
    fn as_ref(&self) -> &f32 {
        &self.0
    }
}

/// Enum representing the different modes for metadata operations
#[derive(Debug, Clone, PartialEq)]
pub enum MetadataOperationMode {
    /// Apply changes to files (permanent changes)
    Apply,
    /// Dry run mode - show what would change without modifying files
    DryRun,
    /// Validate metadata without applying changes
    Validate,
}
