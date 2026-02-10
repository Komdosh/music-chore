use crate::core::services::scanner::scan_with_duplicates;
use serde_json::to_string_pretty;
use std::fmt::Write;
use std::path::Path;

pub fn find_duplicates(path: &Path, json: bool, verbose: bool) -> Result<String, String> {
    let (tracks, duplicates) = scan_with_duplicates(path, verbose);

    if tracks.is_empty() {
        return Err(format!(
            "No music files found in directory: {}",
            path.display()
        ));
    }

    if duplicates.is_empty() {
        return Err("No duplicate tracks found.".to_string());
    }

    if json {
        match to_string_pretty(&duplicates) {
            Ok(s) => Ok(s),
            Err(e) => Err(format!("Error serializing to JSON: {}", e)),
        }
    } else {
        let mut out = String::new();

        writeln!(out, "Found {} duplicate groups:\n", duplicates.len()).unwrap();

        for (i, duplicate_group) in duplicates.iter().enumerate() {
            writeln!(
                out,
                "Duplicate Group {} ({} files):",
                i + 1,
                duplicate_group.len()
            )
            .unwrap();

            for track in duplicate_group {
                if verbose {
                    let artist = track.metadata.artist.as_ref().map(|v| v.value.as_str()).unwrap_or("Unknown Artist");
                    let album = track.metadata.album.as_ref().map(|v| v.value.as_str()).unwrap_or("Unknown Album");
                    let title = track.metadata.title.as_ref().map(|v| v.value.as_str()).unwrap_or("Unknown Title");
                    let duration = track.metadata.duration.as_ref().map(|v| format!("{:.0}s", v.value)).unwrap_or_else(|| "unknown".to_string());
                    let checksum = track.checksum.as_deref().unwrap_or("unknown");
                    
                    writeln!(
                        out,
                        "  {} - {} - {} [{}] ({}) [sha256: {}]",
                        artist, album, title, duration, track.file_path.display(), checksum
                    )
                    .unwrap();
                } else {
                    writeln!(out, "  {}", track.file_path.display()).unwrap();
                }
            }

            writeln!(out).unwrap();
        }

        Ok(out)
    }
}
