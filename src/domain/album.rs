//! Album data model and related logic.

use std::collections::HashMap;
use crate::domain::artist::Artist;
use crate::domain::track::Track;

/// Represents a music album
#[derive(Debug, Clone, PartialEq)]
pub struct Album {
    /// The album's title
    pub title: String,

    /// The album's artist
    pub artist: Artist,

    /// The album's year (optional)
    pub year: Option<u32>,

    /// The album's genre (optional)
    pub genre: Option<String>,

    /// The album's tracks
    pub tracks: Vec<Track>,

    /// Metadata provenance for the album
    pub provenance: Provenance,

    /// Additional metadata fields
    pub metadata: HashMap<String, String>,
}

impl Album {
    /// Creates a new album with the given parameters
    pub fn new(
        title: String,
        artist: Artist,
        year: Option<u32>,
        genre: Option<String>,
        tracks: Vec<Track>,
        provenance: Provenance,
    ) -> Self {
        Self {
            title,
            artist,
            year,
            genre,
            tracks,
            provenance,
            metadata: HashMap::new(),
        }
    }

    /// Adds a track to the album
    pub fn add_track(&mut self, track: Track) {
        self.tracks.push(track);
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