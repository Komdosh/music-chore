use crate::services::scanner::scan_with_duplicates;
use serde_json::to_string_pretty;
use std::fmt::Write;
use std::path::Path;

pub fn find_duplicates(path: &Path, json: bool) -> Result<String, String> {
    let (tracks, duplicates) = scan_with_duplicates(path);

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
        return Ok(to_string_pretty(&duplicates)
            .unwrap_or_else(|e| format!("Error serializing to JSON: {}", e)));
    }

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
            writeln!(out, "  {}", track.file_path.display()).unwrap();
        }

        writeln!(out).unwrap();
    }

    Ok(out)
}
