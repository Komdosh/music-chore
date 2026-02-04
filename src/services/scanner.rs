//! Format-agnostic directory scanner for music files.

use crate::domain::models::{MetadataValue, Track, TrackMetadata, FOLDER_INFERRED_CONFIDENCE};
use crate::services::formats;
use crate::services::inference::{infer_album_from_path, infer_artist_from_path};
use log::{debug, warn};
use serde_json::to_string_pretty;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Recursively scan `base` for supported music files and return a vector of Track.
/// Uses deterministic ordering: sorted by filename for consistent output.
/// Logs warnings for unsupported file types found during scan.
pub fn scan_dir(base: &Path) -> Vec<Track> {
    let mut tracks = Vec::new();
    let supported_extensions = formats::get_supported_extensions();

    for entry in WalkDir::new(base)
        .follow_links(false) // Don't follow symlinks for determinism
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.is_file() {
            if is_supported_audio_file(path, &supported_extensions) {
                // Infer basic info from directory structure first (faster than full metadata read)
                let inferred_artist = infer_artist_from_path(path)
                    .map(|artist| MetadataValue::inferred(artist, FOLDER_INFERRED_CONFIDENCE));

                let inferred_album = infer_album_from_path(path)
                    .map(|album| MetadataValue::inferred(album, FOLDER_INFERRED_CONFIDENCE));

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

                let track = Track::new(path.to_path_buf(), metadata);
                tracks.push(track);
            } else if has_audio_extension(path) {
                // File has an audio extension but format is not supported
                warn!(
                    "Unsupported audio format: {} (supported: {})",
                    path.display(),
                    supported_extensions.join(", ")
                );
            }
        }
    }

    // Sort by filename for deterministic ordering
    tracks.sort_by(|a, b| {
        let file_a = a.file_path.file_name().unwrap_or_default();
        let file_b = b.file_path.file_name().unwrap_or_default();
        debug!("Comparing files: {:?} vs {:?}", file_a, file_b);
        file_a.cmp(&file_b)
    });

    tracks
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

/// Scan only the immediate directory level (non-recursive) for music files.
pub fn scan_dir_immediate(base: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let supported_extensions = formats::get_supported_extensions();

    if !base.exists() || !base.is_dir() {
        return paths;
    }

    if let Ok(entries) = std::fs::read_dir(base) {
        for entry in entries.into_iter().flatten() {
            let path = entry.path();
            if path.is_file() && is_supported_audio_file(&path, &supported_extensions) {
                paths.push(path);
            }
        }
    }

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

/// Scan for tracks and detect duplicates by checksum
pub fn scan_with_duplicates(base: &Path) -> (Vec<Track>, Vec<Vec<Track>>) {
    let tracks = scan_dir(base);
    let mut checksum_map = std::collections::HashMap::new();
    let mut tracks_with_checksums = Vec::new();

    for mut track in tracks {
        match track.calculate_checksum() {
            Ok(checksum) => {
                track.checksum = Some(checksum.clone());
                checksum_map
                    .entry(checksum)
                    .or_insert_with(Vec::new)
                    .push(track.clone());
                tracks_with_checksums.push(track);
            }
            Err(e) => {
                eprintln!(
                    "Warning: Failed to calculate checksum for {}: {}",
                    track.file_path.display(),
                    e
                );
                tracks_with_checksums.push(track);
            }
        }
    }

    let duplicates: Vec<Vec<Track>> = checksum_map
        .into_values()
        .filter(|group| group.len() > 1)
        .collect();

    (tracks_with_checksums, duplicates)
}

pub fn scan_tracks(path: PathBuf, json: bool) -> Result<String, String> {
    let tracks = scan_dir(&path);

    if tracks.is_empty() {
        return Err(format!(
            "No music files found in directory: {}",
            path.display()
        ));
    }

    if json {
        match to_string_pretty(&tracks) {
            Ok(s) => Ok(s),
            Err(e) => Err(format!("Error serializing to JSON: {}", e)),
        }
    } else {
        let mut out = String::new();

        for track in tracks {
            out.push_str(&format!("{}\n", track.file_path.display()));
        }

        Ok(out)
    }
}

/// Scan directory with optional max depth limit.
/// None = unlimited depth (full recursion)
/// Some(0) = immediate directory only (like ls)
/// Some(1) = base dir + 1 level deep
/// Some(2) = base dir + 2 levels deep, etc.
pub fn scan_dir_with_depth(base: &Path, max_depth: Option<usize>) -> Vec<Track> {
    let supported_extensions = formats::get_supported_extensions();

    let mut walkdir = WalkDir::new(base).follow_links(false);

    if let Some(depth) = max_depth {
        walkdir = walkdir.max_depth(depth + 1); // +1 because WalkDir counts the base as depth 0
    }

    let mut tracks = Vec::new();

    for entry in walkdir.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.is_file() {
            if is_supported_audio_file(path, &supported_extensions) {
                let inferred_artist = infer_artist_from_path(path)
                    .map(|artist| MetadataValue::inferred(artist, FOLDER_INFERRED_CONFIDENCE));

                let inferred_album = infer_album_from_path(path)
                    .map(|album| MetadataValue::inferred(album, FOLDER_INFERRED_CONFIDENCE));

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

                let track = Track::new(path.to_path_buf(), metadata);
                tracks.push(track);
            } else if has_audio_extension(path) {
                warn!(
                    "Unsupported audio format: {} (supported: {})",
                    path.display(),
                    supported_extensions.join(", ")
                );
            }
        }
    }

    tracks.sort_by(|a, b| {
        let file_a = a.file_path.file_name().unwrap_or_default();
        let file_b = b.file_path.file_name().unwrap_or_default();
        file_a.cmp(&file_b)
    });

    tracks
}

/// Scan directory with optional max depth limit and symlink handling.
/// If follow_symlinks is true, symbolic links to files are followed.
/// None = unlimited depth (full recursion)
/// Some(0) = immediate files only (like ls)
/// Some(1) = base dir + 1 level deep
/// Some(2) = base dir + 2 levels deep, etc.
pub fn scan_dir_with_depth_and_symlinks(
    base: &Path,
    max_depth: Option<usize>,
    follow_symlinks: bool,
) -> Vec<Track> {
    let supported_extensions = formats::get_supported_extensions();

    let mut walkdir = WalkDir::new(base).follow_links(follow_symlinks);

    if let Some(depth) = max_depth {
        walkdir = walkdir.max_depth(depth + 1);
    }

    let mut tracks = Vec::new();

    for entry in walkdir.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.is_file() {
            if is_supported_audio_file(path, &supported_extensions) {
                let inferred_artist = infer_artist_from_path(path)
                    .map(|artist| MetadataValue::inferred(artist, FOLDER_INFERRED_CONFIDENCE));

                let inferred_album = infer_album_from_path(path)
                    .map(|album| MetadataValue::inferred(album, FOLDER_INFERRED_CONFIDENCE));

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

                let track = Track::new(path.to_path_buf(), metadata);
                tracks.push(track);
            } else if has_audio_extension(path) {
                warn!(
                    "Unsupported audio format: {} (supported: {})",
                    path.display(),
                    supported_extensions.join(", ")
                );
            }
        }
    }

    tracks.sort_by(|a, b| {
        let file_a = a.file_path.file_name().unwrap_or_default();
        let file_b = b.file_path.file_name().unwrap_or_default();
        file_a.cmp(&file_b)
    });

    tracks
}

/// Check if a file is a supported audio file
fn is_supported_audio_file(path: &Path, supported_extensions: &[String]) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| supported_extensions.contains(&ext.to_lowercase()))
        .unwrap_or(false)
}

/// Check if a file has an audio extension (regardless of support)
fn has_audio_extension(path: &Path) -> bool {
    const AUDIO_EXTENSIONS: &[&str] = &[
        "flac", "mp3", "wav", "ogg", "m4a", "aac", "wma", "aiff", "dsf", "opus", "webm",
    ];
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| AUDIO_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
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

    #[test]
    fn test_unicode_paths() {
        // Test that Unicode characters in paths are handled correctly
        let unicode_path = PathBuf::from("björk/album/track.flac");
        assert!(unicode_path.to_str().is_some());

        let complex_unicode = PathBuf::from("éxito ñoño/café/track.flac");
        assert!(complex_unicode.to_str().is_some());
    }

    #[test]
    fn test_scan_dir_with_depth_unlimited() {
        let base = PathBuf::from("tests/fixtures/inference");
        let tracks = scan_dir_with_depth(&base, None);
        assert!(tracks.len() >= 6);
    }

    #[test]
    fn test_scan_dir_with_depth_zero() {
        let base = PathBuf::from("tests/fixtures/inference/flat");
        let tracks = scan_dir_with_depth(&base, Some(0));
        assert!(tracks.is_empty());
    }

    #[test]
    fn test_scan_dir_with_depth_one() {
        let base = PathBuf::from("tests/fixtures/inference");
        let tracks = scan_dir_with_depth(&base, Some(1));
        assert_eq!(tracks.len(), 1);
        assert!(tracks[0]
            .file_path
            .to_string_lossy()
            .contains("root/track.flac"));
    }

    #[test]
    fn test_scan_dir_with_depth_two() {
        let base = PathBuf::from("tests/fixtures/inference");
        let tracks = scan_dir_with_depth(&base, Some(2));
        assert_eq!(tracks.len(), 2);
    }

    #[test]
    fn test_has_audio_extension() {
        assert!(has_audio_extension(&PathBuf::from("test.flac")));
        assert!(has_audio_extension(&PathBuf::from("test.mp3")));
        assert!(has_audio_extension(&PathBuf::from("test.wav")));
        assert!(has_audio_extension(&PathBuf::from("test.ogg")));
        assert!(has_audio_extension(&PathBuf::from("test.m4a")));
        assert!(has_audio_extension(&PathBuf::from("test.dsf")));
        assert!(has_audio_extension(&PathBuf::from("test.FLAC")));
        assert!(!has_audio_extension(&PathBuf::from("test.txt")));
        assert!(!has_audio_extension(&PathBuf::from("test.jpg")));
        assert!(!has_audio_extension(&PathBuf::from("test")));
    }
}
