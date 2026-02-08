//! Enhanced directory scanner with improved error handling and edge cases.
#[allow(unused_imports)]
use crate::adapters::audio_formats as formats;
use crate::core::domain::models::{
    FOLDER_INFERRED_CONFIDENCE, MetadataSource, MetadataValue, Track, TrackMetadata,
};
use crate::core::services::cue::{CueFile, CueTrack, parse_cue_file};
use crate::core::services::inference::{infer_album_from_path, infer_artist_from_path};
// Added Cue imports
use glob::Pattern;
use log::{error, info, warn};
// Removed debug
use crate::adapters::audio_formats::{BasicAudioInfo, read_basic_info};
use serde_json::to_string_pretty;
use std::collections::{BTreeMap, HashSet};
// Added HashSet
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
// Added this import

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
pub fn scan_dir(base: &Path, skip_metadata: bool) -> Vec<Track> {
    scan_dir_with_options_impl(base, None, false, Vec::new(), false, skip_metadata)
}

/// Helper function to determine if a file is a supported audio file based on its extension.
fn is_supported_audio_file(path: &Path, supported_extensions: &HashSet<String>) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map_or(false, |ext| {
            supported_extensions.contains(&ext.to_lowercase())
        })
}

/// Helper function to determine if a file has an audio extension (supported or not).
fn has_audio_extension(path: &Path) -> bool {
    let audio_extensions = ["mp3", "flac", "wav", "dsf", "wv"];
    path.extension()
        .and_then(|ext| ext.to_str())
        .map_or(false, |ext| {
            audio_extensions.contains(&ext.to_lowercase().as_str())
        })
}

/// Helper function to check if a path matches any of the given glob patterns.
fn matches_pattern(path: &Path, patterns: &[String]) -> bool {
    patterns.iter().any(|pattern| {
        Pattern::new(pattern)
            .map(|p| p.matches_path(path))
            .unwrap_or_else(|e| {
                error!(target: "music_chore", "Invalid glob pattern: {} - {}", pattern, e);
                false
            })
    })
}

/// Check if a file is valid (not empty, readable, etc.)
fn check_file_validity(path: &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
    let supported_extensions_vec = formats::get_supported_extensions();
    let supported_extensions: HashSet<String> = supported_extensions_vec.into_iter().collect();

    for entry in WalkDir::new(base)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Skip symlinks to files
        if let Ok(metadata) = path.symlink_metadata() {
            if metadata.file_type().is_symlink() {
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
    let supported_extensions_vec = formats::get_supported_extensions();
    let supported_extensions: HashSet<String> = supported_extensions_vec.into_iter().collect();

    if !base.exists() || !base.is_dir() {
        return paths;
    }

    if let Ok(entries) = std::fs::read_dir(base) {
        for entry in entries.into_iter().flatten() {
            let path = entry.path();

            // Skip symlinks to files
            if let Ok(metadata) = path.symlink_metadata() {
                if metadata.file_type().is_symlink() {
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
pub fn scan_dir_with_metadata(base: &Path) -> Result<Vec<Track>, String> {
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
    let tracks = scan_dir(base, false); // Pass false for skip_metadata
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

/// Helper function to get the track name string for display in `scan` output.
/// It prioritizes CUE-inferred, then embedded, then folder-inferred filename.
/// This function includes the source icon.
pub fn get_track_name_for_scan_output(track: &Track) -> String {
    let mut track_name = track
        .file_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string(); // Default to filename

    if let Some(title_metadata_value) = track.metadata.title.as_ref() {
        if title_metadata_value.source == MetadataSource::CueInferred {
            // For CUE-inferred, use the title value and append filename in parentheses
            track_name = format!(
                "{} ({})",
                title_metadata_value.value,
                track
                    .file_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
            );
        } else if !title_metadata_value.value.is_empty() {
            // For non-CUE-inferred titles, use the title if it exists and is not empty
            track_name = title_metadata_value.value.clone();
        }
    }
    // If no title, or empty title that's not CUE-inferred, track_name remains the filename.

    // Add source icon based on the source of the final track_name
    let source_icon = if let Some(title_metadata_value) = track.metadata.title.as_ref() {
        if title_metadata_value.source == MetadataSource::CueInferred {
            "📄" // Cue-inferred
        } else if title_metadata_value.source == MetadataSource::Embedded {
            "🎯" // Embedded
        } else if title_metadata_value.source == MetadataSource::UserEdited {
            "👤" // User-edited
        } else {
            "🤖" // Folder-inferred (includes inferred from filename)
        }
    } else {
        // If no title metadata, use source from artist or album if available, otherwise default to folder inferred
        let source = track
            .metadata
            .artist
            .as_ref()
            .or(track.metadata.album.as_ref())
            .map(|mv| &mv.source)
            .unwrap_or(&MetadataSource::FolderInferred);

        match source {
            MetadataSource::CueInferred => "📄",
            MetadataSource::Embedded => "🎯",
            MetadataSource::UserEdited => "👤",
            MetadataSource::FolderInferred => "🤖",
        }
    };

    format!("{} {}", track_name, source_icon)
}

/// Scan directory with optional max depth limit.
/// None = unlimited depth (full recursion)
/// Some(0) = immediate directory only (like ls)
/// Some(1) = base dir + 1 level deep
/// Some(2) = base dir + 2 levels deep, etc.
pub fn scan_dir_with_depth(base: &Path, max_depth: Option<usize>) -> Vec<Track> {
    let supported_extensions_vec = formats::get_supported_extensions();
    let supported_extensions: HashSet<String> = supported_extensions_vec.into_iter().collect();

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
                warn!(target: "music_chore", "Unsupported audio format: {} (supported: {})", path.display(), supported_extensions.iter().map(|s| s.as_str()).collect::<Vec<&str>>().join(", "));
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
    let supported_extensions_vec = formats::get_supported_extensions();
    let supported_extensions: HashSet<String> = supported_extensions_vec.into_iter().collect();

    let mut walkdir = WalkDir::new(base).follow_links(follow_symlinks);

    if let Some(depth) = max_depth {
        walkdir = walkdir.max_depth(depth + 1);
    }

    let mut tracks = Vec::new();

    for entry in walkdir.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        // Skip symlinks to files unless follow_symlinks is true
        if let Ok(metadata) = path.symlink_metadata() {
            if metadata.file_type().is_symlink() {
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
                warn!(target: "music_chore", "Unsupported audio format: {} (supported: {})", path.display(), supported_extensions.iter().map(|s| s.as_str()).collect::<Vec<&str>>().join(", "));
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
    skip_metadata: bool,
) -> Vec<Track> {
    scan_dir_with_options_impl(
        base,
        max_depth,
        follow_symlinks,
        exclude_patterns,
        false,
        skip_metadata,
    )
}

/// Scan tracks and return formatted output (text or JSON)
pub fn scan_tracks(path: std::path::PathBuf, json_output: bool) -> Result<String, String> {
    if !path.exists() {
        return Err("No music files found in directory".to_string());
    }

    let tracks = scan_dir(&path, false); // Use scan_dir with skip_metadata=false

    if tracks.is_empty() {
        return Err("No music files found in directory".to_string());
    }

    if json_output {
        match serde_json::to_string_pretty(&tracks) {
            Ok(json) => Ok(json),
            Err(e) => Err(format!("Error serializing to JSON: {}", e)),
        }
    } else {
        let mut output = String::new();
        for track in tracks {
            let track_name_for_display = get_track_name_for_scan_output(&track);
            output.push_str(&format!(
                "{} [{}]\n",
                track.file_path.display(),
                track_name_for_display
            ));
        }
        Ok(output)
    }
}

/// Internal implementation of scan_dir_with_options that supports verbose output
fn scan_dir_with_options_impl(
    base: &Path,
    max_depth: Option<usize>,
    follow_symlinks: bool,
    exclude_patterns: Vec<String>,
    verbose: bool,
    skip_metadata: bool,
) -> Vec<Track> {
    let supported_extensions_vec = formats::get_supported_extensions();
    let supported_extensions: HashSet<String> = supported_extensions_vec.into_iter().collect();

    let mut tracks = Vec::new();
    let mut processed_files = 0;
    let mut supported_files = 0;
    let unsupported_files = 0;
    let mut processed_album_dirs: HashSet<PathBuf> = HashSet::new(); // Track dirs processed by CUE

    // First pass: identify and process CUE files in album directories
    // Only process CUE files if we're NOT skipping metadata
    if !skip_metadata {
        let mut walkdir_first_pass = WalkDir::new(base).follow_links(follow_symlinks);

        if let Some(depth) = max_depth {
            walkdir_first_pass = walkdir_first_pass.max_depth(depth + 1);
        }

        for entry in walkdir_first_pass.into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();

            // Apply exclusion patterns to directories and files
            if matches_pattern(path, &exclude_patterns) {
                if verbose {
                    println!("Skipping excluded path: {}", path.display());
                }

                continue;
            }

            if path.is_dir() {
                // Check for .cue files in this directory
                if let Some(cue_file_path) = std::fs::read_dir(path).ok().and_then(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .find(|e| {
                            e.path()
                                .extension()
                                .map_or(false, |ext| ext.eq_ignore_ascii_case("cue"))
                        })
                        .map(|e| e.path())
                }) {
                    match parse_cue_file(&cue_file_path) {
                        Ok(cue_file) => {
                            let album_dir = path.to_path_buf();
                            let inferred_artist_from_dir =
                                infer_artist_from_path(&album_dir).map(|artist| {
                                    MetadataValue::inferred(artist, FOLDER_INFERRED_CONFIDENCE)
                                });
                            let inferred_album_from_path_from_dir =
                                infer_album_from_path(&album_dir).map(|album| {
                                    MetadataValue::inferred(album, FOLDER_INFERRED_CONFIDENCE)
                                });

                            let cue_performer =
                                cue_file.performer.map(|s| MetadataValue::inferred(s, 1.0));
                            let cue_album = cue_file.title.map(|s| MetadataValue::inferred(s, 1.0));

                            let album =
                                cue_album.or_else(|| inferred_album_from_path_from_dir.clone());

                            for cue_track in cue_file.tracks {
                                if let Some(audio_file_name) = cue_track.file {
                                    let audio_file_path = album_dir.join(&audio_file_name);

                                    // Get basic info (duration, format) from the actual audio file
                                    let basic_info_metadata = match read_basic_info(
                                        &audio_file_path,
                                    ) {
                                        Ok(m) => Some(m),
                                        Err(e) => {
                                            if verbose {
                                                println!(
                                                    "Could not read basic info for CUE-referenced file {}: {}",
                                                    audio_file_path.display(),
                                                    e
                                                );
                                            }
                                            None
                                        }
                                    };

                                    let title = cue_track
                                        .title
                                        .map(|s| MetadataValue::cue_inferred(s, 1.0)); // CUE title is high confidence
                                    let artist = cue_track
                                        .performer
                                        .map(|s| MetadataValue::cue_inferred(s, 1.0)); // CUE performer is high confidence
                                    let track_number =
                                        Some(MetadataValue::cue_inferred(cue_track.number, 1.0));
                                    let artist = artist
                                        .or_else(|| cue_performer.clone())
                                        .or_else(|| inferred_artist_from_dir.clone());
                                    let metadata = TrackMetadata {
                                        title,
                                        artist, // Fallback to folder inferred artist
                                        album: album.clone(),
                                        album_artist: cue_performer.clone(),
                                        track_number,
                                        disc_number: None, // CUE files typically don't specify disc number per track
                                        year: cue_file
                                            .date
                                            .clone()
                                            .and_then(|s| s.parse::<u32>().ok())
                                            .map(|y| MetadataValue::cue_inferred(y, 1.0)),
                                        genre: cue_file
                                            .genre
                                            .clone()
                                            .map(|s| MetadataValue::cue_inferred(s, 1.0)),
                                        duration: basic_info_metadata
                                            .as_ref()
                                            .and_then(|m| m.duration.clone()),
                                        format: basic_info_metadata
                                            .map_or("unknown".to_string(), |m| m.format),
                                        path: audio_file_path.clone(),
                                    };
                                    tracks.push(Track::new(audio_file_path.clone(), metadata));
                                } else {
                                    if verbose {
                                        println!(
                                            "CUE track {} in {} has no associated audio file.",
                                            cue_track.number,
                                            cue_file_path.display()
                                        );
                                    }
                                }
                            }
                            processed_album_dirs.insert(album_dir);
                        }
                        Err(e) => {
                            error!(target: "music_chore", "Failed to parse CUE file {}: {}", cue_file_path.display(), e);
                        }
                    }
                }
            }
        }
    }

    // Second pass: process individual files, skipping those in CUE-handled directories
    let mut walkdir_second_pass = WalkDir::new(base).follow_links(follow_symlinks);
    if let Some(depth) = max_depth {
        walkdir_second_pass = walkdir_second_pass.max_depth(depth + 1);
    }

    for entry in walkdir_second_pass.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        // Apply exclusion patterns to directories and files
        if matches_pattern(path, &exclude_patterns) {
            if verbose {
                println!("Skipping excluded path: {}", path.display());
            }
            continue;
        }

        if path.is_file() {
            // If the parent directory was processed by a CUE file, skip this file
            if let Some(parent_dir) = path.parent() {
                if processed_album_dirs.contains(parent_dir) {
                    if verbose {
                        println!(
                            "Skipping file {} as its parent directory was processed by a CUE file.",
                            path.display()
                        );
                    }
                    continue;
                }
            }

            processed_files += 1;

            if verbose && processed_files % 100 == 0 {
                println!("Processed {} files...", processed_files);
            }

            if is_supported_audio_file(path, &supported_extensions) {
                supported_files += 1;

                // Check basic file validity
                if let Err(e) = check_file_validity(path) {
                    error!(target: "music_chore", "Skipping invalid file {}: {}", path.display(), e);
                    continue;
                }

                // Attempt to read embedded metadata first
                let metadata = if skip_metadata {
                    // If skipping metadata, populate with filename and folder inference only
                    let inferred_artist = infer_artist_from_path(path)
                        .map(|artist| MetadataValue::inferred(artist, FOLDER_INFERRED_CONFIDENCE));

                    let inferred_album = infer_album_from_path(path)
                        .map(|album| MetadataValue::inferred(album, FOLDER_INFERRED_CONFIDENCE));

                    let final_album = if inferred_album.is_none() {
                        if let Some(filename) = path.file_stem().and_then(|n| n.to_str()) {
                            if let Some(album) = extract_album_from_filename(filename) {
                                Some(MetadataValue::inferred(album, FOLDER_INFERRED_CONFIDENCE))
                            } else {
                                let cleaned = clean_filename_as_album(filename);
                                if !cleaned.is_empty() {
                                    Some(MetadataValue::inferred(
                                        cleaned,
                                        FOLDER_INFERRED_CONFIDENCE,
                                    ))
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

                    // Extract title from filename when skipping metadata
                    let title_from_filename =
                        if let Some(filename) = path.file_stem().and_then(|n| n.to_str()) {
                            // When skipping metadata, use the full filename (without extension) as the title
                            Some(MetadataValue::inferred(
                                filename.to_string(),
                                FOLDER_INFERRED_CONFIDENCE,
                            ))
                        } else {
                            None
                        };

                    TrackMetadata {
                        title: title_from_filename, // No title from embedded metadata, but use filename as title
                        artist: inferred_artist,
                        album: final_album,
                        album_artist: None,
                        track_number: None,
                        disc_number: None,
                        year: None,
                        genre: None,
                        duration: None,
                        format: path
                            .extension()
                            .and_then(|ext| ext.to_str())
                            .unwrap_or("unknown")
                            .to_lowercase(),
                        path: path.to_path_buf(),
                    }
                } else {
                    // Existing logic: Attempt to read embedded metadata, then infer
                    let mut full_embedded_track_info: Option<Track> = None;
                    if let Ok(track) = formats::read_metadata(path) {
                        full_embedded_track_info = Some(track);
                    } else {
                        if verbose {
                            println!(
                                "Failed to read embedded metadata for file {}. Attempting to infer metadata from filename...",
                                path.display()
                            );
                        }
                    }

                    let mut metadata = TrackMetadata {
                        title: full_embedded_track_info
                            .as_ref()
                            .and_then(|t| t.metadata.title.clone()),
                        artist: full_embedded_track_info
                            .as_ref()
                            .and_then(|t| t.metadata.artist.clone()),
                        album: full_embedded_track_info
                            .as_ref()
                            .and_then(|t| t.metadata.album.clone()),
                        album_artist: full_embedded_track_info
                            .as_ref()
                            .and_then(|t| t.metadata.album_artist.clone()),
                        track_number: full_embedded_track_info
                            .as_ref()
                            .and_then(|t| t.metadata.track_number.clone()),
                        disc_number: full_embedded_track_info
                            .as_ref()
                            .and_then(|t| t.metadata.disc_number.clone()),
                        year: full_embedded_track_info
                            .as_ref()
                            .and_then(|t| t.metadata.year.clone()),
                        genre: full_embedded_track_info
                            .as_ref()
                            .and_then(|t| t.metadata.genre.clone()),
                        duration: full_embedded_track_info
                            .as_ref()
                            .and_then(|t| t.metadata.duration.clone()),
                        format: path
                            .extension()
                            .and_then(|ext| ext.to_str())
                            .unwrap_or("unknown")
                            .to_lowercase(),
                        path: path.to_path_buf(),
                    };

                    // Apply folder inference only if corresponding embedded metadata is missing
                    if metadata.artist.is_none() {
                        metadata.artist = infer_artist_from_path(path).map(|artist| {
                            MetadataValue::inferred(artist, FOLDER_INFERRED_CONFIDENCE)
                        });
                    }

                    // Album inference: prioritize embedded, then folder-inferred from path, then from filename
                    if metadata.album.is_none() {
                        let inferred_album_from_path = infer_album_from_path(path).map(|album| {
                            MetadataValue::inferred(album, FOLDER_INFERRED_CONFIDENCE)
                        });

                        metadata.album = if inferred_album_from_path.is_some() {
                            inferred_album_from_path
                        } else if let Some(filename) = path.file_stem().and_then(|n| n.to_str()) {
                            if let Some(album) = extract_album_from_filename(filename) {
                                Some(MetadataValue::inferred(album, FOLDER_INFERRED_CONFIDENCE))
                            } else {
                                let cleaned = clean_filename_as_album(filename);
                                if !cleaned.is_empty() {
                                    Some(MetadataValue::inferred(
                                        cleaned,
                                        FOLDER_INFERRED_CONFIDENCE,
                                    ))
                                } else {
                                    None
                                }
                            }
                        } else {
                            None
                        };
                    }
                    metadata
                };

                let track = Track::new(path.to_path_buf(), metadata);
                tracks.push(track);
            } else if has_audio_extension(path) {
                if verbose {
                    println!(
                        "Unsupported audio format: {} (supported: {})",
                        path.display(),
                        supported_extensions
                            .iter()
                            .map(|s| s.as_str())
                            .collect::<Vec<&str>>()
                            .join(", ")
                    );
                }
            }
        }
    }

    if verbose {
        println!(
            "Scan completed: {} processed, {} supported, {} unsupported",
            processed_files, supported_files, unsupported_files
        );
    }

    tracks.sort_by(|a, b| {
        let file_a = a.file_path.file_name().unwrap_or_default();
        let file_b = b.file_path.file_name().unwrap_or_default();
        file_a.cmp(&file_b)
    });

    tracks
}
