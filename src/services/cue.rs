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

    // Group tracks by their source file
    let mut current_file: Option<String> = None;
    let mut track_index_in_file: u32 = 0;

    for (index, track) in album.tracks.iter().enumerate() {
        let file_name = track
            .file_path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| String::from("unknown.flac"));

        // New file section
        if current_file.as_ref() != Some(&file_name) {
            current_file = Some(file_name.clone());
            track_index_in_file = 0;
            cue_content.push_str(&format!("FILE \"{}\" WAVE\n", file_name));
        }

        let track_number = track
            .metadata
            .track_number
            .as_ref()
            .map(|mv| mv.value)
            .unwrap_or_else(|| (index + 1) as u32);

        cue_content.push_str(&format!("  TRACK {:02} AUDIO\n", track_number));

        // Add TITLE if available
        if let Some(title) = extract_track_title(track) {
            cue_content.push_str(&format!("    TITLE \"{}\"\n", title));
        }

        // Add PERFORMER if available
        if let Some(performer) = extract_track_performer(track) {
            cue_content.push_str(&format!("    PERFORMER \"{}\"\n", performer));
        }

        // Add INDEX 01 for track start
        cue_content.push_str(&format!(
            "    INDEX 01 00:{:02}:00\n",
            track_index_in_file * 2
        ));
        track_index_in_file += 1;
    }

    cue_content
}

/// Generates the default .cue filename for an album.
pub fn generate_cue_file_name(album: &AlbumNode) -> String {
    format!("{}.cue", album.title)
}

/// Writes a .cue file for an album to the specified path.
pub fn write_cue_file(album: &AlbumNode, output_path: &Path) -> Result<(), std::io::Error> {
    let cue_content = generate_cue_content(album);
    std::fs::write(output_path, cue_content)
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
    loop {
        match chars.next() {
            Some('"') => break,
            Some(_) => continue,
            None => return None,
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{AlbumNode, MetadataValue, TrackMetadata, TrackNode};
    use std::path::PathBuf;

    fn create_test_track(title: &str, artist: &str, file_name: &str) -> TrackNode {
        TrackNode {
            file_path: PathBuf::from(file_name),
            metadata: TrackMetadata {
                title: Some(MetadataValue::embedded(title.to_string())),
                artist: Some(MetadataValue::embedded(artist.to_string())),
                album: None,
                album_artist: None,
                track_number: None,
                disc_number: None,
                year: None,
                genre: None,
                duration: None,
                format: "FLAC".to_string(),
                path: PathBuf::from(file_name),
            },
        }
    }

    fn create_test_album(title: &str, year: Option<u32>, tracks: Vec<TrackNode>) -> AlbumNode {
        AlbumNode {
            title: title.to_string(),
            year,
            tracks,
            path: PathBuf::from("/test"),
        }
    }

    #[test]
    fn test_generate_cue_content_basic() {
        let tracks = vec![
            create_test_track("Song One", "Test Artist", "track1.flac"),
            create_test_track("Song Two", "Test Artist", "track2.flac"),
        ];
        let album = create_test_album("Test Album", Some(2024), tracks);

        let content = generate_cue_content(&album);

        assert!(content.contains("PERFORMER \"Test Artist\""));
        assert!(content.contains("TITLE \"Test Album\""));
        assert!(content.contains("REM DATE 2024"));
        assert!(content.contains("FILE \"track1.flac\" WAVE"));
        assert!(content.contains("FILE \"track2.flac\" WAVE"));
        assert!(content.contains("TRACK 01 AUDIO"));
        assert!(content.contains("TRACK 02 AUDIO"));
    }

    #[test]
    fn test_generate_cue_file_name() {
        let album = create_test_album("My Album", None, vec![]);
        let name = generate_cue_file_name(&album);

        assert_eq!(name, "My Album.cue");
    }

    #[test]
    fn test_write_cue_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let cue_path = temp_dir.path().join("Test Album.cue");

        let tracks = vec![create_test_track("Song", "Artist", "file.flac")];
        let album = create_test_album("Test Album", Some(2024), tracks);

        write_cue_file(&album, &cue_path).unwrap();

        assert!(cue_path.exists());

        let content = std::fs::read_to_string(&cue_path).unwrap();
        assert!(content.contains("PERFORMER \"Artist\""));
        assert!(content.contains("TITLE \"Test Album\""));
    }

    #[test]
    fn test_generate_cue_content_single_file_all_tracks() {
        let tracks = vec![
            create_test_track("Track 1", "Artist", "album.flac"),
            create_test_track("Track 2", "Artist", "album.flac"),
            create_test_track("Track 3", "Artist", "album.flac"),
        ];
        let album = create_test_album("Album", None, tracks);

        let content = generate_cue_content(&album);

        let file_count = content.match_indices("FILE ").count();
        assert_eq!(
            file_count, 1,
            "Should have one FILE entry for single file album"
        );
        assert!(content.contains("FILE \"album.flac\" WAVE"));
        assert!(content.contains("TRACK 01 AUDIO"));
        assert!(content.contains("TRACK 02 AUDIO"));
        assert!(content.contains("TRACK 03 AUDIO"));
    }

    #[test]
    fn test_generate_cue_content_multiple_files() {
        let tracks = vec![
            create_test_track("Track 1", "Artist", "01.flac"),
            create_test_track("Track 2", "Artist", "02.flac"),
            create_test_track("Track 3", "Artist", "03.flac"),
        ];
        let album = create_test_album("Album", None, tracks);

        let content = generate_cue_content(&album);

        let file_count = content.match_indices("FILE ").count();
        assert_eq!(file_count, 3, "Should have FILE entry for each track");
        assert!(content.contains("FILE \"01.flac\" WAVE"));
        assert!(content.contains("FILE \"02.flac\" WAVE"));
        assert!(content.contains("FILE \"03.flac\" WAVE"));
    }

    #[test]
    fn test_generate_cue_content_track_timing() {
        let tracks = vec![
            create_test_track("Track 1", "Artist", "file.flac"),
            create_test_track("Track 2", "Artist", "file.flac"),
        ];
        let album = create_test_album("Album", None, tracks);

        let content = generate_cue_content(&album);

        assert!(content.contains("INDEX 01 00:00:00"));
        assert!(content.contains("INDEX 01 00:02:00"));
    }

    #[test]
    fn test_extract_quoted_value() {
        let line = r#"PERFORMER "Test Artist""#;
        let result = extract_quoted_value(line);
        assert_eq!(result, Some("Test Artist".to_string()));

        let line2 = r#"TITLE "Test Album""#;
        let result2 = extract_quoted_value(line2);
        assert_eq!(result2, Some("Test Album".to_string()));
    }

    #[test]
    fn test_extract_quoted_value_no_quotes() {
        let result = extract_quoted_value("TITLE Hello World");
        assert!(result.is_none());
    }
}
