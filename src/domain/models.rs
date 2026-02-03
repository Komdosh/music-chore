//! Core domain models for music library representation.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Source of metadata information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MetadataSource {
    /// From file metadata
    Embedded,
    /// Inferred from directory structure
    FolderInferred,
    /// Explicitly set by user
    UserEdited,
}

/// Wrapper for metadata values with provenance
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetadataValue<T> {
    pub value: T,
    pub source: MetadataSource,
    pub confidence: f32,
}

impl<T> MetadataValue<T> {
    pub fn embedded(value: T) -> Self {
        Self {
            value,
            source: MetadataSource::Embedded,
            confidence: 1.0,
        }
    }

    pub fn inferred(value: T, confidence: f32) -> Self {
        Self {
            value,
            source: MetadataSource::FolderInferred,
            confidence,
        }
    }

    pub fn user_set(value: T) -> Self {
        Self {
            value,
            source: MetadataSource::UserEdited,
            confidence: 1.0,
        }
    }
}

/// Track metadata with provenance tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrackMetadata {
    pub title: Option<MetadataValue<String>>,
    pub artist: Option<MetadataValue<String>>,
    pub album: Option<MetadataValue<String>>,
    pub album_artist: Option<MetadataValue<String>>,
    pub track_number: Option<MetadataValue<u32>>,
    pub disc_number: Option<MetadataValue<u32>>,
    pub year: Option<MetadataValue<u32>>,
    pub genre: Option<MetadataValue<String>>,
    pub duration: Option<MetadataValue<f64>>, // seconds
    pub format: String,
    pub path: PathBuf,
}

/// Basic representation of a music track.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Track {
    pub file_path: PathBuf,
    pub metadata: TrackMetadata,
    pub checksum: Option<String>,
}

impl Track {
    /// Create a new track without checksum
    pub fn new(file_path: PathBuf, metadata: TrackMetadata) -> Self {
        Self {
            file_path,
            metadata,
            checksum: None,
        }
    }

    /// Create a new track with checksum
    pub fn with_checksum(file_path: PathBuf, metadata: TrackMetadata, checksum: String) -> Self {
        Self {
            file_path,
            metadata,
            checksum: Some(checksum),
        }
    }

    /// Calculate SHA256 checksum of the file
    pub fn calculate_checksum(&self) -> Result<String, Box<dyn std::error::Error>> {
        use sha2::{Digest, Sha256};
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(&self.file_path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }
}

/// Album node in library hierarchy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AlbumNode {
    pub title: String,
    pub year: Option<u32>,
    pub tracks: Vec<TrackNode>,
    pub path: PathBuf,
}

/// Track node with simplified info for tree display
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrackNode {
    pub file_path: PathBuf,
    pub metadata: TrackMetadata,
}

/// Artist node in library hierarchy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArtistNode {
    pub name: String,
    pub albums: Vec<AlbumNode>,
}

/// Complete library representation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Library {
    pub artists: Vec<ArtistNode>,
    pub total_tracks: usize,
    pub total_artists: usize,
    pub total_albums: usize,
}

/// Result of a normalization operation
#[derive(Debug, Clone, serde::Serialize)]
pub enum OperationResult {
    Updated {
        track: Track,
        old_title: String,
        new_title: String,
    },
    NoChange {
        track: Track,
    },
    Error {
        track: Track,
        error: String,
    },
}

impl Library {
    pub fn new() -> Self {
        Self {
            artists: Vec::new(),
            total_tracks: 0,
            total_artists: 0,
            total_albums: 0,
        }
    }

    pub fn add_artist(&mut self, artist: ArtistNode) {
        self.total_artists += 1;
        self.total_albums += artist.albums.len();
        for album in &artist.albums {
            self.total_tracks += album.tracks.len();
        }
        self.artists.push(artist);
    }
}
