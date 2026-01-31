//! Format-agnostic directory scanner for music files.

use crate::domain::models::{MetadataValue, Track, TrackMetadata};
use crate::infrastructure::formats;
use crate::services::inference::{infer_album_from_path, infer_artist_from_path};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Recursively scan `base` for supported music files and return a vector of Track.
/// Uses deterministic ordering: sorted paths for consistent output.
pub fn scan_dir(base: &Path) -> Vec<Track> {
    let mut tracks_map = BTreeMap::new();
    let supported_extensions = formats::get_supported_extensions();

    for entry in WalkDir::new(base)
        .follow_links(false) // Don't follow symlinks for determinism
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.is_file() && is_supported_audio_file(path, &supported_extensions) {
            // Infer basic info from directory structure first (faster than full metadata read)
            let inferred_artist =
                infer_artist_from_path(path).map(|artist| MetadataValue::inferred(artist, 0.8));

            let inferred_album =
                infer_album_from_path(path).map(|album| MetadataValue::inferred(album, 0.8));

            // Get file extension for format identification
            let format = path
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("unknown")
                .to_lowercase();

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
                format,
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
    let supported_extensions = formats::get_supported_extensions();

    for entry in WalkDir::new(base)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.is_file() && is_supported_audio_file(path, &supported_extensions) {
            paths.push(path.to_path_buf());
        }
    }

    // Sort for deterministic ordering
    paths.sort();
    paths
}

/// Scan and read full metadata for all files in directory
pub fn scan_dir_with_metadata(base: &Path) -> Result<Vec<Track>, Box<dyn std::error::Error>> {
    let mut tracks_map = BTreeMap::new();

    for entry in WalkDir::new(base)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.is_file() && formats::is_format_supported(path) {
            match formats::read_metadata(path) {
                Ok(track) => {
                    tracks_map.insert(path.to_path_buf(), track);
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to read metadata for {}: {}",
                        path.display(),
                        e
                    );
                }
            }
        }
    }

    Ok(tracks_map.into_values().collect())
}

/// Check if a file is a supported audio file
fn is_supported_audio_file(path: &Path, supported_extensions: &[String]) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| supported_extensions.contains(&ext.to_lowercase()))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_is_supported_audio_file() {
        let extensions = vec!["flac".to_string(), "mp3".to_string()];

        assert!(is_supported_audio_file(
            &PathBuf::from("test.flac"),
            &extensions
        ));
        assert!(is_supported_audio_file(
            &PathBuf::from("test.FLAC"),
            &extensions
        ));
        assert!(is_supported_audio_file(
            &PathBuf::from("test.mp3"),
            &extensions
        ));
        assert!(!is_supported_audio_file(
            &PathBuf::from("test.wav"),
            &extensions
        ));
        assert!(!is_supported_audio_file(
            &PathBuf::from("test.txt"),
            &extensions
        ));
        assert!(!is_supported_audio_file(
            &PathBuf::from("test"),
            &extensions
        ));
    }
}
