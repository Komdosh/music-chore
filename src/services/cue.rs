//! Cue file generation and parsing services.

use crate::domain::models::{AlbumNode, TrackNode};
use std::path::Path;

fn extract_track_artist(track: &TrackNode) -> Option<&String> {
    track.metadata.artist.as_ref().map(|mv| &mv.value)
}

fn extract_track_title(track: &TrackNode) -> Option<&String> {
    track.metadata.title.as_ref().map(|mv| &mv.value)
}

fn extract_track_performer(track: &TrackNode) -> Option<&String> {
    track
        .metadata
        .album_artist
        .as_ref()
        .map(|mv| &mv.value)
        .or(track.metadata.artist.as_ref().map(|mv| &mv.value))
}

/// Generates a .cue file content for an album based on its track metadata.
pub fn generate_cue_content(album: &AlbumNode) -> String {
    let mut cue_content = String::new();

    // Add PERFORMER (artist) if available from first track
    if let Some(first_track) = album.tracks.first() {
        if let Some(artist) = extract_track_artist(first_track) {
            cue_content.push_str(&format!("PERFORMER \"{}\"\n", artist));
        }
    }

    // Add TITLE (album title)
    cue_content.push_str(&format!("TITLE \"{}\"\n", album.title));

    // Add REM YEAR if available
    if let Some(year) = album.year {
        cue_content.push_str(&format!("REM DATE {}\n", year));
    }

    // Add FILE entry with the first track's file name (assuming all tracks in same file)
    if let Some(first_track) = album.tracks.first() {
        if let Some(file_name) = first_track.file_path.file_name() {
            cue_content.push_str(&format!("FILE \"{}\" WAVE\n", file_name.to_string_lossy()));
        }
    }

    // Add TRACK entries
    for (index, track) in album.tracks.iter().enumerate() {
        let track_number = index + 1;

        cue_content.push_str(&format!("  TRACK {:02} AUDIO\n", track_number));

        // Add TITLE if available
        if let Some(title) = extract_track_title(track) {
            cue_content.push_str(&format!("    TITLE \"{}\"\n", title));
        }

        // Add PERFORMER if available
        if let Some(performer) = extract_track_performer(track) {
            cue_content.push_str(&format!("    PERFORMER \"{}\"\n", performer));
        }

        // Add INDEX 01 for track start (assuming 2 seconds per track for now)
        let start_time = track_number * 2;
        let minutes = start_time / 60;
        let seconds = start_time % 60;
        cue_content.push_str(&format!("    INDEX 01 {:02}:{:02}:00\n", minutes, seconds));
    }

    cue_content
}

/// Writes a .cue file for an album to the specified path.
pub fn write_cue_file(album: &AlbumNode, output_path: &Path) -> Result<(), std::io::Error> {
    let cue_content = generate_cue_content(album);
    std::fs::write(output_path, cue_content)
}

/// Generates a .cue file name based on the album information.
pub fn generate_cue_file_name(album: &AlbumNode) -> String {
    format!("{}.cue", album.title)
}

/// Parses a .cue file and extracts basic information.
pub fn parse_cue_file(cue_path: &Path) -> Result<CueFile, std::io::Error> {
    let content = std::fs::read_to_string(cue_path)?;
    let mut cue_file = CueFile::default();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("PERFORMER") {
            if let Some(value) = extract_quoted_value(line) {
                cue_file.performer = Some(value);
            }
        } else if line.starts_with("TITLE") && !line.starts_with("  TITLE") {
            if let Some(value) = extract_quoted_value(line) {
                cue_file.title = Some(value);
            }
        } else if line.starts_with("FILE") {
            if let Some(value) = extract_quoted_value(line) {
                cue_file.file = Some(value);
            }
        } else if line.starts_with("  TRACK") {
            if let Some(track) = parse_track_line(line) {
                cue_file.tracks.push(track);
            }
        }
    }

    Ok(cue_file)
}

fn extract_quoted_value(line: &str) -> Option<String> {
    let mut chars = line.chars();
    if chars.next()? != '"' {
        return None;
    }
    let mut value = String::new();
    for c in chars {
        if c == '"' {
            break;
        }
        value.push(c);
    }
    Some(value)
}

fn parse_track_line(line: &str) -> Option<CueTrack> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 4 {
        return None;
    }

    let track_number = parts[1].parse::<u32>().ok()?;
    if parts[2] != "AUDIO" {
        return None;
    }

    let mut track = CueTrack::default();
    track.number = track_number;

    Some(track)
}

/// Represents a parsed .cue file.
#[derive(Debug, Default, Clone)]
pub struct CueFile {
    pub performer: Option<String>,
    pub title: Option<String>,
    pub file: Option<String>,
    pub tracks: Vec<CueTrack>,
}

/// Represents a track in a .cue file.
#[derive(Debug, Default, Clone)]
pub struct CueTrack {
    pub number: u32,
    pub title: Option<String>,
    pub performer: Option<String>,
    pub index: Option<String>,
}

/// Validates the consistency of a .cue file with its associated audio files.
pub fn validate_cue_consistency(cue_path: &Path, audio_files: &[&Path]) -> CueValidationResult {
    let mut result = CueValidationResult::default();

    if let Ok(cue_file) = parse_cue_file(cue_path) {
        result.is_valid = true;

        if let Some(file_name) = &cue_file.file {
            let file_found = audio_files.iter().any(|&path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|n| n == file_name)
                    .unwrap_or(false)
            });
            if !file_found {
                result.file_missing = true;
                result.is_valid = false;
            }
        }

        if cue_file.tracks.len() != audio_files.len() {
            result.track_count_mismatch = true;
            result.is_valid = false;
        }
    } else {
        result.is_valid = false;
        result.parsing_error = true;
    }

    result
}

/// Result of .cue file validation.
#[derive(Debug, Default, Clone)]
pub struct CueValidationResult {
    pub is_valid: bool,
    pub parsing_error: bool,
    pub file_missing: bool,
    pub track_count_mismatch: bool,
}
