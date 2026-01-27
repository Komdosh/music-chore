//! Validation logic for music metadata.

use crate::domain::metadata::Metadata;
use crate::domain::Provenance;

/// Validates metadata for consistency and correctness
pub fn validate_metadata(metadata: &Metadata) -> Vec<String> {
    let mut errors = Vec::new();

    // Check that required fields are present
    if metadata.inferred.get("artist").is_none() && metadata.embedded.get("artist").is_none() {
        errors.push("Missing artist information".to_string());
    }

    if metadata.inferred.get("title").is_none() && metadata.embedded.get("title").is_none() {
        errors.push("Missing title information".to_string());
    }

    // Check for inconsistent data
    if let (Some(artist1), Some(artist2)) = (
        metadata.inferred.get("artist"),
        metadata.embedded.get("artist"),
    ) {
        if artist1 != artist2 {
            errors.push("Inconsistent artist information between inferred and embedded metadata".to_string());
        }
    }

    errors
}

/// Validates that a metadata field has a valid value
pub fn is_valid_field(field: &str, value: &str) -> bool {
    match field {
        "artist" | "album" | "title" => !value.trim().is_empty(),
        "year" => {
            // Year should be a 4-digit number or empty
            value.is_empty() || value.chars().all(|c| c.is_ascii_digit()) && value.len() == 4
        }
        _ => true, // Other fields are allowed to be empty
    }
}

/// Validates that a track has required information
pub fn validate_track(track: &crate::domain::Track) -> Vec<String> {
    let mut errors = Vec::new();

    if track.title.is_empty() {
        errors.push("Track title is empty".to_string());
    }

    if track.file_path.is_empty() {
        errors.push("Track file path is empty".to_string());
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_metadata() {
        let metadata = Metadata::new();
        let errors = validate_metadata(&metadata);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_is_valid_field() {
        assert!(is_valid_field("artist", "The Beatles"));
        assert!(is_valid_field("album", "Abbey Road"));
        assert!(is_valid_field("title", "Come Together"));
        assert!(is_valid_field("year", "1969"));
        assert!(is_valid_field("year", ""));
        assert!(!is_valid_field("year", "69"));
        assert!(!is_valid_field("artist", ""));
    }

    #[test]
    fn test_validate_track() {
        let track = crate::domain::Track::new(
            "Come Together".to_string(),
            Some(1),
            Some(259),
            "/path/to/file.flac".to_string(),
            Provenance::Inferred,
        );
        let errors = validate_track(&track);
        assert!(errors.is_empty());
    }
}