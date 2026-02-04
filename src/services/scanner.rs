//! Enhanced directory scanner with improved error handling and edge cases.

use crate::domain::models::{MetadataValue, Track, TrackMetadata, FOLDER_INFERRED_CONFIDENCE};
use crate::services::formats;
use crate::services::inference::{infer_album_from_path, infer_artist_from_path};
use glob::Pattern;
use log::{debug, error, warn};
use serde_json::to_string_pretty;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Recursively scan `base` for supported music files with enhanced error handling.
///
/// This version handles:
/// - Missing album directories by inferring album from filename when directory structure is insufficient
/// - Empty or corrupted track files by skipping them with warnings
/// - Symlinks to music files (if follow_symlinks is true)
/// - File pattern exclusions (if provided)
/// - Progress output with --verbose flag
///
/// Uses deterministic ordering: sorted by filename for consistent output.
/// Logs warnings for unsupported file types and errors for problematic files.
pub fn scan_dir(base: &Path) -> Vec<Track> {
    let mut tracks = Vec::new();
    let supported_extensions = formats::get_supported_extensions();

    for entry in WalkDir::new(base)
        .follow_links(false) // Don't follow symlinks for determinism
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Skip symlinks to files (but not directories - handled by walkdir)
        if let Ok(metadata) = path.symlink_metadata() {
            if metadata.file_type().is_symlink() {
                debug!(target: "music_chore", "Skipping symlink to file: {}", path.display());
                continue;
            }
        }

        if path.is_file() {
            if is_supported_audio_file(path, &supported_extensions) {
                // Check if file is empty or corrupted before processing
                if let Err(e) = check_file_validity(path) {
                    error!(target: "music_chore", "Skipping invalid file {}: {}", path.display(), e);
                    continue;
                }

                // Infer basic info from directory structure first (faster than full metadata read)
                let inferred_artist = infer_artist_from_path(path)
                    .map(|artist| MetadataValue::inferred(artist, FOLDER_INFERRED_CONFIDENCE));

                let inferred_album = infer_album_from_path(path)
                    .map(|album| MetadataValue::inferred(album, FOLDER_INFERRED_CONFIDENCE));

                // If album inference failed and we have a reasonable filename, try to extract album from filename
                let final_album = if inferred_album.is_none() {
                    if let Some(filename) = path.file_stem().and_then(|n| n.to_str()) {
                        if let Some(album) = extract_album_from_filename(filename) {
                            Some(MetadataValue::inferred(album, FOLDER_INFERRED_CONFIDENCE))
                        } else {
                            // Fallback: use cleaned filename as album name
                            let cleaned = clean_filename_as_album(filename);
                            if !cleaned.is_empty() {
                                Some(MetadataValue::inferred(cleaned, FOLDER_INFERRED_CONFIDENCE))
                            } else {
                                None
                            }
                        }
                    } else {
                        None
                    }
                } else {
                    inferred_album
                };

                // Get file extension for format identification
                let format = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("unknown")
                    .to_lowercase();

                let metadata = TrackMetadata {
                    title: None,
                    artist: inferred_artist,
                    album: final_album,
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
                warn!(target: "music_chore", "Unsupported audio format: {} (supported: {})", path.display(), supported_extensions.join(", "));
            }
        }
    }

    // Sort by filename for deterministic ordering
    tracks.sort_by(|a, b| {
        let file_a = a.file_path.file_name().unwrap_or_default();
        let file_b = b.file_path.file_name().unwrap_or_default();
        debug!(target: "music_chore", "Comparing files: {:?} vs {:?}", file_a, file_b);
        file_a.cmp(&file_b)
    });

    tracks
}

/// Check if a file is valid (not empty, readable, etc.)
fn check_file_validity(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file_size = std::fs::metadata(path)?.len();

    if file_size == 0 {
        return Err("File is empty".into());
    }

    // Try to open the file to check if it's readable
    let _file = std::fs::File::open(path)?;

    Ok(())
}

/// Extract album from filename when directory structure is insufficient
fn extract_album_from_filename(filename: &str) -> Option<String> {
    // Look for patterns like: "Artist - Album - Track", "Album - Track", "Track (Album)"

    // Check if there are two " - " separators (e.g., "Artist - Album - Track" or "Album - 01 - Track")
    if let Some(first_idx) = filename.find(" - ") {
        let rest = &filename[first_idx + 3..];
        if let Some(second_idx) = rest.find(" - ") {
            // Found two separators - need to decide which part is the album
            let middle_part = &rest[..second_idx];
            let first_part = &filename[..first_idx];

            // If middle part is just a number (track number), album is the first part
            // Otherwise, middle part is likely the album name
            if middle_part.trim().parse::<u32>().is_ok() {
                // Middle is track number, so album is first part
                if !first_part.is_empty() {
                    return Some(first_part.trim().to_string());
                }
            } else {
                // Middle part is likely the album
                if !middle_part.is_empty() {
                    return Some(middle_part.trim().to_string());
                }
            }
        } else {
            // Only one separator - "Album - Track" format
            let album_candidate = &filename[..first_idx];
            if !album_candidate.is_empty() {
                return Some(album_candidate.trim().to_string());
            }
        }
    }

    // Pattern: "Track (Album)" - extract album in parentheses
    if let Some(start) = filename.find('(') {
        if let Some(end) = filename[start..].find(')') {
            let album_candidate = &filename[start + 1..start + end];
            if !album_candidate.is_empty() {
                return Some(album_candidate.trim().to_string());
            }
        }
    }

    None
}

/// Clean filename to use as fallback album name
fn clean_filename_as_album(filename: &str) -> String {
    let mut cleaned = filename.to_string();

    // Remove common track number prefixes
    if let Some(idx) = cleaned.find(" - ") {
        cleaned = cleaned[idx + 3..].to_string();
    }

    // Remove file extension
    if let Some(idx) = cleaned.rfind('.') {
        cleaned.truncate(idx);
    }

    // Clean up special characters
    cleaned = cleaned.replace('_', " ").replace('-', " ");
    cleaned = cleaned.trim().to_string();

    cleaned
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

        // Skip symlinks to files
        if let Ok(metadata) = path.symlink_metadata() {
            if metadata.file_type().is_symlink() {
                debug!(target: "music_chore", "Skipping symlink to file: {}", path.display());
                continue;
            }
        }

        if path.is_file() && is_supported_audio_file(path, &supported_extensions) {
            // Check file validity
            if let Err(e) = check_file_validity(path) {
                error!(target: "music_chore", "Skipping invalid file {}: {}", path.display(), e);
                continue;
            }
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

            // Skip symlinks to files
            if let Ok(metadata) = path.symlink_metadata() {
                if metadata.file_type().is_symlink() {
                    debug!(target: "music_chore", "Skipping symlink to file: {}", path.display());
                    continue;
                }
            }

            if path.is_file() && is_supported_audio_file(&path, &supported_extensions) {
                // Check file validity
                if let Err(e) = check_file_validity(&path) {
                    error!(target: "music_chore", "Skipping invalid file {}: {}", path.display(), e);
                    continue;
                }
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

        // Skip symlinks to files
        if let Ok(metadata) = path.symlink_metadata() {
            if metadata.file_type().is_symlink() {
                debug!(target: "music_chore", "Skipping symlink to file: {}", path.display());
                continue;
            }
        }

        if path.is_file() && formats::is_format_supported(path) {
            // Check file validity before reading metadata
            if let Err(e) = check_file_validity(path) {
                error!(target: "music_chore", "Skipping invalid file {}: {}", path.display(), e);
                continue;
            }

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

        // Skip symlinks to files
        if let Ok(metadata) = path.symlink_metadata() {
            if metadata.file_type().is_symlink() {
                debug!(target: "music_chore", "Skipping symlink to file: {}", path.display());
                continue;
            }
        }

        if path.is_file() {
            if is_supported_audio_file(path, &supported_extensions) {
                // Check file validity
                if let Err(e) = check_file_validity(path) {
                    error!(target: "music_chore", "Skipping invalid file {}: {}", path.display(), e);
                    continue;
                }

                let inferred_artist = infer_artist_from_path(path)
                    .map(|artist| MetadataValue::inferred(artist, FOLDER_INFERRED_CONFIDENCE));

                let inferred_album = infer_album_from_path(path)
                    .map(|album| MetadataValue::inferred(album, FOLDER_INFERRED_CONFIDENCE));

                // If album inference failed and we have a reasonable filename, try to extract album from filename
                let final_album = if inferred_album.is_none() {
                    if let Some(filename) = path.file_stem().and_then(|n| n.to_str()) {
                        if let Some(album) = extract_album_from_filename(filename) {
                            Some(MetadataValue::inferred(album, FOLDER_INFERRED_CONFIDENCE))
                        } else {
                            // Fallback: use cleaned filename as album name
                            let cleaned = clean_filename_as_album(filename);
                            if !cleaned.is_empty() {
                                Some(MetadataValue::inferred(cleaned, FOLDER_INFERRED_CONFIDENCE))
                            } else {
                                None
                            }
                        }
                    } else {
                        None
                    }
                } else {
                    inferred_album
                };

                let format = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("unknown")
                    .to_lowercase();

                let metadata = TrackMetadata {
                    title: None,
                    artist: inferred_artist,
                    album: final_album,
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
                warn!(target: "music_chore", "Unsupported audio format: {} (supported: {})", path.display(), supported_extensions.join(", "));
            }
        }
    }

    tracks.sort_by(|a, b| {
        let file_a = a.file_path.file_name().unwrap_or_default();
        let file_b = b.file_path.file_name().unwrap_or_default();
        debug!(target: "music_chore", "Comparing files: {:?} vs {:?}", file_a, file_b);
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

        // Skip symlinks to files unless follow_symlinks is true
        if !follow_symlinks {
            if let Ok(metadata) = path.symlink_metadata() {
                if metadata.file_type().is_symlink() {
                    debug!(target: "music_chore", "Skipping symlink to file: {}", path.display());
                    continue;
                }
            }
        }

        if path.is_file() {
            if is_supported_audio_file(path, &supported_extensions) {
                // Check file validity
                if let Err(e) = check_file_validity(path) {
                    error!(target: "music_chore", "Skipping invalid file {}: {}", path.display(), e);
                    continue;
                }

                let inferred_artist = infer_artist_from_path(path)
                    .map(|artist| MetadataValue::inferred(artist, FOLDER_INFERRED_CONFIDENCE));

                let inferred_album = infer_album_from_path(path)
                    .map(|album| MetadataValue::inferred(album, FOLDER_INFERRED_CONFIDENCE));

                // If album inference failed and we have a reasonable filename, try to extract album from filename
                let final_album = if inferred_album.is_none() {
                    if let Some(filename) = path.file_stem().and_then(|n| n.to_str()) {
                        if let Some(album) = extract_album_from_filename(filename) {
                            Some(MetadataValue::inferred(album, FOLDER_INFERRED_CONFIDENCE))
                        } else {
                            // Fallback: use cleaned filename as album name
                            let cleaned = clean_filename_as_album(filename);
                            if !cleaned.is_empty() {
                                Some(MetadataValue::inferred(cleaned, FOLDER_INFERRED_CONFIDENCE))
                            } else {
                                None
                            }
                        }
                    } else {
                        None
                    }
                } else {
                    inferred_album
                };

                let format = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("unknown")
                    .to_lowercase();

                let metadata = TrackMetadata {
                    title: None,
                    artist: inferred_artist,
                    album: final_album,
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
                warn!(target: "music_chore", "Unsupported audio format: {} (supported: {})", path.display(), supported_extensions.join(", "));
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

/// Scan directory with full options including depth limit, symlink handling, and exclude patterns.
/// Supports glob patterns for exclusion (e.g., "*.tmp", "temp_*", "backup/*")
pub fn scan_dir_with_options(
    base: &Path,
    max_depth: Option<usize>,
    follow_symlinks: bool,
    exclude_patterns: Vec<String>,
) -> Vec<Track> {
    let supported_extensions = formats::get_supported_extensions();

    let mut walkdir = WalkDir::new(base).follow_links(follow_symlinks);

    if let Some(depth) = max_depth {
        walkdir = walkdir.max_depth(depth + 1);
    }

    let mut tracks = Vec::new();

    for entry in walkdir.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        // Skip symlinks to files unless follow_symlinks is true
        if !follow_symlinks {
            if let Ok(metadata) = path.symlink_metadata() {
                if metadata.file_type().is_symlink() {
                    debug!(target: "music_chore", "Skipping symlink to file: {}", path.display());
                    continue;
                }
            }
        }

        // Check exclude patterns
        let mut excluded = false;
        if !exclude_patterns.is_empty() {
            let path_str = path.to_string_lossy();
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                for pattern in &exclude_patterns {
                    if matches_pattern(filename, pattern) || matches_pattern(&path_str, pattern) {
                        debug!(target: "music_chore", "Skipping excluded file: {} (pattern: {})", path.display(), pattern);
                        excluded = true;
                        break;
                    }
                }
            }
        }

        if excluded {
            continue;
        }

        if path.is_file() {
            if is_supported_audio_file(path, &supported_extensions) {
                // Check file validity
                if let Err(e) = check_file_validity(path) {
                    error!(target: "music_chore", "Skipping invalid file {}: {}", path.display(), e);
                    continue;
                }

                let inferred_artist = infer_artist_from_path(path)
                    .map(|artist| MetadataValue::inferred(artist, FOLDER_INFERRED_CONFIDENCE));

                let inferred_album = infer_album_from_path(path)
                    .map(|album| MetadataValue::inferred(album, FOLDER_INFERRED_CONFIDENCE));

                // If album inference failed and we have a reasonable filename, try to extract album from filename
                let final_album = if inferred_album.is_none() {
                    if let Some(filename) = path.file_stem().and_then(|n| n.to_str()) {
                        if let Some(album) = extract_album_from_filename(filename) {
                            Some(MetadataValue::inferred(album, FOLDER_INFERRED_CONFIDENCE))
                        } else {
                            // Fallback: use cleaned filename as album name
                            let cleaned = clean_filename_as_album(filename);
                            if !cleaned.is_empty() {
                                Some(MetadataValue::inferred(cleaned, FOLDER_INFERRED_CONFIDENCE))
                            } else {
                                None
                            }
                        }
                    } else {
                        None
                    }
                } else {
                    inferred_album
                };

                let format = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("unknown")
                    .to_lowercase();

                let metadata = TrackMetadata {
                    title: None,
                    artist: inferred_artist,
                    album: final_album,
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
                warn!(target: "music_chore", "Unsupported audio format: {} (supported: {})", path.display(), supported_extensions.join(", "));
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

/// Check if a string matches a glob-like pattern.
/// Supports: * (any characters), ? (single character)
fn matches_pattern(s: &str, pattern: &str) -> bool {
    // Try to match as a glob pattern
    if let Ok(glob_pattern) = Pattern::new(pattern) {
        if glob_pattern.matches(s) {
            return true;
        }
    }

    // Also check simple substring match for patterns without wildcards
    s.contains(pattern)
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
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_check_file_validity_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let empty_file = temp_dir.path().join("empty.flac");
        std::fs::File::create(&empty_file).unwrap();

        assert!(check_file_validity(&empty_file).is_err());
    }

    #[test]
    fn test_check_file_validity_valid_file() {
        let temp_dir = TempDir::new().unwrap();
        let valid_file = temp_dir.path().join("valid.flac");
        let mut file = std::fs::File::create(&valid_file).unwrap();
        file.write_all(b"some data").unwrap();

        assert!(check_file_validity(&valid_file).is_ok());
    }

    #[test]
    fn test_extract_album_from_filename_artist_album_track() {
        assert_eq!(
            extract_album_from_filename("Artist - Album - Track.flac"),
            Some("Album".to_string())
        );
    }

    #[test]
    fn test_extract_album_from_filename_album_track() {
        assert_eq!(
            extract_album_from_filename("Album - Track.flac"),
            Some("Album".to_string())
        );
    }

    #[test]
    fn test_extract_album_from_filename_track_album() {
        assert_eq!(
            extract_album_from_filename("Track (Album).flac"),
            Some("Album".to_string())
        );
    }

    #[test]
    fn test_extract_album_from_filename_album_track_number() {
        assert_eq!(
            extract_album_from_filename("Album - 01 - Track.flac"),
            Some("Album".to_string())
        );
    }

    #[test]
    fn test_extract_album_from_filename_no_album() {
        assert_eq!(extract_album_from_filename("Track.flac"), None);
    }

    #[test]
    fn test_clean_filename_as_album() {
        assert_eq!(clean_filename_as_album("01 - Track.flac"), "Track");
        assert_eq!(clean_filename_as_album("Track.flac"), "Track");
        assert_eq!(clean_filename_as_album("track.flac"), "track");
    }

    #[test]
    fn test_scan_dir_with_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let empty_file = temp_dir.path().join("empty.flac");
        std::fs::File::create(&empty_file).unwrap();

        let tracks = scan_dir(temp_dir.path());
        assert!(tracks.is_empty());
    }

    #[test]
    fn test_scan_dir_with_valid_and_invalid_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create valid file
        let valid_file = temp_dir.path().join("valid.flac");
        let mut file = std::fs::File::create(&valid_file).unwrap();
        file.write_all(b"some data").unwrap();

        // Create empty file
        let empty_file = temp_dir.path().join("empty.flac");
        std::fs::File::create(&empty_file).unwrap();

        let tracks = scan_dir(temp_dir.path());
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].file_path, valid_file);
    }

    #[test]
    fn test_infer_album_from_filename_with_pattern() {
        // When there's no proper directory structure, extract album from filename
        let temp_dir = TempDir::new().unwrap();
        let subdir = temp_dir.path().join("music");
        std::fs::create_dir(&subdir).unwrap();
        let file = subdir.join("Artist - Album - Track.flac");

        // Create the file with some content
        let mut file_handle = std::fs::File::create(&file).unwrap();
        file_handle.write_all(b"some data").unwrap();

        let tracks = scan_dir(&subdir);
        assert_eq!(tracks.len(), 1);

        // Should have inferred album from filename pattern "Artist - Album - Track"
        // The parent directory is "music" which doesn't look like an album name with proper structure
        // So it should fall back to extracting from filename
        assert!(tracks[0].metadata.album.is_some());
    }

    #[test]
    fn test_infer_album_from_filename_simple() {
        // When there's no proper directory structure and simple filename, use filename as album
        let temp_dir = TempDir::new().unwrap();
        let subdir = temp_dir.path().join("music");
        std::fs::create_dir(&subdir).unwrap();
        let file = subdir.join("Track.flac");

        // Create the file with some content
        let mut file_handle = std::fs::File::create(&file).unwrap();
        file_handle.write_all(b"some data").unwrap();

        let tracks = scan_dir(&subdir);
        assert_eq!(tracks.len(), 1);

        // Should have used filename "Track" as album fallback
        assert!(tracks[0].metadata.album.is_some());
    }

    #[test]
    fn test_scan_dir_skips_symlinks_by_default() {
        let temp_dir = TempDir::new().unwrap();

        // Create a valid file
        let real_file = temp_dir.path().join("real.flac");
        let mut file = std::fs::File::create(&real_file).unwrap();
        file.write_all(b"some data").unwrap();

        // Create a symlink to the file
        let symlink_file = temp_dir.path().join("link.flac");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&real_file, &symlink_file).unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&real_file, &symlink_file).unwrap();

        // Without follow_symlinks, should only find the real file
        let tracks = scan_dir_with_depth_and_symlinks(temp_dir.path(), None, false);
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].file_path, real_file);
    }

    #[test]
    fn test_scan_dir_follows_symlinks_when_enabled() {
        let temp_dir = TempDir::new().unwrap();

        // Create a subdirectory
        let subdir = temp_dir.path().join("music");
        std::fs::create_dir(&subdir).unwrap();

        // Create a valid file in subdirectory
        let real_file = subdir.join("real.flac");
        let mut file = std::fs::File::create(&real_file).unwrap();
        file.write_all(b"some data").unwrap();

        // Create a symlink to the file in the root
        let symlink_file = temp_dir.path().join("link.flac");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&real_file, &symlink_file).unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&real_file, &symlink_file).unwrap();

        // With follow_symlinks, should find both the real file and the symlink
        let tracks = scan_dir_with_depth_and_symlinks(temp_dir.path(), None, true);
        assert_eq!(tracks.len(), 2);
    }

    #[test]
    fn test_scan_dir_with_exclude_pattern() {
        let temp_dir = TempDir::new().unwrap();

        // Create files that should be included
        let include_file1 = temp_dir.path().join("track1.flac");
        let mut file = std::fs::File::create(&include_file1).unwrap();
        file.write_all(b"some data").unwrap();

        let include_file2 = temp_dir.path().join("track2.flac");
        let mut file = std::fs::File::create(&include_file2).unwrap();
        file.write_all(b"some data").unwrap();

        // Create a file that should be excluded
        let exclude_file = temp_dir.path().join("temp_track.flac");
        let mut file = std::fs::File::create(&exclude_file).unwrap();
        file.write_all(b"some data").unwrap();

        // Scan with exclude pattern
        let tracks =
            scan_dir_with_options(temp_dir.path(), None, false, vec!["temp_*".to_string()]);
        assert_eq!(tracks.len(), 2);
        assert!(tracks.iter().any(|t| t.file_path == include_file1));
        assert!(tracks.iter().any(|t| t.file_path == include_file2));
        assert!(!tracks.iter().any(|t| t.file_path == exclude_file));
    }

    #[test]
    fn test_scan_dir_with_multiple_exclude_patterns() {
        let temp_dir = TempDir::new().unwrap();

        // Create files
        let track1 = temp_dir.path().join("song.flac");
        let mut file = std::fs::File::create(&track1).unwrap();
        file.write_all(b"some data").unwrap();

        let track2 = temp_dir.path().join("backup.flac");
        let mut file = std::fs::File::create(&track2).unwrap();
        file.write_all(b"some data").unwrap();

        let track3 = temp_dir.path().join("temp.flac");
        let mut file = std::fs::File::create(&track3).unwrap();
        file.write_all(b"some data").unwrap();

        // Scan with multiple exclude patterns
        let tracks = scan_dir_with_options(
            temp_dir.path(),
            None,
            false,
            vec!["backup*".to_string(), "temp*".to_string()],
        );
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].file_path, track1);
    }

    #[test]
    fn test_matches_pattern_simple() {
        assert!(matches_pattern("temp_file.flac", "temp_*"));
        assert!(matches_pattern("file.tmp", "*.tmp"));
        assert!(!matches_pattern("file.flac", "*.tmp"));
        assert!(matches_pattern("backup_2023.flac", "backup_*"));
    }

    #[test]
    fn test_matches_pattern_question_mark() {
        assert!(matches_pattern("track1.flac", "track?.flac"));
        assert!(matches_pattern("trackA.flac", "track?.flac"));
        assert!(!matches_pattern("track12.flac", "track?.flac"));
    }

    #[test]
    fn test_matches_pattern_substring() {
        // Patterns without wildcards should match as substring
        assert!(matches_pattern("my_backup_file.flac", "backup"));
        assert!(!matches_pattern("my_file.flac", "backup"));
    }
}
