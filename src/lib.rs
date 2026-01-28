// Domain definitions

//! Domain models used by musicctl.

pub mod domain {
    use serde::{Deserialize, Serialize};
    use std::path::PathBuf;

    /// Source of metadata information
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub enum MetadataSource {
        Embedded,       // From file metadata
        FolderInferred, // Inferred from directory structure
        UserEdited,     // Explicitly set by user
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
}

// Re‑export for external use
pub use domain::{
    AlbumNode, ArtistNode, Library, MetadataSource, MetadataValue, Track, TrackMetadata, TrackNode,
};

pub mod infra {
    use super::domain::{MetadataSource, MetadataValue, Track, TrackMetadata};
    use std::path::{Path, PathBuf};
    use walkdir::WalkDir;

    pub mod scanner {
        use super::*;
        use std::collections::BTreeMap;

        /// Recursively scan `base` for .flac files and return a vector of Track.
        /// Uses deterministic ordering: sorted paths for consistent output.
        pub fn scan_dir(base: &Path) -> Vec<Track> {
            let mut tracks_map = BTreeMap::new();

            for entry in WalkDir::new(base)
                .follow_links(false) // Don't follow symlinks for determinism
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();

                if path.is_file()
                    && path
                        .extension()
                        .map_or(false, |ext| ext.eq_ignore_ascii_case("flac"))
                {
                    let metadata = TrackMetadata {
                        title: None,
                        artist: None,
                        album: None,
                        album_artist: None,
                        track_number: None,
                        disc_number: None,
                        year: None,
                        genre: None,
                        duration: None,
                        format: "flac".to_string(),
                        path: path.to_path_buf(),
                    };

                    let track = Track {
                        file_path: path.to_path_buf(),
                        metadata,
                    };

                    tracks_map.insert(path.to_path_buf(), track);
                }
            }

            // Convert to sorted vector
            tracks_map.into_values().collect()
        }

        /// Scan and return basic file paths only (for simple operations)
        pub fn scan_dir_paths(base: &Path) -> Vec<PathBuf> {
            let mut paths = Vec::new();

            for entry in WalkDir::new(base)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();

                if path.is_file()
                    && path
                        .extension()
                        .map_or(false, |ext| ext.eq_ignore_ascii_case("flac"))
                {
                    paths.push(path.to_path_buf());
                }
            }

            // Sort for deterministic ordering
            paths.sort();
            paths
        }
    }

    pub mod audio {
        use super::*;
        use std::io::{self, Error, ErrorKind};

        pub mod flac {
            use super::*;
            /// Stub implementation: ensures file exists and is .flac.
            pub fn read_flac_metadata(path: &Path) -> Result<Track, io::Error> {
                if path.is_file()
                    && path
                        .extension()
                        .map_or(false, |ext| ext.eq_ignore_ascii_case("flac"))
                {
                    let metadata = TrackMetadata {
                        title: None,
                        artist: None,
                        album: None,
                        album_artist: None,
                        track_number: None,
                        disc_number: None,
                        year: None,
                        genre: None,
                        duration: None,
                        format: "flac".to_string(),
                        path: path.to_path_buf(),
                    };

                    Ok(Track {
                        file_path: path.to_path_buf(),
                        metadata,
                    })
                } else {
                    Err(Error::new(ErrorKind::InvalidInput, "Not a FLAC file"))
                }
            }
        }
    }
}

// Re‑export infra modules
pub use infra::audio;
pub use infra::scanner;
