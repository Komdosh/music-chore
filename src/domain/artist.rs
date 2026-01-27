//! Artist data model and related logic.

use std::collections::HashMap;

/// Represents a music artist
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Artist {
    /// The artist's name
    pub name: String,

    /// Metadata provenance for the artist name
    pub provenance: Provenance,

    /// Additional metadata fields
    pub metadata: HashMap<String, String>,
}

impl Artist {
    /// Creates a new artist with the given name and provenance
    pub fn new(name: String, provenance: Provenance) -> Self {
        Self {
            name,
            provenance,
            metadata: HashMap::new(),
        }
    }
}

/// Provenance tracking for metadata fields
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Provenance {
    /// Metadata was embedded in the file
    Embedded,

    /// Metadata was inferred from folder structure or filename
    Inferred,

    /// Metadata was user-edited
    UserEdited,
}