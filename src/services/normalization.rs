//! Text normalization services.

use crate::domain::models::{OperationResult, Track};
use crate::services::formats;
use std::path::{Path, PathBuf};

/// Convert string to title case (first letter of each word capitalized)
pub fn to_title_case(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut capitalize_next = true;

    for c in input.chars() {
        if c.is_whitespace() || c == '-' || c == '_' {
            capitalize_next = true;
            result.push(c);
        } else if capitalize_next {
            for uppercase_char in c.to_uppercase() {
                result.push(uppercase_char);
            }
            capitalize_next = false;
        } else {
            for lowercase_char in c.to_lowercase() {
                result.push(lowercase_char);
            }
        }
    }

    result
}

/// Normalize track titles to title case with options
pub fn normalize(path: PathBuf, dry_run: bool) -> Result<String, String> {
    let mut out = String::new();

    match normalize_track_titles_with_options(&path, dry_run) {
        Ok(results) => {
            for result in results {
                match result {
                    OperationResult::Updated {
                        track,
                        old_title,
                        new_title,
                    } => {
                        if dry_run {
                            out.push_str(&format!(
                                "DRY RUN: Would normalize '{}' -> '{}' in {}\n",
                                track.file_path.display(),
                                old_title,
                                new_title
                            ));
                        } else {
                            out.push_str(&format!(
                                "NORMALIZED: '{}' -> '{}' in {}\n",
                                track.file_path.display(),
                                old_title,
                                new_title
                            ));
                        }
                    }

                    OperationResult::NoChange { track } => {
                        if !dry_run {
                            out.push_str(&format!(
                                "NO CHANGE: Title already title case in {}\n",
                                track.file_path.display()
                            ));
                        }
                    }

                    OperationResult::Error { track, error } => {
                        out.push_str(&format!(
                            "ERROR: {} in {}\n",
                            error,
                            track.file_path.display()
                        ));
                    }
                }
            }

            Ok(out)
        }

        Err(e) => {
            Err(format!("Error normalizing titles: {}\n", e))
        }
    }
}
fn normalize_track_titles_with_options(
    path: &Path,
    dry_run: bool,
) -> Result<Vec<OperationResult>, String> {
    let mut results = Vec::new();

    // Check if path is a file or directory
    if path.is_file() {
        // Single file
        let track = formats::read_metadata(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
        results.push(normalize_single_track(track, dry_run));
    } else if path.is_dir() {
        // Directory - scan for supported audio files
        let tracks = crate::services::scanner::scan_dir(path);
        for track in tracks {
            results.push(normalize_single_track(track, dry_run));
        }
    } else {
        return Err(format!("Path does not exist: {}", path.display()));
    }

    Ok(results)
}

/// Normalize a single track's title
fn normalize_single_track(track: Track, dry_run: bool) -> OperationResult {
    let current_title = match &track.metadata.title {
        Some(title) => &title.value,
        None => {
            return OperationResult::Error {
                track,
                error: "No title found".to_string(),
            }
        }
    };

    let normalized_title = to_title_case(current_title);
    let old_title = current_title.clone();

    // Check if title needs to be changed
    if current_title == &normalized_title {
        return OperationResult::NoChange { track };
    }

    if dry_run {
        // Just return what would be changed
        OperationResult::Updated {
            track,
            old_title,
            new_title: normalized_title,
        }
    } else {
        // Actually update the metadata
        // TODO: Implement actual metadata writing
        // For now, just return the operation result
        OperationResult::Updated {
            track,
            old_title,
            new_title: normalized_title,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_title_case() {
        assert_eq!(to_title_case("hello world"), "Hello World");
        assert_eq!(to_title_case("HELLO WORLD"), "Hello World");
        assert_eq!(to_title_case("hello-world_test"), "Hello-World_Test");
        assert_eq!(to_title_case("  leading  spaces  "), "  Leading  Spaces  ");
        assert_eq!(to_title_case(""), "");
        assert_eq!(to_title_case("a"), "A");
        assert_eq!(to_title_case("already Title Case"), "Already Title Case");
    }
}
