//! Cue file generation and parsing services.

use std::collections::HashSet;
use std::fmt::Write;
use std::path::{Path, PathBuf};

use crate::adapters::audio_formats::read_metadata;
use crate::core::domain::models::{AlbumNode, MetadataSource, MetadataValue, TrackNode};
use crate::core::services::normalization::to_title_case;
use crate::core::services::scanner::scan_dir_immediate;

// ── Metadata helpers ────────────────────────────────────────────────────────

/// Returns `true` when the metadata value originates from an embedded tag.
fn is_embedded<T>(mv: &MetadataValue<T>) -> bool {
    matches!(mv.source, MetadataSource::Embedded)
}

/// Selects the best value across all tracks for a given metadata field.
///
/// *Embedded* sources always beat inferred ones; among values with the same
/// source kind the highest confidence wins.  Ties are broken in favour of the
/// first occurrence (track order).
fn best_value<T: Clone>(
    tracks: &[TrackNode],
    extractor: impl Fn(&TrackNode) -> Option<&MetadataValue<T>>,
) -> Option<T> {
    let mut best: Option<(T, bool, f32)> = None;

    for track in tracks {
        if let Some(mv) = extractor(track) {
            let emb = is_embedded(mv);
            let dominated = best.as_ref().is_some_and(|(_, cur_emb, cur_conf)| {
                (*cur_emb && !emb) || (*cur_emb == emb && *cur_conf >= mv.confidence)
            });
            if !dominated {
                best = Some((mv.value.clone(), emb, mv.confidence));
            }
        }
    }

    best.map(|(v, _, _)| v)
}

/// Returns the track-level performer: prefers `album_artist`, falls back to
/// `artist`.
fn track_performer(track: &TrackNode) -> Option<&String> {
    track
        .metadata
        .album_artist
        .as_ref()
        .or(track.metadata.artist.as_ref())
        .map(|mv| &mv.value)
}

// ── CUE generation ─────────────────────────────────────────────────────────

/// Generates `.cue` file content for an album from its track metadata.
///
/// When tracks carry conflicting metadata (different artists, years, or
/// genres), embedded metadata takes precedence over folder-inferred values.
/// Among values with the same source kind the highest confidence wins.
/// Text fields (artist, album, genre) are normalised to title case.
pub fn generate_cue_content(album: &AlbumNode) -> String {
    let tracks = &album.tracks;
    let mut out = String::new();

    // Album-level PERFORMER
    let artist = best_value(tracks, |t| t.metadata.album_artist.as_ref())
        .or_else(|| best_value(tracks, |t| t.metadata.artist.as_ref()));
    if let Some(name) = artist {
        let _ = writeln!(out, "PERFORMER \"{}\"", to_title_case(&name));
    }

    // Album-level TITLE
    let title =
        best_value(tracks, |t| t.metadata.album.as_ref()).unwrap_or_else(|| album.title.clone());
    let _ = writeln!(out, "TITLE \"{}\"", to_title_case(&title));

    // REM GENRE
    if let Some(genre) = best_value(tracks, |t| t.metadata.genre.as_ref()) {
        let _ = writeln!(out, "REM GENRE {}", to_title_case(&genre));
    }

    // REM DATE – prefer embedded year, then album.year, then best inferred
    let year = best_value(tracks, |t| {
        t.metadata.year.as_ref().filter(|mv| is_embedded(mv))
    })
    .or(album.year)
    .or_else(|| best_value(tracks, |t| t.metadata.year.as_ref()));
    if let Some(y) = year {
        let _ = writeln!(out, "REM DATE {}", y);
    }

    // Tracks, grouped by source file
    let mut current_file: Option<String> = None;
    let mut file_track_idx: u32 = 0;

    for (i, track) in tracks.iter().enumerate() {
        let file_name = track
            .file_path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "unknown.flac".to_string());

        if current_file.as_deref() != Some(&file_name) {
            current_file = Some(file_name.clone());
            file_track_idx = 0;
            let _ = writeln!(out, "FILE \"{}\" WAVE", file_name);
        }

        let track_num = track
            .metadata
            .track_number
            .as_ref()
            .map(|mv| mv.value)
            .unwrap_or((i + 1) as u32);

        let _ = writeln!(out, "  TRACK {:02} AUDIO", track_num);

        if let Some(t) = track.metadata.title.as_ref() {
            let _ = writeln!(out, "    TITLE \"{}\"", t.value);
        }

        if let Some(performer) = track_performer(track) {
            let _ = writeln!(out, "    PERFORMER \"{}\"", to_title_case(performer));
        }

        let _ = writeln!(out, "    INDEX 01 00:{:02}:00", file_track_idx * 2);
        file_track_idx += 1;
    }

    out
}

/// Returns the default `.cue` filename for an album.
pub fn generate_cue_file_name(album: &AlbumNode) -> String {
    format!("{}.cue", album.title)
}

/// Writes a `.cue` file for an album to the given path.
pub fn write_cue_file(album: &AlbumNode, output_path: &Path) -> Result<(), std::io::Error> {
    std::fs::write(output_path, generate_cue_content(album))
}

// ── Path-based CUE generation ───────────────────────────────────────────────

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

    let tracks: Vec<_> = file_paths
        .iter()
        .map(|fp| {
            read_metadata(fp).map_err(|e| {
                CueGenerationError::FileReadError(
                    format!("Failed to read {}: {}", fp.display(), e,),
                )
            })
        })
        .collect::<Result<_, _>>()?;

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
        .map(|t| TrackNode {
            file_path: t.file_path,
            metadata: t.metadata,
        })
        .collect();

    let album_files: HashSet<PathBuf> = track_nodes.iter().map(|t| t.file_path.clone()).collect();

    let album = AlbumNode {
        title: album_name,
        year: None,
        tracks: track_nodes,
        files: album_files,
        path: path.to_path_buf(),
    };

    let output_path = output.unwrap_or_else(|| path.join(generate_cue_file_name(&album)));
    let cue_content = generate_cue_content(&album);

    Ok(CueGenerationResult {
        cue_content,
        output_path,
        tracks_count,
    })
}

// ── CUE parsing ─────────────────────────────────────────────────────────────

/// Represents a parsed `.cue` file.
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct CueFile {
    pub performer: Option<String>,
    pub title: Option<String>,
    pub genre: Option<String>,
    pub date: Option<String>,
    pub files: Vec<String>,
    pub tracks: Vec<CueTrack>,
}

/// Represents a track in a `.cue` file.
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct CueTrack {
    pub number: u32,
    pub title: Option<String>,
    pub performer: Option<String>,
    pub index: Option<String>,
    pub file: Option<String>,
}

/// Extracts the text between the first and last `"` on a line.
fn extract_quoted_value(line: &str) -> Option<String> {
    let start = line.find('"')?;
    let end = line.rfind('"')?;
    (start < end).then(|| line[start + 1..end].to_string())
}

/// Parses a `TRACK NN AUDIO` line into a [`CueTrack`].
fn parse_track_line(line: &str) -> Option<CueTrack> {
    let mut parts = line.split_whitespace();
    let _ = parts.next(); // "TRACK"
    let number = parts.next()?.parse::<u32>().ok()?;
    (parts.next()? == "AUDIO").then(|| CueTrack {
        number,
        ..Default::default()
    })
}

/// Parses a `.cue` file and returns a [`CueFile`] with the extracted data.
pub fn parse_cue_file(cue_path: &Path) -> Result<CueFile, String> {
    let content = std::fs::read_to_string(cue_path)
        .map_err(|e| format!("Failed to read CUE file '{}': {}", cue_path.display(), e))?;

    let mut cue = CueFile::default();
    let mut current_track: Option<CueTrack> = None;
    let mut current_file: Option<String> = None;

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        let is_track_level = line.starts_with("  ") || line.starts_with('\t');
        let line_ctx = || format!("line {}: {}", line_num + 1, line);

        match (
            trimmed.split_whitespace().next().unwrap_or(""),
            is_track_level,
        ) {
            // Album-level directives
            ("PERFORMER", false) => {
                cue.performer = Some(
                    extract_quoted_value(trimmed)
                        .ok_or_else(|| format!("Malformed PERFORMER at {}", line_ctx()))?,
                );
            }
            ("TITLE", false) => {
                cue.title = Some(
                    extract_quoted_value(trimmed)
                        .ok_or_else(|| format!("Malformed TITLE at {}", line_ctx()))?,
                );
            }
            ("FILE", _) => {
                let name = extract_quoted_value(trimmed)
                    .ok_or_else(|| format!("Malformed FILE at {}", line_ctx()))?;
                current_file = Some(name.clone());
                cue.files.push(name);
            }

            // REM fields
            ("REM", _) if trimmed.starts_with("REM GENRE") => {
                cue.genre = extract_quoted_value(trimmed).or_else(|| {
                    let v = trimmed.trim_start_matches("REM GENRE").trim();
                    (!v.is_empty()).then(|| v.to_string())
                });
            }
            ("REM", _) if trimmed.starts_with("REM DATE") => {
                let v = trimmed.trim_start_matches("REM DATE").trim();
                if !v.is_empty() {
                    cue.date = Some(v.to_string());
                }
            }

            // Track-level directives
            ("TRACK", true) => {
                if let Some(prev) = current_track.take() {
                    cue.tracks.push(prev);
                }
                let mut track = parse_track_line(trimmed)
                    .ok_or_else(|| format!("Malformed TRACK at {}", line_ctx()))?;
                track.file = current_file.clone();
                current_track = Some(track);
            }
            ("TITLE", true) if current_track.is_some() => {
                current_track.as_mut().unwrap().title = Some(
                    extract_quoted_value(trimmed)
                        .ok_or_else(|| format!("Malformed TRACK TITLE at {}", line_ctx()))?,
                );
            }
            ("PERFORMER", true) if current_track.is_some() => {
                current_track.as_mut().unwrap().performer = Some(
                    extract_quoted_value(trimmed)
                        .ok_or_else(|| format!("Malformed TRACK PERFORMER at {}", line_ctx()))?,
                );
            }
            ("INDEX", true) if current_track.is_some() => {
                let remainder = trimmed.trim_start_matches("INDEX").trim();
                let parts: Vec<&str> = remainder.split_whitespace().collect();
                if parts.len() >= 2 && parts[0].parse::<u32>().is_ok() {
                    current_track.as_mut().unwrap().index = Some(remainder.to_string());
                } else {
                    return Err(format!("Malformed INDEX at {}", line_ctx()));
                }
            }

            _ => {} // ignore unknown / blank lines
        }
    }

    if let Some(track) = current_track {
        cue.tracks.push(track);
    }

    Ok(cue)
}

// ── CUE validation ─────────────────────────────────────────────────────────

/// Result of `.cue` file validation.
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct CueValidationResult {
    pub is_valid: bool,
    pub parsing_error: bool,
    pub file_missing: bool,
    pub track_count_mismatch: bool,
}

/// Validates the consistency of a `.cue` file against a set of audio files.
pub fn validate_cue_consistency(cue_path: &Path, audio_files: &[&Path]) -> CueValidationResult {
    let mut result = CueValidationResult::default();

    let cue = match parse_cue_file(cue_path) {
        Ok(c) => c,
        Err(_) => {
            result.parsing_error = true;
            result.track_count_mismatch = true;
            return result;
        }
    };

    result.is_valid = true;

    for ref_name in &cue.files {
        let found = audio_files.iter().any(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n == ref_name)
        });
        if !found {
            result.file_missing = true;
            result.is_valid = false;
        }
    }

    if cue.files.len() != audio_files.len() {
        result.track_count_mismatch = true;
        result.is_valid = false;
    }

    result
}

pub fn format_cue_validation_result(result: &CueValidationResult) -> String {
    if result.is_valid {
        return "CUE file is valid: All referenced files exist and track count matches."
            .to_string();
    }

    let mut errors = Vec::new();
    if result.parsing_error {
        errors.push("Error parsing CUE file");
    }
    if result.file_missing {
        errors.push("Referenced audio file(s) missing");
    }
    if result.track_count_mismatch {
        errors.push("Track count mismatch between CUE and audio files");
    }

    if errors.is_empty() {
        "CUE file validation failed.".to_string()
    } else {
        format!("CUE file validation failed:\n  - {}", errors.join("\n  - "))
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TrackMetadata;
    use std::path::PathBuf;

    fn make_track(
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

    fn make_album(title: &str, year: Option<u32>, tracks: Vec<TrackNode>) -> AlbumNode {
        let files: HashSet<PathBuf> = tracks.iter().map(|t| t.file_path.clone()).collect();
        AlbumNode {
            title: title.to_string(),
            year,
            tracks,
            files,
            path: PathBuf::from("/test"),
        }
    }

    #[test]
    fn test_generate_cue_content_basic() {
        let tracks = vec![
            make_track(
                "Song One",
                "Test Artist",
                "track1.flac",
                Some(2024),
                Some("Rock"),
            ),
            make_track(
                "Song Two",
                "Test Artist",
                "track2.flac",
                Some(2024),
                Some("Rock"),
            ),
        ];
        let album = make_album("Test Album", Some(2024), tracks);
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
        let album = make_album("My Album", None, vec![]);
        assert_eq!(generate_cue_file_name(&album), "My Album.cue");
    }

    #[test]
    fn test_write_cue_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let cue_path = temp_dir.path().join("Test Album.cue");

        let tracks = vec![make_track(
            "Song",
            "Artist",
            "file.flac",
            None,
            Some("Rock"),
        )];
        let album = make_album("Test Album", Some(2024), tracks);

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
            make_track("Track 1", "Artist", "album.flac", Some(2023), Some("Jazz")),
            make_track("Track 2", "Artist", "album.flac", Some(2023), Some("Jazz")),
            make_track("Track 3", "Artist", "album.flac", Some(2023), Some("Jazz")),
        ];
        let album = make_album("Album", Some(2023), tracks);
        let content = generate_cue_content(&album);

        assert_eq!(
            content.match_indices("FILE ").count(),
            1,
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
            make_track("Track 1", "Artist", "01.flac", None, Some("Electronic")),
            make_track("Track 2", "Artist", "02.flac", None, Some("Electronic")),
            make_track("Track 3", "Artist", "03.flac", None, Some("Electronic")),
        ];
        let album = make_album("Album", None, tracks);
        let content = generate_cue_content(&album);

        assert_eq!(
            content.match_indices("FILE ").count(),
            3,
            "Should have FILE entry for each track"
        );
        assert!(content.contains("FILE \"01.flac\" WAVE"));
        assert!(content.contains("FILE \"02.flac\" WAVE"));
        assert!(content.contains("FILE \"03.flac\" WAVE"));
    }

    #[test]
    fn test_generate_cue_content_track_timing() {
        let tracks = vec![
            make_track("Track 1", "Artist", "file.flac", None, None),
            make_track("Track 2", "Artist", "file.flac", None, None),
        ];
        let album = make_album("Album", None, tracks);
        let content = generate_cue_content(&album);

        assert!(content.contains("INDEX 01 00:00:00"));
        assert!(content.contains("INDEX 01 00:02:00"));
    }

    #[test]
    fn test_generate_cue_content_genre_from_track() {
        let tracks = vec![
            make_track(
                "Song One",
                "Artist",
                "track1.flac",
                Some(2020),
                Some("Classical"),
            ),
            make_track(
                "Song Two",
                "Artist",
                "track2.flac",
                Some(2020),
                Some("Classical"),
            ),
        ];
        let album = make_album("Album Title", Some(2020), tracks);
        let content = generate_cue_content(&album);
        assert!(content.contains("REM GENRE Classical"));
    }

    #[test]
    fn test_generate_cue_content_year_from_track() {
        let tracks = vec![
            make_track("Song One", "Artist", "track1.flac", Some(2019), Some("Pop")),
            make_track("Song Two", "Artist", "track2.flac", Some(2019), Some("Pop")),
        ];
        let album = make_album("Album", None, tracks);
        let content = generate_cue_content(&album);
        assert!(content.contains("REM DATE 2019"));
    }

    #[test]
    fn test_generate_cue_content_no_genre() {
        let tracks = vec![
            make_track("Song One", "Artist", "track1.flac", None, None),
            make_track("Song Two", "Artist", "track2.flac", None, None),
        ];
        let album = make_album("Album", None, tracks);
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
        let album = make_album("Album", None, tracks);
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
        let album = make_album("Folder Name", None, tracks);
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
        let album = make_album("Folder Album", None, tracks);
        let content = generate_cue_content(&album);

        assert!(content.contains("TITLE \"Album From Tags\""));
        assert!(content.contains("REM DATE 2021"));
        assert!(content.contains("REM GENRE Metal"));
    }

    #[test]
    fn test_generate_cue_content_title_case_normalization() {
        let tracks = vec![make_track(
            "Song",
            "UPPERCASE ARTIST",
            "track1.flac",
            Some(2020),
            Some("ROCK"),
        )];
        let album = make_album("UPPERCASE ALBUM", Some(2020), tracks);
        let content = generate_cue_content(&album);

        assert!(content.contains("PERFORMER \"Uppercase Artist\""));
        assert!(content.contains("TITLE \"Uppercase Album\""));
        assert!(content.contains("REM GENRE Rock"));
    }

    #[test]
    fn test_extract_quoted_value() {
        assert_eq!(
            extract_quoted_value(r#"PERFORMER "Test Artist""#),
            Some("Test Artist".to_string()),
        );
        assert_eq!(
            extract_quoted_value(r#"TITLE "Test Album""#),
            Some("Test Album".to_string()),
        );
    }

    #[test]
    fn test_extract_quoted_value_no_quotes() {
        assert!(extract_quoted_value("TITLE Hello World").is_none());
    }

    #[test]
    fn test_parse_cue_file_basic() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let cue_path = temp_dir.path().join("test.cue");

        std::fs::write(
            &cue_path,
            r#"PERFORMER "Test Artist"
TITLE "Test Album"
FILE "test.flac" WAVE
  TRACK 01 AUDIO
    TITLE "Track One"
    PERFORMER "Track Artist"
    INDEX 01 00:00:00
  TRACK 02 AUDIO
    TITLE "Track Two"
    INDEX 01 00:03:00
"#,
        )
        .unwrap();

        let result = parse_cue_file(&cue_path).unwrap();

        assert_eq!(result.performer, Some("Test Artist".to_string()));
        assert_eq!(result.title, Some("Test Album".to_string()));
        assert!(result.genre.is_none());
        assert!(result.date.is_none());
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

        std::fs::write(
            &cue_path,
            r#"PERFORMER "Artist"
TITLE "Album"
FILE "tracks.flac" WAVE
  TRACK 01 AUDIO
    TITLE "First Track"
    INDEX 01 00:00:00
"#,
        )
        .unwrap();

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

        std::fs::write(
            &cue_path,
            r#"PERFORMER "Various Artists"
TITLE "Compilation"
FILE "disc1.flac" WAVE
  TRACK 01 AUDIO
    TITLE "Track 1"
    INDEX 01 00:00:00
FILE "disc2.flac" WAVE
  TRACK 02 AUDIO
    TITLE "Track 2"
    INDEX 01 00:00:00
"#,
        )
        .unwrap();

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

        std::fs::write(
            &cue_path,
            r#"FILE "audio.flac" WAVE
  TRACK 01 AUDIO
    INDEX 01 00:00:00
"#,
        )
        .unwrap();

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

        std::fs::write(
            &cue_path,
            r#"PERFORMER "Artist"
TITLE "Album"
  TRACK 01 AUDIO
    TITLE "Track"
    INDEX 01 00:00:00
"#,
        )
        .unwrap();

        let result = parse_cue_file(&cue_path).unwrap();

        assert!(result.files.is_empty());
        assert_eq!(result.tracks.len(), 1);
        assert!(result.tracks[0].file.is_none());
    }

    #[test]
    fn test_parse_cue_file_nonexistent() {
        assert!(parse_cue_file(&PathBuf::from("/nonexistent/test.cue")).is_err());
    }

    #[test]
    fn test_validate_cue_consistency_valid() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let cue_path = temp_dir.path().join("test.cue");
        let audio1 = temp_dir.path().join("track1.flac");
        let audio2 = temp_dir.path().join("track2.flac");

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
        std::fs::write(&audio1, b"dummy audio").unwrap();
        std::fs::write(&audio2, b"dummy audio").unwrap();

        let result = validate_cue_consistency(&cue_path, &[audio1.as_path(), audio2.as_path()]);

        assert!(result.is_valid);
        assert!(!result.parsing_error);
        assert!(!result.file_missing);
        assert!(!result.track_count_mismatch);
    }

    #[test]
    fn test_validate_cue_consistency_missing_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let cue_path = temp_dir.path().join("test.cue");
        let audio = temp_dir.path().join("existing.flac");

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
        std::fs::write(&audio, b"dummy audio").unwrap();

        let result = validate_cue_consistency(&cue_path, &[audio.as_path()]);

        assert!(!result.is_valid);
        assert!(result.file_missing);
    }

    #[test]
    fn test_validate_cue_consistency_track_count_mismatch() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let cue_path = temp_dir.path().join("test.cue");
        let audio1 = temp_dir.path().join("track1.flac");
        let audio2 = temp_dir.path().join("track2.flac");
        let audio3 = temp_dir.path().join("track3.flac");

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
        std::fs::write(&audio1, b"dummy audio").unwrap();
        std::fs::write(&audio2, b"dummy audio").unwrap();
        std::fs::write(&audio3, b"dummy audio").unwrap();

        let result = validate_cue_consistency(
            &cue_path,
            &[audio1.as_path(), audio2.as_path(), audio3.as_path()],
        );

        assert!(!result.is_valid);
        assert!(result.track_count_mismatch);
    }

    #[test]
    fn test_validate_cue_consistency_invalid_content() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let cue_path = temp_dir.path().join("test.cue");
        let audio = temp_dir.path().join("track.flac");

        std::fs::write(&cue_path, "INVALID CUE CONTENT").unwrap();
        std::fs::write(&audio, b"dummy audio").unwrap();

        let result = validate_cue_consistency(&cue_path, &[audio.as_path()]);

        assert!(!result.is_valid);
        assert!(!result.parsing_error);
        assert!(result.track_count_mismatch);
    }

    #[test]
    fn test_validate_cue_consistency_nonexistent_cue() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let audio = temp_dir.path().join("track.flac");
        std::fs::write(&audio, b"dummy audio").unwrap();

        let result =
            validate_cue_consistency(&temp_dir.path().join("nonexistent.cue"), &[audio.as_path()]);

        assert!(!result.is_valid);
        assert!(result.parsing_error);
    }

    #[test]
    fn test_parse_cue_file_with_rem_genre() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let cue_path = temp_dir.path().join("test.cue");

        std::fs::write(
            &cue_path,
            r#"PERFORMER "Test Artist"
TITLE "Test Album"
REM GENRE "Rock"
REM DATE 2024
FILE "test.flac" WAVE
  TRACK 01 AUDIO
    TITLE "Track One"
    INDEX 01 00:00:00
"#,
        )
        .unwrap();

        let result = parse_cue_file(&cue_path).unwrap();

        assert_eq!(result.genre, Some("Rock".to_string()));
        assert_eq!(result.date, Some("2024".to_string()));
    }

    #[test]
    fn test_parse_cue_file_with_rem_genre_no_quotes() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let cue_path = temp_dir.path().join("test.cue");

        std::fs::write(
            &cue_path,
            r#"PERFORMER "Test Artist"
TITLE "Test Album"
REM GENRE Rock
REM DATE 2024
FILE "test.flac" WAVE
  TRACK 01 AUDIO
    TITLE "Track One"
    INDEX 01 00:00:00
"#,
        )
        .unwrap();

        let result = parse_cue_file(&cue_path).unwrap();

        assert_eq!(result.genre, Some("Rock".to_string()));
        assert_eq!(result.date, Some("2024".to_string()));
    }

    #[test]
    fn test_parse_cue_file_without_rem_fields() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let cue_path = temp_dir.path().join("test.cue");

        std::fs::write(
            &cue_path,
            r#"PERFORMER "Test Artist"
TITLE "Test Album"
FILE "test.flac" WAVE
  TRACK 01 AUDIO
    TITLE "Track One"
    INDEX 01 00:00:00
"#,
        )
        .unwrap();

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
