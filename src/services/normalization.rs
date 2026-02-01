//! Text normalization services.

use crate::domain::models::{OperationResult, Track};
use crate::infrastructure::formats;
use std::path::Path;

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

/// Normalize track titles to title case
pub fn normalize_track_titles(path: &Path) -> Result<Vec<OperationResult>, String> {
    normalize_track_titles_with_options(path, false)
}

/// Normalize track titles to title case with options
pub fn normalize_track_titles_with_options(
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
        let tracks = crate::infrastructure::scanner::scan_dir(path);
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
