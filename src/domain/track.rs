//! Track data model and related logic.

use std::collections::HashMap;

/// Represents a music track
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Track {
    /// The track's title
    pub title: String,

    /// The track's number in the album (optional)
    pub track_number: Option<u32>,

    /// The track's duration in seconds (optional)
    pub duration: Option<u32>,

    /// The track's file path
    pub file_path: String,

    /// Metadata provenance for the track
    pub provenance: Provenance,

    /// Additional metadata fields
    pub metadata: HashMap<String, String>,
}

impl Track {
    /// Creates a new track with the given parameters
    pub fn new(
        title: String,
        track_number: Option<u32>,
        duration: Option<u32>,
        file_path: String,
        provenance: Provenance,
    ) -> Self {
        Self {
            title,
            track_number,
            duration,
            file_path,
            provenance,
            metadata: HashMap::new(),
        }
    }
}

/// Provenance tracking for metadata fields
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Provenance {
    /// Metadata was embedded in the file
    Embedded,

    /// Metadata was inferred from folder structure or filename
    Inferred,

    /// Metadata was user-edited
    UserEdited,
}