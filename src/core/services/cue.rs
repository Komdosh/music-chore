//! Cue file generation and parsing services.

use crate::core::domain::models::{AlbumNode, MetadataValue, TrackNode};
use crate::core::services::normalization::to_title_case;
use std::path::{Path, PathBuf};

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

fn get_track_genre(track: &TrackNode) -> Option<&String> {
    track.metadata.genre.as_ref().map(|mv| &mv.value)
}

fn get_track_album(track: &TrackNode) -> Option<&String> {
    track.metadata.album.as_ref().map(|mv| &mv.value)
}

fn get_track_album_artist(track: &TrackNode) -> Option<&String> {
    track.metadata.album_artist.as_ref().map(|mv| &mv.value)
}

fn confidence_is_embedded(mv: &MetadataValue<String>) -> bool {
    matches!(mv.source, crate::core::domain::models::MetadataSource::Embedded)
}

fn year_confidence_is_embedded(mv: &MetadataValue<u32>) -> bool {
    matches!(mv.source, crate::core::domain::models::MetadataSource::Embedded)
}

fn get_highest_confidence_value(
    tracks: &[TrackNode],
    extractor: fn(&TrackNode) -> Option<&String>,
) -> Option<String> {
    let mut best_value: Option<(String, bool, f32)> = None;

    for track in tracks {
        if let Some(value) = extractor(track) {
            let is_embedded = track
                .metadata
                .album
                .as_ref()
                .filter(|mv| confidence_is_embedded(mv))
                .is_some()
                || track
                    .metadata
                    .artist
                    .as_ref()
                    .filter(|mv| confidence_is_embedded(mv))
                    .is_some()
                || track
                    .metadata
                    .genre
                    .as_ref()
                    .filter(|mv| confidence_is_embedded(mv))
                    .is_some();

            let confidence = track
                .metadata
                .album
                .as_ref()
                .map(|mv| mv.confidence)
                .or_else(|| track.metadata.artist.as_ref().map(|mv| mv.confidence))
                .or_else(|| track.metadata.genre.as_ref().map(|mv| mv.confidence))
                .unwrap_or(0.0);

            let is_embedded_for_genre = track
                .metadata
                .genre
                .as_ref()
                .filter(|mv| confidence_is_embedded(mv))
                .is_some();

            let genre_confidence = track
                .metadata
                .genre
                .as_ref()
                .map(|mv| mv.confidence)
                .unwrap_or(0.0);

            let final_is_embedded = is_embedded || is_embedded_for_genre;
            let final_confidence = if is_embedded_for_genre {
                genre_confidence
            } else {
                confidence
            };

            match &best_value {
                Some((_, existing_is_embedded, existing_confidence)) => {
                    if (final_is_embedded && !existing_is_embedded) ||
                       (final_is_embedded == *existing_is_embedded && final_confidence > *existing_confidence) {
                        best_value = Some((value.clone(), final_is_embedded, final_confidence));
                    }
                }
                None => {
                    best_value = Some((value.clone(), final_is_embedded, final_confidence));
                }
            }
        }
    }

    best_value.map(|(v, _, _)| v)
}

fn get_embedded_year(tracks: &[TrackNode]) -> Option<u32> {
    for track in tracks {
        if let Some(year) = &track.metadata.year
            && year_confidence_is_embedded(year)
        {
            return Some(year.value);
        }
    }
    None
}

fn get_highest_confidence_year(tracks: &[TrackNode]) -> Option<u32> {
    let mut best_year: Option<(u32, bool, f32)> = None;

    for track in tracks {
        if let Some(year) = &track.metadata.year {
            let is_embedded = year_confidence_is_embedded(year);
            let confidence = year.confidence;

            match &best_year {
                Some((_, existing_is_embedded, existing_confidence)) => {
                    if (is_embedded && !existing_is_embedded) ||
                       (is_embedded == *existing_is_embedded && confidence > *existing_confidence) {
                        best_year = Some((year.value, is_embedded, confidence));
                    }
                }
                None => {
                    best_year = Some((year.value, is_embedded, confidence));
                }
            }
        }
    }

    best_year.map(|(v, _, _)| v)
}

/// Generates a .cue file content for an album based on its track metadata.
/// When tracks contain conflicting metadata (different artists, years, or genres),
/// embedded metadata (confidence=1.0) takes precedence over folder-inferred metadata.
/// If multiple embedded values exist, the most common value is selected.
/// All text fields (artist, album, genre) are normalized to title case.
pub fn generate_cue_content(album: &AlbumNode) -> String {
    let mut cue_content = String::new();

    let tracks = &album.tracks;

    let artist = get_highest_confidence_value(tracks, get_track_album_artist)
        .or_else(|| get_highest_confidence_value(tracks, extract_track_artist));

    if let Some(artist_name) = artist {
        cue_content.push_str(&format!("PERFORMER \"{}\"\n", to_title_case(&artist_name)));
    }

    let album_title = get_highest_confidence_value(tracks, get_track_album)
        .unwrap_or_else(|| album.title.clone());

    cue_content.push_str(&format!("TITLE \"{}\"\n", to_title_case(&album_title)));

    let genre = get_highest_confidence_value(tracks, get_track_genre);
    if let Some(genre_value) = genre {
        cue_content.push_str(&format!("REM GENRE {}\n", to_title_case(&genre_value)));
    }

    let year = get_embedded_year(tracks)
        .or(album.year)
        .or_else(|| get_highest_confidence_year(tracks));
    if let Some(year_value) = year {
        cue_content.push_str(&format!("REM DATE {}\n", year_value));
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

        // Add PERFORMER if available (normalized to title case)
        if let Some(performer) = extract_track_performer(track) {
            cue_content.push_str(&format!("    PERFORMER \"{}\"\n", to_title_case(performer)));
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

use crate::adapters::audio_formats::read_metadata;
use crate::core::services::scanner::scan_dir_immediate;

pub struct CueGenerationResult {
    pub cue_content: String,
    pub output_path: PathBuf,
    pub tracks_count: usize,
}

pub enum CueGenerationError {
    NoMusicFiles,
    NoReadableFiles,
    FileReadError(String),
}

pub fn generate_cue_for_path(
    path: &Path,
    output: Option<PathBuf>,
) -> Result<CueGenerationResult, CueGenerationError> {
    let file_paths = scan_dir_immediate(path);
    if file_paths.is_empty() {
        return Err(CueGenerationError::NoMusicFiles);
    }

    let mut tracks = Vec::new();
    for file_path in &file_paths {
        match read_metadata(file_path) {
            Ok(track) => tracks.push(track),
            Err(e) => {
                return Err(CueGenerationError::FileReadError(format!(
                    "Failed to read {}: {}",
                    file_path.display(),
                    e
                )));
            }
        }
    }

    if tracks.is_empty() {
        return Err(CueGenerationError::NoReadableFiles);
    }

    let album_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Unknown Album".to_string());

    let tracks_count = tracks.len();

    let track_nodes: Vec<TrackNode> = tracks
        .into_iter()
        .map(|track| TrackNode {
            file_path: track.file_path,
            metadata: track.metadata,
        })
        .collect();

    let album = AlbumNode {
        title: album_name.clone(),
        year: None,
        tracks: track_nodes,
        path: path.to_path_buf(),
    };

    let cue_file_name = generate_cue_file_name(&album);
    let output_path = output.unwrap_or_else(|| path.join(cue_file_name));

    let cue_content = generate_cue_content(&album);

    Ok(CueGenerationResult {
        cue_content,
        output_path,
        tracks_count,
    })
}

/// Parses a .cue file and extracts basic information.
pub fn parse_cue_file(cue_path: &Path) -> Result<CueFile, std::io::Error> {
    let content = std::fs::read_to_string(cue_path)?;
    let mut cue_file = CueFile::default();
    let mut current_track: Option<CueTrack> = None;
    let mut current_file: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        let is_track_level = line.starts_with("  ") || line.starts_with("\t");

        if trimmed.starts_with("PERFORMER") && !is_track_level {
            if let Some(value) = extract_quoted_value(trimmed) {
                cue_file.performer = Some(value);
            }
        } else if trimmed.starts_with("TITLE") && !is_track_level {
            if let Some(value) = extract_quoted_value(trimmed) {
                cue_file.title = Some(value);
            }
        } else if trimmed.starts_with("REM GENRE") {
            if let Some(value) = extract_quoted_value(trimmed) {
                cue_file.genre = Some(value);
            } else {
                let value = trimmed.trim_start_matches("REM GENRE").trim();
                if !value.is_empty() {
                    cue_file.genre = Some(value.to_string());
                }
            }
        } else if trimmed.starts_with("REM DATE") {
            let value = trimmed.trim_start_matches("REM DATE").trim();
            if !value.is_empty() {
                cue_file.date = Some(value.to_string());
            }
        } else if trimmed.starts_with("FILE") {
            if let Some(value) = extract_quoted_value(trimmed) {
                current_file = Some(value.clone());
                cue_file.files.push(value);
            }
        } else if trimmed.starts_with("TRACK") && is_track_level {
            if let Some(track) = parse_track_line(trimmed) {
                if let Some(prev_track) = current_track.take() {
                    cue_file.tracks.push(prev_track);
                }
                let mut new_track = track;
                new_track.file = current_file.clone();
                current_track = Some(new_track);
            }
        } else if trimmed.starts_with("TITLE") && is_track_level {
            if let Some(value) = extract_quoted_value(trimmed) {
                if let Some(ref mut track) = current_track {
                    track.title = Some(value);
                }
            }
        } else if trimmed.starts_with("PERFORMER") && is_track_level {
            if let Some(value) = extract_quoted_value(trimmed) {
                if let Some(ref mut track) = current_track {
                    track.performer = Some(value);
                }
            }
        } else if trimmed.starts_with("INDEX") && is_track_level {
            if let Some(value) = extract_quoted_value(trimmed) {
                if let Some(ref mut track) = current_track {
                    track.index = Some(value);
                }
            }
        }
    }

    if let Some(track) = current_track {
        cue_file.tracks.push(track);
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
    if parts.len() < 3 {
        return None;
    }

    let track_number = parts[1].parse::<u32>().ok()?;
    if parts[2] != "AUDIO" {
        return None;
    }

    Some(CueTrack {
        number: track_number,
        ..Default::default()
    })
}

/// Represents a parsed .cue file.
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct CueFile {
    pub performer: Option<String>,
    pub title: Option<String>,
    pub genre: Option<String>,
    pub date: Option<String>,
    pub files: Vec<String>,
    pub tracks: Vec<CueTrack>,
}

/// Represents a track in a .cue file.
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct CueTrack {
    pub number: u32,
    pub title: Option<String>,
    pub performer: Option<String>,
    pub index: Option<String>,
    pub file: Option<String>,
}

/// Validates the consistency of a .cue file with its associated audio files.
pub fn validate_cue_consistency(cue_path: &Path, audio_files: &[&Path]) -> CueValidationResult {
    let mut result = CueValidationResult::default();

    if let Ok(cue_file) = parse_cue_file(cue_path) {
        result.is_valid = true;

        for file_name in &cue_file.files {
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
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct CueValidationResult {
    pub is_valid: bool,
    pub parsing_error: bool,
    pub file_missing: bool,
    pub track_count_mismatch: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use crate::TrackMetadata;

    fn create_test_track(
        title: &str,
        artist: &str,
        file_name: &str,
        year: Option<u32>,
        genre: Option<&str>,
    ) -> TrackNode {
        TrackNode {
            file_path: PathBuf::from(file_name),
            metadata: TrackMetadata {
                title: Some(MetadataValue::embedded(title.to_string())),
                artist: Some(MetadataValue::embedded(artist.to_string())),
                album: None,
                album_artist: None,
                track_number: None,
                disc_number: None,
                year: year.map(MetadataValue::embedded),
                genre: genre.map(|g| MetadataValue::embedded(g.to_string())),
                duration: None,
                format: "FLAC".to_string(),
                path: PathBuf::from(file_name),
            },
        }
    }

    fn create_test_album_with_genre(
        title: &str,
        year: Option<u32>,
        _genre: Option<&str>,
        tracks: Vec<TrackNode>,
    ) -> AlbumNode {
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
            create_test_track(
                "Song One",
                "Test Artist",
                "track1.flac",
                Some(2024),
                Some("Rock"),
            ),
            create_test_track(
                "Song Two",
                "Test Artist",
                "track2.flac",
                Some(2024),
                Some("Rock"),
            ),
        ];
        let album = create_test_album_with_genre("Test Album", Some(2024), Some("Rock"), tracks);

        let content = generate_cue_content(&album);

        assert!(content.contains("PERFORMER \"Test Artist\""));
        assert!(content.contains("TITLE \"Test Album\""));
        assert!(content.contains("REM DATE 2024"));
        assert!(content.contains("REM GENRE Rock"));
        assert!(content.contains("FILE \"track1.flac\" WAVE"));
        assert!(content.contains("FILE \"track2.flac\" WAVE"));
        assert!(content.contains("TRACK 01 AUDIO"));
        assert!(content.contains("TRACK 02 AUDIO"));
    }

    #[test]
    fn test_generate_cue_file_name() {
        let album = create_test_album_with_genre("My Album", None, None, vec![]);
        let name = generate_cue_file_name(&album);

        assert_eq!(name, "My Album.cue");
    }

    #[test]
    fn test_write_cue_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let cue_path = temp_dir.path().join("Test Album.cue");

        let tracks = vec![create_test_track(
            "Song",
            "Artist",
            "file.flac",
            None,
            Some("Rock"),
        )];
        let album = create_test_album_with_genre("Test Album", Some(2024), Some("Rock"), tracks);

        write_cue_file(&album, &cue_path).unwrap();

        assert!(cue_path.exists());

        let content = std::fs::read_to_string(&cue_path).unwrap();
        assert!(content.contains("PERFORMER \"Artist\""));
        assert!(content.contains("TITLE \"Test Album\""));
        assert!(content.contains("REM GENRE Rock"));
    }

    #[test]
    fn test_generate_cue_content_single_file_all_tracks() {
        let tracks = vec![
            create_test_track("Track 1", "Artist", "album.flac", Some(2023), Some("Jazz")),
            create_test_track("Track 2", "Artist", "album.flac", Some(2023), Some("Jazz")),
            create_test_track("Track 3", "Artist", "album.flac", Some(2023), Some("Jazz")),
        ];
        let album = create_test_album_with_genre("Album", Some(2023), Some("Jazz"), tracks);

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
            create_test_track("Track 1", "Artist", "01.flac", None, Some("Electronic")),
            create_test_track("Track 2", "Artist", "02.flac", None, Some("Electronic")),
            create_test_track("Track 3", "Artist", "03.flac", None, Some("Electronic")),
        ];
        let album = create_test_album_with_genre("Album", None, Some("Electronic"), tracks);

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
            create_test_track("Track 1", "Artist", "file.flac", None, None),
            create_test_track("Track 2", "Artist", "file.flac", None, None),
        ];
        let album = create_test_album_with_genre("Album", None, None, tracks);

        let content = generate_cue_content(&album);

        assert!(content.contains("INDEX 01 00:00:00"));
        assert!(content.contains("INDEX 01 00:02:00"));
    }

    #[test]
    fn test_generate_cue_content_genre_from_track() {
        let tracks = vec![
            create_test_track(
                "Song One",
                "Artist",
                "track1.flac",
                Some(2020),
                Some("Classical"),
            ),
            create_test_track(
                "Song Two",
                "Artist",
                "track2.flac",
                Some(2020),
                Some("Classical"),
            ),
        ];
        let album =
            create_test_album_with_genre("Album Title", Some(2020), Some("Classical"), tracks);

        let content = generate_cue_content(&album);

        assert!(content.contains("REM GENRE Classical"));
    }

    #[test]
    fn test_generate_cue_content_year_from_track() {
        let tracks = vec![
            create_test_track("Song One", "Artist", "track1.flac", Some(2019), Some("Pop")),
            create_test_track("Song Two", "Artist", "track2.flac", Some(2019), Some("Pop")),
        ];
        let album = create_test_album_with_genre("Album", None, Some("Pop"), tracks);

        let content = generate_cue_content(&album);

        assert!(content.contains("REM DATE 2019"));
    }

    #[test]
    fn test_generate_cue_content_no_genre() {
        let tracks = vec![
            create_test_track("Song One", "Artist", "track1.flac", None, None),
            create_test_track("Song Two", "Artist", "track2.flac", None, None),
        ];
        let album = create_test_album_with_genre("Album", None, None, tracks);

        let content = generate_cue_content(&album);

        assert!(!content.contains("REM GENRE"));
    }

    #[test]
    fn test_generate_cue_content_album_artist_takes_precedence() {
        let tracks = vec![
            TrackNode {
                file_path: PathBuf::from("track1.flac"),
                metadata: TrackMetadata {
                    title: Some(MetadataValue::embedded("Song One".to_string())),
                    artist: Some(MetadataValue::embedded("Track Artist".to_string())),
                    album: None,
                    album_artist: Some(MetadataValue::embedded("Album Artist".to_string())),
                    track_number: None,
                    disc_number: None,
                    year: None,
                    genre: None,
                    duration: None,
                    format: "FLAC".to_string(),
                    path: PathBuf::from("track1.flac"),
                },
            },
            TrackNode {
                file_path: PathBuf::from("track2.flac"),
                metadata: TrackMetadata {
                    title: Some(MetadataValue::embedded("Song Two".to_string())),
                    artist: Some(MetadataValue::embedded("Track Artist".to_string())),
                    album: None,
                    album_artist: Some(MetadataValue::embedded("Album Artist".to_string())),
                    track_number: None,
                    disc_number: None,
                    year: None,
                    genre: None,
                    duration: None,
                    format: "FLAC".to_string(),
                    path: PathBuf::from("track2.flac"),
                },
            },
        ];
        let album = create_test_album_with_genre("Album", None, None, tracks);

        let content = generate_cue_content(&album);

        assert!(content.contains("PERFORMER \"Album Artist\""));
    }

    #[test]
    fn test_generate_cue_content_album_title_from_track_metadata() {
        let tracks = vec![
            TrackNode {
                file_path: PathBuf::from("track1.flac"),
                metadata: TrackMetadata {
                    title: Some(MetadataValue::embedded("Song One".to_string())),
                    artist: Some(MetadataValue::embedded("Artist".to_string())),
                    album: Some(MetadataValue::embedded("Real Album Name".to_string())),
                    album_artist: None,
                    track_number: None,
                    disc_number: None,
                    year: None,
                    genre: None,
                    duration: None,
                    format: "FLAC".to_string(),
                    path: PathBuf::from("track1.flac"),
                },
            },
            TrackNode {
                file_path: PathBuf::from("track2.flac"),
                metadata: TrackMetadata {
                    title: Some(MetadataValue::embedded("Song Two".to_string())),
                    artist: Some(MetadataValue::embedded("Artist".to_string())),
                    album: Some(MetadataValue::embedded("Real Album Name".to_string())),
                    album_artist: None,
                    track_number: None,
                    disc_number: None,
                    year: None,
                    genre: None,
                    duration: None,
                    format: "FLAC".to_string(),
                    path: PathBuf::from("track2.flac"),
                },
            },
        ];
        let album = create_test_album_with_genre("Folder Name", None, None, tracks);

        let content = generate_cue_content(&album);

        assert!(content.contains("TITLE \"Real Album Name\""));
    }

    #[test]
    fn test_generate_cue_content_conflicting_metadata_uses_embedded() {
        let tracks = vec![
            TrackNode {
                file_path: PathBuf::from("track1.flac"),
                metadata: TrackMetadata {
                    title: Some(MetadataValue::embedded("Song One".to_string())),
                    artist: Some(MetadataValue::embedded("Artist".to_string())),
                    album: Some(MetadataValue::embedded("Album From Tags".to_string())),
                    album_artist: None,
                    track_number: None,
                    disc_number: None,
                    year: Some(MetadataValue::embedded(2021)),
                    genre: Some(MetadataValue::embedded("Metal".to_string())),
                    duration: None,
                    format: "FLAC".to_string(),
                    path: PathBuf::from("track1.flac"),
                },
            },
            TrackNode {
                file_path: PathBuf::from("track2.flac"),
                metadata: TrackMetadata {
                    title: Some(MetadataValue::embedded("Song Two".to_string())),
                    artist: Some(MetadataValue::embedded("Artist".to_string())),
                    album: Some(MetadataValue::inferred("Folder Album".to_string(), 0.3)),
                    album_artist: None,
                    track_number: None,
                    disc_number: None,
                    year: Some(MetadataValue::inferred(2020, 0.3)),
                    genre: Some(MetadataValue::inferred("Rock".to_string(), 0.3)),
                    duration: None,
                    format: "FLAC".to_string(),
                    path: PathBuf::from("track2.flac"),
                },
            },
        ];
        let album = create_test_album_with_genre("Folder Album", None, Some("Rock"), tracks);

        let content = generate_cue_content(&album);

        assert!(content.contains("TITLE \"Album From Tags\""));
        assert!(content.contains("REM DATE 2021"));
        assert!(content.contains("REM GENRE Metal"));
    }

    #[test]
    fn test_generate_cue_content_title_case_normalization() {
        let tracks = vec![create_test_track(
            "Song",
            "UPPERCASE ARTIST",
            "track1.flac",
            Some(2020),
            Some("ROCK"),
        )];
        let album =
            create_test_album_with_genre("UPPERCASE ALBUM", Some(2020), Some("ROCK"), tracks);

        let content = generate_cue_content(&album);

        assert!(content.contains("PERFORMER \"Uppercase Artist\""));
        assert!(content.contains("TITLE \"Uppercase Album\""));
        assert!(content.contains("REM GENRE Rock"));
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

    #[test]
    fn test_parse_cue_file_basic() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let cue_path = temp_dir.path().join("test.cue");

        let cue_content = r#"PERFORMER "Test Artist"
TITLE "Test Album"
FILE "test.flac" WAVE
  TRACK 01 AUDIO
    TITLE "Track One"
    PERFORMER "Track Artist"
    INDEX 01 00:00:00
  TRACK 02 AUDIO
    TITLE "Track Two"
    INDEX 01 00:03:00
"#;
        std::fs::write(&cue_path, cue_content).unwrap();

        let result = parse_cue_file(&cue_path).unwrap();

        assert_eq!(
            result.performer,
            Some("Test Artist".to_string()),
            "Album performer should be 'Test Artist'"
        );
        assert_eq!(
            result.title,
            Some("Test Album".to_string()),
            "Album title should be 'Test Album'"
        );
        assert!(
            result.genre.is_none(),
            "Genre should be None for basic test"
        );
        assert!(result.date.is_none(), "Date should be None for basic test");
        assert_eq!(result.files, vec!["test.flac".to_string()]);
        assert_eq!(result.tracks.len(), 2);
        assert_eq!(result.tracks[0].number, 1);
        assert_eq!(result.tracks[0].title, Some("Track One".to_string()));
        assert_eq!(result.tracks[0].performer, Some("Track Artist".to_string()));
        assert_eq!(result.tracks[0].file, Some("test.flac".to_string()));
        assert_eq!(result.tracks[1].number, 2);
        assert_eq!(result.tracks[1].title, Some("Track Two".to_string()));
        assert_eq!(result.tracks[1].file, Some("test.flac".to_string()));
    }

    #[test]
    fn test_parse_cue_file_minimal() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let cue_path = temp_dir.path().join("minimal.cue");

        let cue_content = r#"PERFORMER "Artist"
TITLE "Album"
FILE "tracks.flac" WAVE
  TRACK 01 AUDIO
    TITLE "First Track"
    INDEX 01 00:00:00
"#;
        std::fs::write(&cue_path, cue_content).unwrap();

        let result = parse_cue_file(&cue_path).unwrap();

        assert_eq!(result.performer, Some("Artist".to_string()));
        assert_eq!(result.title, Some("Album".to_string()));
        assert_eq!(result.files, vec!["tracks.flac".to_string()]);
        assert_eq!(result.tracks.len(), 1);
    }

    #[test]
    fn test_parse_cue_file_multiple_files() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let cue_path = temp_dir.path().join("multi.cue");

        let cue_content = r#"PERFORMER "Various Artists"
TITLE "Compilation"
FILE "disc1.flac" WAVE
  TRACK 01 AUDIO
    TITLE "Track 1"
    INDEX 01 00:00:00
FILE "disc2.flac" WAVE
  TRACK 02 AUDIO
    TITLE "Track 2"
    INDEX 01 00:00:00
"#;
        std::fs::write(&cue_path, cue_content).unwrap();

        let result = parse_cue_file(&cue_path).unwrap();

        assert_eq!(result.performer, Some("Various Artists".to_string()));
        assert_eq!(
            result.files,
            vec!["disc1.flac".to_string(), "disc2.flac".to_string()]
        );
        assert_eq!(result.tracks.len(), 2);
        assert_eq!(result.tracks[0].number, 1);
        assert_eq!(result.tracks[0].file, Some("disc1.flac".to_string()));
        assert_eq!(result.tracks[1].number, 2);
        assert_eq!(result.tracks[1].file, Some("disc2.flac".to_string()));
    }

    #[test]
    fn test_parse_cue_file_empty_fields() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let cue_path = temp_dir.path().join("empty.cue");

        let cue_content = r#"FILE "audio.flac" WAVE
  TRACK 01 AUDIO
    INDEX 01 00:00:00
"#;
        std::fs::write(&cue_path, cue_content).unwrap();

        let result = parse_cue_file(&cue_path).unwrap();

        assert!(result.performer.is_none());
        assert!(result.title.is_none());
        assert_eq!(result.files, vec!["audio.flac".to_string()]);
        assert_eq!(result.tracks.len(), 1);
        assert_eq!(result.tracks[0].file, Some("audio.flac".to_string()));
    }

    #[test]
    fn test_parse_cue_file_missing_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let cue_path = temp_dir.path().join("nofile.cue");

        let cue_content = r#"PERFORMER "Artist"
TITLE "Album"
  TRACK 01 AUDIO
    TITLE "Track"
    INDEX 01 00:00:00
"#;
        std::fs::write(&cue_path, cue_content).unwrap();

        let result = parse_cue_file(&cue_path).unwrap();

        assert!(result.files.is_empty());
        assert_eq!(result.tracks.len(), 1);
        assert!(result.tracks[0].file.is_none());
    }

    #[test]
    fn test_parse_cue_file_nonexistent() {
        let result = parse_cue_file(&PathBuf::from("/nonexistent/test.cue"));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_cue_consistency_valid() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let cue_path = temp_dir.path().join("test.cue");
        let audio_path1 = temp_dir.path().join("track1.flac");
        let audio_path2 = temp_dir.path().join("track2.flac");

        std::fs::write(
            &cue_path,
            r#"PERFORMER "Artist"
TITLE "Album"
FILE "track1.flac" WAVE
  TRACK 01 AUDIO
    TITLE "Track 1"
    INDEX 01 00:00:00
FILE "track2.flac" WAVE
  TRACK 02 AUDIO
    TITLE "Track 2"
    INDEX 01 00:02:00
"#,
        )
        .unwrap();
        std::fs::write(&audio_path1, b"dummy audio").unwrap();
        std::fs::write(&audio_path2, b"dummy audio").unwrap();

        let audio_files: Vec<&Path> = vec![audio_path1.as_path(), audio_path2.as_path()];
        let result = validate_cue_consistency(&cue_path, &audio_files);

        assert!(result.is_valid);
        assert!(!result.parsing_error);
        assert!(!result.file_missing);
        assert!(!result.track_count_mismatch);
    }

    #[test]
    fn test_validate_cue_consistency_missing_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let cue_path = temp_dir.path().join("test.cue");
        let audio_path = temp_dir.path().join("existing.flac");

        std::fs::write(
            &cue_path,
            r#"PERFORMER "Artist"
TITLE "Album"
FILE "missing.flac" WAVE
  TRACK 01 AUDIO
    TITLE "Track"
    INDEX 01 00:00:00
"#,
        )
        .unwrap();
        std::fs::write(&audio_path, b"dummy audio").unwrap();

        let audio_files: Vec<&Path> = vec![audio_path.as_path()];
        let result = validate_cue_consistency(&cue_path, &audio_files);

        assert!(!result.is_valid);
        assert!(result.file_missing);
    }

    #[test]
    fn test_validate_cue_consistency_track_count_mismatch() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let cue_path = temp_dir.path().join("test.cue");
        let audio_path1 = temp_dir.path().join("track1.flac");
        let audio_path2 = temp_dir.path().join("track2.flac");
        let audio_path3 = temp_dir.path().join("track3.flac");

        std::fs::write(
            &cue_path,
            r#"PERFORMER "Artist"
TITLE "Album"
FILE "track1.flac" WAVE
  TRACK 01 AUDIO
    TITLE "Track 1"
    INDEX 01 00:00:00
FILE "track2.flac" WAVE
  TRACK 02 AUDIO
    TITLE "Track 2"
    INDEX 01 00:02:00
"#,
        )
        .unwrap();
        std::fs::write(&audio_path1, b"dummy audio").unwrap();
        std::fs::write(&audio_path2, b"dummy audio").unwrap();
        std::fs::write(&audio_path3, b"dummy audio").unwrap();

        let audio_files: Vec<&Path> = vec![
            audio_path1.as_path(),
            audio_path2.as_path(),
            audio_path3.as_path(),
        ];
        let result = validate_cue_consistency(&cue_path, &audio_files);

        assert!(!result.is_valid);
        assert!(result.track_count_mismatch);
    }

    #[test]
    fn test_validate_cue_consistency_invalid_content() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let cue_path = temp_dir.path().join("test.cue");
        let audio_path = temp_dir.path().join("track.flac");

        std::fs::write(&cue_path, "INVALID CUE CONTENT").unwrap();
        std::fs::write(&audio_path, b"dummy audio").unwrap();

        let audio_files: Vec<&Path> = vec![audio_path.as_path()];
        let result = validate_cue_consistency(&cue_path, &audio_files);

        assert!(!result.is_valid);
        assert!(!result.parsing_error);
        assert!(result.track_count_mismatch);
    }

    #[test]
    fn test_validate_cue_consistency_nonexistent_cue() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let cue_path = temp_dir.path().join("nonexistent.cue");
        let audio_path = temp_dir.path().join("track.flac");

        std::fs::write(&audio_path, b"dummy audio").unwrap();

        let audio_files: Vec<&Path> = vec![audio_path.as_path()];
        let result = validate_cue_consistency(&cue_path, &audio_files);

        assert!(!result.is_valid);
        assert!(result.parsing_error);
    }

    #[test]
    fn test_parse_cue_file_with_rem_genre() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let cue_path = temp_dir.path().join("test.cue");

        let cue_content = r#"PERFORMER "Test Artist"
TITLE "Test Album"
REM GENRE "Rock"
REM DATE 2024
FILE "test.flac" WAVE
  TRACK 01 AUDIO
    TITLE "Track One"
    INDEX 01 00:00:00
"#;
        std::fs::write(&cue_path, cue_content).unwrap();

        let result = parse_cue_file(&cue_path).unwrap();

        assert_eq!(result.genre, Some("Rock".to_string()));
        assert_eq!(result.date, Some("2024".to_string()));
    }

    #[test]
    fn test_parse_cue_file_with_rem_genre_no_quotes() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let cue_path = temp_dir.path().join("test.cue");

        let cue_content = r#"PERFORMER "Test Artist"
TITLE "Test Album"
REM GENRE Rock
REM DATE 2024
FILE "test.flac" WAVE
  TRACK 01 AUDIO
    TITLE "Track One"
    INDEX 01 00:00:00
"#;
        std::fs::write(&cue_path, cue_content).unwrap();

        let result = parse_cue_file(&cue_path).unwrap();

        assert_eq!(result.genre, Some("Rock".to_string()));
        assert_eq!(result.date, Some("2024".to_string()));
    }

    #[test]
    fn test_parse_cue_file_without_rem_fields() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let cue_path = temp_dir.path().join("test.cue");

        let cue_content = r#"PERFORMER "Test Artist"
TITLE "Test Album"
FILE "test.flac" WAVE
  TRACK 01 AUDIO
    TITLE "Track One"
    INDEX 01 00:00:00
"#;
        std::fs::write(&cue_path, cue_content).unwrap();

        let result = parse_cue_file(&cue_path).unwrap();

        assert!(result.genre.is_none());
        assert!(result.date.is_none());
    }

    #[test]
    fn test_parse_cue_file_with_tabs() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let cue_path = temp_dir.path().join("test.cue");

        let cue_content = "PERFORMER \"Test Artist\"\n\
TITLE \"Test Album\"\n\
FILE \"test.flac\" WAVE\n\
\tTRACK 01 AUDIO\n\
\t\tTITLE \"Track One\"\n\
\t\tPERFORMER \"Track Artist\"\n\
\t\tINDEX 01 00:00:00\n\
\tTRACK 02 AUDIO\n\
\t\tTITLE \"Track Two\"\n\
\t\tINDEX 01 00:03:00\n";
        std::fs::write(&cue_path, cue_content).unwrap();

        let result = parse_cue_file(&cue_path).unwrap();

        assert_eq!(result.performer, Some("Test Artist".to_string()));
        assert_eq!(result.title, Some("Test Album".to_string()));
        assert_eq!(result.files, vec!["test.flac".to_string()]);
        assert_eq!(result.tracks.len(), 2);
        assert_eq!(result.tracks[0].number, 1);
        assert_eq!(result.tracks[0].title, Some("Track One".to_string()));
        assert_eq!(result.tracks[0].performer, Some("Track Artist".to_string()));
        assert_eq!(result.tracks[1].number, 2);
        assert_eq!(result.tracks[1].title, Some("Track Two".to_string()));
    }
}
