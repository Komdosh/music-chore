use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crate::domain::track::{Track, Provenance};

/// Recursively scans a directory for supported music files and returns a vector of `Track`.
///
/// Currently only `.flac` files are considered supported.
pub fn scan_dir(base: &Path) -> Vec<Track> {
    let mut tracks: Vec<Track> = Vec::new();

    for entry in WalkDir::new(base).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if ext.eq_ignore_ascii_case("flac") {
                    let title = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string();
                    let file_path = path
                        .canonicalize()
                        .unwrap_or_else(|_| path.to_path_buf())
                        .to_string_lossy()
                        .to_string();
                    let track = Track::new(title, None, None, file_path, Provenance::Inferred);
                    tracks.push(track);
                }
            }
        }
    }

    // Deterministic order by absolute path
    tracks.sort_by(|a, b| a.file_path.cmp(&b.file_path));
    tracks
}
