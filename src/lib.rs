// Domain definitions

//! Domain models used by musicctl.

pub mod domain {
    use serde::{Deserialize, Serialize};
    use std::path::{Path, PathBuf};

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

    /// Infer artist name from track file path
    pub fn infer_artist_from_path(track_path: &Path) -> Option<String> {
        // Path structure should be: Artist/Album/track.flac
        // Only infer artist when we have exactly Artist/Album/track.flac structure

        let components: Vec<&str> = track_path
            .components()
            .filter_map(|c| c.as_os_str().to_str())
            .collect();

        // Need exactly Artist/Album/track.flac (4 components minimum)
        if components.len() >= 4 {
            let _track_name = components.last()?; // filename
            let album_name = components[components.len() - 2]; // parent directory
            let potential_artist = components[components.len() - 3]; // grandparent directory

            // Only infer if we have a clear Artist/Album/track.flac pattern
            // and parent directory is not the same as Artist (to avoid Album/Album/track.flac where both are "Album")
            if !potential_artist.is_empty()
                && !album_name.is_empty()
                && potential_artist != album_name
            {
                return Some(potential_artist.to_string());
            }
        }

        None
    }

    /// Infer album name from track file path
    pub fn infer_album_from_path(track_path: &Path) -> Option<String> {
        // Path structure should be: Artist/Album/track.flac or Album/track.flac
        // Only infer album when we have at least Album/track.flac structure (3 components minimum)

        let components: Vec<&str> = track_path
            .components()
            .filter_map(|c| c.as_os_str().to_str())
            .collect();

        // Need at least Album/track.flac (3 components minimum)
        if components.len() >= 3 {
            // The component right before the filename should be the album
            let potential_album = components[components.len() - 2]; // parent directory

            // Only infer if we have a valid album name
            if !potential_album.is_empty() {
                return Some(potential_album.to_string());
            }
        }

        None
    }

    /// Convert string to title case (first letter of each word capitalized)
    pub fn to_title_case(input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        let mut capitalize_next = true;

        for c in input.chars() {
            if c.is_whitespace() || c == '-' || c == '_' {
                capitalize_next = true;
                result.push(c);
            } else if capitalize_next {
                for uppercase_char in c.to_uppercase() {
                    result.push(uppercase_char);
                }
                capitalize_next = false;
            } else {
                for lowercase_char in c.to_lowercase() {
                    result.push(lowercase_char);
                }
            }
        }

        result
    }

    /// Result of a normalization operation
    #[derive(Debug, Clone)]
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

    /// Normalize track titles to title case
    pub fn normalize_track_titles(
        path: &Path,
        dry_run: bool,
    ) -> Result<Vec<OperationResult>, String> {
        let mut results = Vec::new();

        // Check if path is a file or directory
        if path.is_file() {
            // Single file
            let track = match crate::infra::audio::flac::read_flac_metadata(path) {
                Ok(track) => track,
                Err(e) => return Err(format!("Failed to read {}: {}", path.display(), e)),
            };

            results.push(normalize_single_track(track, dry_run));
        } else if path.is_dir() {
            // Directory - scan for FLAC files
            let tracks = crate::infra::scanner::scan_dir(path);
            for track in tracks {
                results.push(normalize_single_track(track, dry_run));
            }
        } else {
            return Err(format!("Path does not exist: {}", path.display()));
        }

        Ok(results)
    }

    /// Normalize a single track's title
    fn normalize_single_track(track: Track, dry_run: bool) -> OperationResult {
        let current_title = match &track.metadata.title {
            Some(title) => &title.value,
            None => {
                return OperationResult::Error {
                    track,
                    error: "No title found".to_string(),
                }
            }
        };

        let normalized_title = to_title_case(current_title);
        let old_title = current_title.clone();

        // Check if title needs to be changed
        if current_title == &normalized_title {
            return OperationResult::NoChange { track };
        }

        if dry_run {
            // Just return what would be changed
            OperationResult::Updated {
                track,
                old_title,
                new_title: normalized_title,
            }
        } else {
            // Actually update the metadata
            // TODO: Implement actual FLAC metadata writing
            // For now, just return the operation result
            OperationResult::Updated {
                track,
                old_title,
                new_title: normalized_title,
            }
        }
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

// Build library hierarchy from flat track list
pub fn build_library_hierarchy(tracks: Vec<Track>) -> Library {
    use std::collections::HashMap;
    use std::path::PathBuf;

    let mut artists_map: HashMap<String, Vec<Track>> = HashMap::new();

    // Group tracks by artist
    for track in tracks {
        let artist_name = track
            .metadata
            .artist
            .as_ref()
            .map(|a| a.value.clone())
            .unwrap_or_else(|| "Unknown Artist".to_string());

        artists_map.entry(artist_name).or_default().push(track);
    }

    let mut library = Library::new();

    // Build artist -> album -> track hierarchy
    for (artist_name, artist_tracks) in artists_map {
        let mut albums_map: HashMap<String, Vec<Track>> = HashMap::new();

        // Group tracks by album
        for track in artist_tracks {
            let album_name = track
                .metadata
                .album
                .as_ref()
                .map(|a| a.value.clone())
                .unwrap_or_else(|| "Unknown Album".to_string());

            albums_map.entry(album_name).or_default().push(track);
        }

        let mut albums = Vec::new();
        for (album_name, album_tracks) in albums_map {
            // Extract year from first track (assuming all tracks in album have same year)
            let year = album_tracks
                .first()
                .and_then(|t| t.metadata.year.as_ref())
                .map(|y| y.value);

            // Capture album path before moving tracks
            let album_path = album_tracks
                .first()
                .map(|t| t.file_path.parent().unwrap().to_path_buf())
                .unwrap_or_else(|| PathBuf::from(""));

            let mut track_nodes = Vec::new();
            for track in album_tracks {
                track_nodes.push(TrackNode {
                    file_path: track.file_path,
                    metadata: track.metadata,
                });
            }

            albums.push(AlbumNode {
                title: album_name,
                year,
                tracks: track_nodes,
                path: album_path,
            });
        }

        library.add_artist(ArtistNode {
            name: artist_name,
            albums,
        });
    }

    library
}

// Re‑export for external use
pub use domain::{
    infer_album_from_path, infer_artist_from_path, normalize_track_titles, to_title_case,
    AlbumNode, ArtistNode, Library, MetadataSource, MetadataValue, OperationResult, Track,
    TrackMetadata, TrackNode,
};

pub mod infra {
    use super::domain::{MetadataValue, Track, TrackMetadata};
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
                    // Infer artist from directory structure
                    let inferred_artist = crate::infer_artist_from_path(path)
                        .map(|artist| MetadataValue::inferred(artist, 0.8));

                    let inferred_album = crate::infer_album_from_path(path)
                        .map(|album| MetadataValue::inferred(album, 0.8));

                    let metadata = TrackMetadata {
                        title: None,
                        artist: inferred_artist,
                        album: inferred_album,
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
            use lofty::{
                file::{AudioFile, TaggedFile, TaggedFileExt},
                prelude::ItemKey,
                read_from_path,
                tag::ItemValue,
            };

            /// Read actual FLAC metadata using lofty library.
            pub fn read_flac_metadata(path: &Path) -> Result<Track, io::Error> {
                if !path.is_file()
                    || !path
                        .extension()
                        .map_or(false, |ext| ext.eq_ignore_ascii_case("flac"))
                {
                    return Err(Error::new(ErrorKind::InvalidInput, "Not a FLAC file"));
                }

                // Use lofty to read the file (without properties initially)
                let tagged_file = match read_from_path(path) {
                    Ok(file) => file,
                    Err(e) => {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            format!("Failed to read FLAC file: {}", e),
                        ))
                    }
                };

                // Extract metadata from tags and file properties
                let metadata = extract_metadata_from_tags(&tagged_file, path);

                Ok(Track {
                    file_path: path.to_path_buf(),
                    metadata,
                })
            }

            /// Extract metadata from lofty TaggedFile and convert to our TrackMetadata
            fn extract_metadata_from_tags(tagged_file: &TaggedFile, path: &Path) -> TrackMetadata {
                use crate::infer_album_from_path;
                use crate::infer_artist_from_path;
                let mut title = None;
                let mut artist = None;
                let mut album = None;
                let mut album_artist = None;
                let mut track_number = None;
                let mut disc_number = None;
                let mut year = None;
                let mut genre = None;
                // Get the primary tag (usually Vorbis Comments for FLAC)
                if let Some(tag) = tagged_file.primary_tag() {
                    for tag_item in tag.items() {
                        // Helper function to convert ItemValue to string
                        let item_value_str = match tag_item.value() {
                            ItemValue::Text(s) => s.to_string(),
                            ItemValue::Locator(s) => s.to_string(),
                            ItemValue::Binary(_) => format!("<binary data>"),
                        };

                        match tag_item.key() {
                            ItemKey::TrackTitle => {
                                title = Some(MetadataValue::embedded(item_value_str));
                            }
                            ItemKey::TrackArtist => {
                                artist = Some(MetadataValue::embedded(item_value_str));
                            }
                            ItemKey::AlbumTitle => {
                                album = Some(MetadataValue::embedded(item_value_str));
                            }
                            ItemKey::AlbumArtist => {
                                album_artist = Some(MetadataValue::embedded(item_value_str));
                            }
                            ItemKey::TrackNumber => {
                                if let Ok(num) = item_value_str.parse::<u32>() {
                                    track_number = Some(MetadataValue::embedded(num));
                                }
                            }
                            ItemKey::DiscNumber => {
                                if let Ok(num) = item_value_str.parse::<u32>() {
                                    disc_number = Some(MetadataValue::embedded(num));
                                }
                            }
                            ItemKey::Year => {
                                if let Ok(year_val) = item_value_str.parse::<u32>() {
                                    year = Some(MetadataValue::embedded(year_val));
                                }
                            }

                            ItemKey::Genre => {
                                genre = Some(MetadataValue::embedded(item_value_str));
                            }
                            ItemKey::RecordingDate => {
                                let clean_value = item_value_str.trim();
                                if let Ok(year_val) = clean_value.parse::<u32>() {
                                    year = Some(MetadataValue::embedded(year_val));
                                }
                            }
                            _ => {} // Ignore other tags for now
                        }
                    }
                }

                // Get duration from file properties (direct reference)
                let properties = tagged_file.properties();
                let duration = Some(MetadataValue::embedded(properties.duration().as_secs_f64()));

                // Apply folder inference as fallback when embedded metadata is missing
                let inferred_artist = if artist.is_none() {
                    infer_artist_from_path(path).map(|artist| MetadataValue::inferred(artist, 0.8))
                } else {
                    artist
                };

                let inferred_album = if album.is_none() {
                    infer_album_from_path(path).map(|album| MetadataValue::inferred(album, 0.8))
                } else {
                    album
                };

                TrackMetadata {
                    title,
                    artist: inferred_artist,
                    album: inferred_album,
                    album_artist,
                    track_number,
                    disc_number,
                    year,
                    genre,
                    duration,
                    format: "flac".to_string(),
                    path: path.to_path_buf(),
                }
            }
        }
    }
}

// Re‑export infra modules
pub use infra::audio;
pub use infra::scanner;
