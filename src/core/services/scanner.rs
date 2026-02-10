//! Enhanced directory scanner with improved error handling and edge cases.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use glob::Pattern;
use log::{error, warn};
use rayon::prelude::*;
use walkdir::WalkDir;

use crate::adapters::audio_formats::{self as formats, read_basic_info};
use crate::core::domain::models::{
    FOLDER_INFERRED_CONFIDENCE, MetadataSource, MetadataValue, Track, TrackMetadata,
};
use crate::core::services::cue::parse_cue_file;
use crate::core::services::inference::{infer_album_from_path, infer_artist_from_path};

// ── Shared helpers ──────────────────────────────────────────────────────────

/// Builds the set of supported audio extensions (lowercase).
fn supported_extensions() -> HashSet<String> {
    formats::get_supported_extensions().into_iter().collect()
}

/// Returns `true` when `path` has a supported audio extension.
fn is_supported(path: &Path, exts: &HashSet<String>) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| exts.contains(&e.to_lowercase()))
}

/// Returns `true` for known audio extensions we don't currently support.
fn has_known_audio_ext(path: &Path) -> bool {
    const KNOWN: &[&str] = &["mp3", "flac", "wav", "dsf", "wv"];
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| KNOWN.contains(&e.to_lowercase().as_str()))
}

/// Returns `true` when `path` is a symbolic link.
fn is_symlink(path: &Path) -> bool {
    path.symlink_metadata()
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

/// Rejects empty or unreadable files.
fn validate_file(path: &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if std::fs::metadata(path)?.len() == 0 {
        return Err("File is empty".into());
    }
    let _ = std::fs::File::open(path)?;
    Ok(())
}

/// Returns `true` if `path` matches any of the given glob patterns.
fn matches_any_pattern(path: &Path, patterns: &[String]) -> bool {
    patterns.iter().any(|pat| {
        Pattern::new(pat)
            .map(|p| p.matches_path(path))
            .unwrap_or_else(|e| {
                error!(target: "music_chore", "Invalid glob pattern: {pat} - {e}");
                false
            })
    })
}

/// Lowercased file extension, or `"unknown"`.
fn file_format(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("unknown")
        .to_lowercase()
}

// ── Album-from-filename heuristics ──────────────────────────────────────────

/// Attempts to extract an album name from a bare filename (no extension).
///
/// Recognised patterns:
/// - `"Artist - Album - Track"` (two `" - "` separators)
/// - `"Album - Track"` (single separator)
/// - `"Track (Album)"` (parenthesised)
fn album_from_filename(name: &str) -> Option<String> {
    // Two separators: decide which part is the album
    if let Some(first) = name.find(" - ") {
        let rest = &name[first + 3..];
        if let Some(second) = rest.find(" - ") {
            let middle = rest[..second].trim();
            let first_part = name[..first].trim();
            // Numeric middle → track number; album is the first part
            return if middle.parse::<u32>().is_ok() {
                (!first_part.is_empty()).then(|| first_part.to_string())
            } else {
                (!middle.is_empty()).then(|| middle.to_string())
            };
        }
        // Single separator: "Album - Track"
        let candidate = name[..first].trim();
        if !candidate.is_empty() {
            return Some(candidate.to_string());
        }
    }

    // Parenthesised: "Track (Album)"
    if let Some(open) = name.find('(') {
        if let Some(close) = name[open..].find(')') {
            let candidate = name[open + 1..open + close].trim();
            if !candidate.is_empty() {
                return Some(candidate.to_string());
            }
        }
    }

    None
}

/// Strips leading track-number prefix and extension, normalises separators.
fn cleaned_filename(name: &str) -> String {
    let mut s = name.to_string();
    if let Some(i) = s.find(" - ") {
        s = s[i + 3..].to_string();
    }
    if let Some(i) = s.rfind('.') {
        s.truncate(i);
    }
    s.replace(['_', '-'], " ").trim().to_string()
}

// ── Metadata inference ──────────────────────────────────────────────────────

/// Resolves the best album value for a path by trying, in order:
/// 1. Folder-inferred album from directory structure
/// 2. Heuristic extraction from filename
/// 3. Cleaned filename as last resort
fn infer_album(path: &Path) -> Option<MetadataValue<String>> {
    if let Some(album) = infer_album_from_path(path) {
        return Some(MetadataValue::inferred(album, FOLDER_INFERRED_CONFIDENCE));
    }
    let stem = path.file_stem().and_then(|n| n.to_str())?;
    if let Some(album) = album_from_filename(stem) {
        return Some(MetadataValue::inferred(album, FOLDER_INFERRED_CONFIDENCE));
    }
    let clean = cleaned_filename(stem);
    (!clean.is_empty()).then(|| MetadataValue::inferred(clean, FOLDER_INFERRED_CONFIDENCE))
}

/// Builds `TrackMetadata` from path inference only (no embedded tag reading).
fn inferred_metadata(path: &Path) -> TrackMetadata {
    TrackMetadata {
        title: path
            .file_stem()
            .and_then(|n| n.to_str())
            .map(|s| MetadataValue::inferred(s.to_string(), FOLDER_INFERRED_CONFIDENCE)),
        artist: infer_artist_from_path(path)
            .map(|a| MetadataValue::inferred(a, FOLDER_INFERRED_CONFIDENCE)),
        album: infer_album(path),
        album_artist: None,
        track_number: None,
        disc_number: None,
        year: None,
        genre: None,
        duration: None,
        format: file_format(path),
        path: path.to_path_buf(),
    }
}

/// Reads embedded tags, then fills any missing fields via path inference.
fn full_metadata(path: &Path) -> TrackMetadata {
    let embedded = formats::read_metadata(path).ok();

    let mut md = match embedded {
        Some(track) => TrackMetadata {
            format: file_format(path),
            path: path.to_path_buf(),
            ..track.metadata
        },
        None => TrackMetadata {
            title: None,
            artist: None,
            album: None,
            album_artist: None,
            track_number: None,
            disc_number: None,
            year: None,
            genre: None,
            duration: None,
            format: file_format(path),
            path: path.to_path_buf(),
        },
    };

    // Fill gaps with folder inference
    if md.artist.is_none() {
        md.artist = infer_artist_from_path(path)
            .map(|a| MetadataValue::inferred(a, FOLDER_INFERRED_CONFIDENCE));
    }
    if md.album.is_none() {
        md.album = infer_album(path);
    }

    md
}

// ── Display helpers ─────────────────────────────────────────────────────────

/// Maps a `MetadataSource` to its display emoji.
fn source_icon(source: &MetadataSource) -> &'static str {
    match source {
        MetadataSource::CueInferred => "📄",
        MetadataSource::Embedded => "🎯",
        MetadataSource::UserEdited => "👤",
        MetadataSource::FolderInferred => "🤖",
    }
}

/// Format the track name for human-readable `scan` output, with source icon.
///
/// Prioritises CUE-inferred title, then embedded title, then bare filename.
pub fn format_track_name_for_scan_output(track: &Track) -> String {
    let filename = track
        .file_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();

    let (name, src) = match track.metadata.title.as_ref() {
        Some(mv) if mv.source == MetadataSource::CueInferred => {
            (format!("{} ({})", mv.value, filename), &mv.source)
        }
        Some(mv) if !mv.value.is_empty() => (mv.value.clone(), &mv.source),
        _ => {
            let fallback = track
                .metadata
                .artist
                .as_ref()
                .or(track.metadata.album.as_ref())
                .map(|mv| &mv.source)
                .unwrap_or(&MetadataSource::FolderInferred);
            (filename.to_string(), fallback)
        }
    };

    format!("{} {}", name, source_icon(src))
}

// ── Public scan API ─────────────────────────────────────────────────────────

/// Recursively scan `base` for supported music files.
///
/// Uses deterministic ordering (sorted by filename).
/// Logs warnings for unsupported file types.
pub fn scan_dir(base: &Path, skip_metadata: bool) -> Vec<Track> {
    scan_dir_with_options(base, None, false, Vec::new(), skip_metadata)
}

/// Scan only the immediate directory (non-recursive) for audio file paths.
pub fn scan_dir_immediate(base: &Path) -> Vec<PathBuf> {
    if !base.is_dir() {
        return Vec::new();
    }

    let exts = supported_extensions();
    let Ok(entries) = std::fs::read_dir(base) else {
        return Vec::new();
    };

    let mut paths: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            !is_symlink(p) && p.is_file() && is_supported(p, &exts) && {
                validate_file(p)
                    .map_err(|e| {
                        error!(target: "music_chore", "Skipping invalid file {}: {}", p.display(), e)
                    })
                    .is_ok()
            }
        })
        .collect();

    paths.sort();
    paths
}

/// Recursively scan and return file paths, skipping symlinks.
pub fn scan_dir_paths(base: &Path) -> Vec<PathBuf> {
    let exts = supported_extensions();
    let mut paths: Vec<PathBuf> = walk(base, None, false)
        .map(|e| e.into_path())
        .filter(|p| {
            !is_symlink(p) && p.is_file() && is_supported(p, &exts) && {
                validate_file(p)
                    .map_err(|e| {
                        error!(target: "music_chore", "Skipping invalid file {}: {}", p.display(), e)
                    })
                    .is_ok()
            }
        })
        .collect();

    paths.sort();
    paths
}

/// Scan and read full metadata for all supported files under `base`.
pub fn scan_dir_with_metadata(base: &Path) -> Result<Vec<Track>, String> {
    let mut map = BTreeMap::new();

    for entry in walk(base, None, false) {
        let path = entry.path();
        if is_symlink(path) || !path.is_file() || !formats::is_format_supported(path) {
            continue;
        }
        if let Err(e) = validate_file(path) {
            error!(target: "music_chore", "Skipping invalid file {}: {}", path.display(), e);
            continue;
        }
        match formats::read_metadata(path) {
            Ok(track) => {
                map.insert(path.to_path_buf(), track);
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

    Ok(map.into_values().collect())
}

/// Scan for tracks and detect duplicates by checksum.
pub fn scan_with_duplicates(
    base: &Path,
    verbose: bool,
    parallel: Option<usize>,
) -> (Vec<Track>, Vec<Vec<Track>>) {
    if let Some(threads) = parallel {
        let _ = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global();
    }

    let tracks = scan_dir(base, true);

    let all: Vec<Track> = tracks
        .into_par_iter()
        .map(|mut track| {
            if verbose {
                println!("Scanning {}...", track.file_path.display());
            }
            match track.calculate_checksum() {
                Ok(cs) => {
                    track.checksum = Some(cs);
                }
                Err(e) => {
                    eprintln!(
                        "Warning: checksum failed for {}: {}",
                        track.file_path.display(),
                        e,
                    );
                }
            }
            track
        })
        .collect();

    let mut by_checksum: HashMap<String, Vec<Track>> = HashMap::new();
    for track in &all {
        if let Some(cs) = &track.checksum {
            by_checksum
                .entry(cs.clone())
                .or_default()
                .push(track.clone());
        }
    }

    let dupes = by_checksum.into_values().filter(|g| g.len() > 1).collect();
    (all, dupes)
}

/// Scan directory with optional depth limit (path-only inference, no metadata).
pub fn scan_dir_with_depth(base: &Path, max_depth: Option<usize>) -> Vec<Track> {
    scan_dir_with_options(base, max_depth, false, Vec::new(), true)
}

/// Scan directory with depth limit and symlink handling (path-only inference).
pub fn scan_dir_with_depth_and_symlinks(
    base: &Path,
    max_depth: Option<usize>,
    follow_symlinks: bool,
) -> Vec<Track> {
    scan_dir_with_options(base, max_depth, follow_symlinks, Vec::new(), true)
}

/// Scan tracks and return formatted output (text or JSON).
pub fn scan_tracks(path: PathBuf, json_output: bool) -> Result<String, String> {
    if !path.exists() {
        return Err("No music files found in directory".to_string());
    }

    let tracks = scan_dir(&path, false);
    if tracks.is_empty() {
        return Err("No music files found in directory".to_string());
    }

    if json_output {
        serde_json::to_string_pretty(&tracks).map_err(|e| format!("Error serializing to JSON: {e}"))
    } else {
        Ok(tracks
            .iter()
            .map(|t| {
                format!(
                    "{} [{}]\n",
                    t.file_path.display(),
                    format_track_name_for_scan_output(t),
                )
            })
            .collect())
    }
}

/// Full-featured directory scan with depth limit, symlink handling, exclude
/// patterns, and optional metadata reading.
///
/// - CUE sheets in album directories are parsed first (unless `skip_metadata`).
/// - Files in CUE-handled directories are not re-scanned individually.
/// - Results are sorted by filename for deterministic output.
pub fn scan_dir_with_options(
    base: &Path,
    max_depth: Option<usize>,
    follow_symlinks: bool,
    exclude_patterns: Vec<String>,
    skip_metadata: bool,
) -> Vec<Track> {
    let exts = supported_extensions();
    let mut tracks = Vec::new();
    let mut cue_dirs: HashSet<PathBuf> = HashSet::new();

    // ── Pass 1: CUE-based tracks ────────────────────────────────────────
    if !skip_metadata {
        for entry in walk(base, max_depth, follow_symlinks) {
            let path = entry.path();
            if matches_any_pattern(path, &exclude_patterns) || !path.is_dir() {
                continue;
            }

            let Some(cue_path) = find_cue_in_dir(path) else {
                continue;
            };
            let cue = match parse_cue_file(&cue_path) {
                Ok(c) => c,
                Err(e) => {
                    log::error!(target: "music_chore", "Failed to parse CUE {}: {e}", cue_path.display());
                    continue;
                }
            };

            let dir = path.to_path_buf();
            let dir_artist = infer_artist_from_path(&dir)
                .map(|a| MetadataValue::inferred(a, FOLDER_INFERRED_CONFIDENCE));
            let dir_album = infer_album_from_path(&dir)
                .map(|a| MetadataValue::inferred(a, FOLDER_INFERRED_CONFIDENCE));

            let cue_performer = cue.performer.map(|s| MetadataValue::inferred(s, 1.0));
            let album = cue
                .title
                .map(|s| MetadataValue::inferred(s, 1.0))
                .or(dir_album);
            let year = cue
                .date
                .as_deref()
                .and_then(|s| s.parse::<u32>().ok())
                .map(|y| MetadataValue::cue_inferred(y, 1.0));
            let genre = cue
                .genre
                .as_deref()
                .map(|g| MetadataValue::cue_inferred(g.to_string(), 1.0));

            for ct in cue.tracks {
                let Some(audio_name) = ct.file else { continue };
                let audio_path = dir.join(&audio_name);
                let basic = read_basic_info(&audio_path).ok();

                let artist = ct
                    .performer
                    .map(|s| MetadataValue::cue_inferred(s, 1.0))
                    .or_else(|| cue_performer.clone())
                    .or_else(|| dir_artist.clone());

                let md = TrackMetadata {
                    title: ct.title.map(|s| MetadataValue::cue_inferred(s, 1.0)),
                    artist,
                    album: album.clone(),
                    album_artist: cue_performer.clone(),
                    track_number: Some(MetadataValue::cue_inferred(ct.number, 1.0)),
                    disc_number: None,
                    year: year.clone(),
                    genre: genre.clone(),
                    duration: basic.as_ref().and_then(|b| b.duration.clone()),
                    format: basic.map_or("unknown".to_string(), |b| b.format),
                    path: audio_path.clone(),
                };
                tracks.push(Track::new(audio_path, md));
            }
            cue_dirs.insert(dir);
        }
    }

    // ── Pass 2: individual audio files ──────────────────────────────────
    for entry in walk(base, max_depth, follow_symlinks) {
        let path = entry.path();
        if matches_any_pattern(path, &exclude_patterns)
            || !path.is_file()
            || is_symlink(path)
            || path.parent().is_some_and(|p| cue_dirs.contains(p))
        {
            continue;
        }

        if !is_supported(path, &exts) {
            if has_known_audio_ext(path) {
                warn!(
                    target: "music_chore",
                    "Unsupported audio format: {} (supported: {})",
                    path.display(),
                    exts.iter().cloned().collect::<Vec<_>>().join(", "),
                );
            }
            continue;
        }

        if let Err(e) = validate_file(path) {
            error!(target: "music_chore", "Skipping invalid file {}: {}", path.display(), e);
            continue;
        }

        let md = if skip_metadata {
            inferred_metadata(path)
        } else {
            full_metadata(path)
        };
        tracks.push(Track::new(path.to_path_buf(), md));
    }

    tracks.sort_by(|a, b| a.file_path.file_name().cmp(&b.file_path.file_name()));
    tracks
}

// ── Walk helpers ────────────────────────────────────────────────────────────

/// Constructs a filtered directory walker with the given settings.
fn walk(
    base: &Path,
    max_depth: Option<usize>,
    follow_symlinks: bool,
) -> impl Iterator<Item = walkdir::DirEntry> {
    let mut w = WalkDir::new(base).follow_links(follow_symlinks);
    if let Some(d) = max_depth {
        w = w.max_depth(d + 1); // WalkDir counts the base directory as depth 0
    }
    w.into_iter().filter_map(|e| e.ok())
}

/// Finds the first `.cue` file in a directory (non-recursive).
fn find_cue_in_dir(dir: &Path) -> Option<PathBuf> {
    std::fs::read_dir(dir).ok()?.flatten().find_map(|e| {
        let p = e.path();
        p.extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("cue"))
            .then_some(p)
    })
}
