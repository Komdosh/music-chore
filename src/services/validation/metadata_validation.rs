//! Metadata schema validation module

use crate::domain::models::{Track, TrackMetadata, MetadataValue};
use std::path::Path;

/// Errors that can occur during metadata validation
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// Missing required field
    MissingRequiredField(String),
    /// Invalid value for a field
    InvalidValue(String, String),
    /// Field value doesn't match expected format
    FormatMismatch(String, String),
    /// IO error during validation
    IoError(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::MissingRequiredField(field) => write!(f, "Missing required field: {}", field),
            ValidationError::InvalidValue(field, value) => write!(f, "Invalid value for {}: {}", field, value),
            ValidationError::FormatMismatch(field, format) => write!(f, "Format mismatch for {}: {}", field, format),
            ValidationError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validate a track's metadata against the schema
pub fn validate_track_metadata(track: &Track) -> Result<(), ValidationError> {
    let metadata = &track.metadata;

    // Validate required fields - for now, we'll require at least a title
    if metadata.title.is_none() {
        return Err(ValidationError::MissingRequiredField("title".to_string()));
    }

    // Validate format field
    if metadata.format.is_empty() {
        return Err(ValidationError::MissingRequiredField("format".to_string()));
    }

    // Validate path
    if !track.file_path.exists() {
        return Err(ValidationError::IoError(format!("File does not exist: {}", track.file_path.display())));
    }

    // Validate numeric fields are within reasonable bounds
    if let Some(ref track_number) = metadata.track_number {
        if track_number.value == 0 || track_number.value > 999 {
            return Err(ValidationError::InvalidValue(
                "track_number".to_string(),
                track_number.value.to_string()
            ));
        }
    }

    if let Some(ref disc_number) = metadata.disc_number {
        if disc_number.value == 0 || disc_number.value > 99 {
            return Err(ValidationError::InvalidValue(
                "disc_number".to_string(),
                disc_number.value.to_string()
            ));
        }
    }

    if let Some(ref year) = metadata.year {
        // Reasonable range for years
        if year.value < 1000 || year.value > 3000 {
            return Err(ValidationError::InvalidValue(
                "year".to_string(),
                year.value.to_string()
            ));
        }
    }

    // Validate string fields are not empty when present
    if let Some(ref title) = metadata.title {
        if title.value.trim().is_empty() {
            return Err(ValidationError::InvalidValue("title".to_string(), "empty".to_string()));
        }
    }

    if let Some(ref artist) = metadata.artist {
        if artist.value.trim().is_empty() {
            return Err(ValidationError::InvalidValue("artist".to_string(), "empty".to_string()));
        }
    }

    if let Some(ref album) = metadata.album {
        if album.value.trim().is_empty() {
            return Err(ValidationError::InvalidValue("album".to_string(), "empty".to_string()));
        }
    }

    if let Some(ref genre) = metadata.genre {
        if genre.value.trim().is_empty() {
            return Err(ValidationError::InvalidValue("genre".to_string(), "empty".to_string()));
        }
    }

    // Validate duration if present
    if let Some(ref duration) = metadata.duration {
        if duration.value < 0.0 || duration.value > 36000.0 { // Max 10 hours
            return Err(ValidationError::InvalidValue(
                "duration".to_string(),
                duration.value.to_string()
            ));
        }
    }

    Ok(())
}

/// Validate metadata when reading from a file
pub fn validate_metadata_on_read(path: &Path, metadata: &TrackMetadata) -> Result<(), ValidationError> {
    // Create a temporary track for validation purposes
    let temp_track = Track::new(path.to_path_buf(), metadata.clone());
    validate_track_metadata(&temp_track)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{MetadataValue, MetadataSource, TrackMetadata};
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_validate_track_metadata_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.flac");
        std::fs::write(&file_path, b"fake flac content").unwrap();

        let metadata = TrackMetadata {
            title: Some(MetadataValue::embedded("Test Title".to_string())),
            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
            album: Some(MetadataValue::embedded("Test Album".to_string())),
            album_artist: None,
            track_number: Some(MetadataValue::embedded(1)),
            disc_number: Some(MetadataValue::embedded(1)),
            year: Some(MetadataValue::embedded(2023)),
            genre: Some(MetadataValue::embedded("Test Genre".to_string())),
            duration: Some(MetadataValue::embedded(180.0)),
            format: "flac".to_string(),
            path: file_path.clone(),
        };

        let track = Track::new(file_path, metadata);
        assert!(validate_track_metadata(&track).is_ok());
    }

    #[test]
    fn test_validate_track_metadata_missing_required_fields() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.flac");
        std::fs::write(&file_path, b"fake flac content").unwrap();

        let metadata = TrackMetadata {
            title: None, // Missing required field
            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
            album: Some(MetadataValue::embedded("Test Album".to_string())),
            album_artist: None,
            track_number: Some(MetadataValue::embedded(1)),
            disc_number: Some(MetadataValue::embedded(1)),
            year: Some(MetadataValue::embedded(2023)),
            genre: Some(MetadataValue::embedded("Test Genre".to_string())),
            duration: Some(MetadataValue::embedded(180.0)),
            format: "flac".to_string(),
            path: file_path.clone(),
        };

        let track = Track::new(file_path, metadata);
        assert!(validate_track_metadata(&track).is_err());
    }

    #[test]
    fn test_validate_track_metadata_invalid_values() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.flac");
        std::fs::write(&file_path, b"fake flac content").unwrap();

        let metadata = TrackMetadata {
            title: Some(MetadataValue::embedded("Test Title".to_string())),
            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
            album: Some(MetadataValue::embedded("Test Album".to_string())),
            album_artist: None,
            track_number: Some(MetadataValue::embedded(1000)), // Invalid: > 999
            disc_number: Some(MetadataValue::embedded(1)),
            year: Some(MetadataValue::embedded(2023)),
            genre: Some(MetadataValue::embedded("Test Genre".to_string())),
            duration: Some(MetadataValue::embedded(180.0)),
            format: "flac".to_string(),
            path: file_path.clone(),
        };

        let track = Track::new(file_path, metadata);
        assert!(validate_track_metadata(&track).is_err());
    }

    #[test]
    fn test_validate_track_metadata_empty_strings() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.flac");
        std::fs::write(&file_path, b"fake flac content").unwrap();

        let metadata = TrackMetadata {
            title: Some(MetadataValue::embedded("".to_string())), // Empty string
            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
            album: Some(MetadataValue::embedded("Test Album".to_string())),
            album_artist: None,
            track_number: Some(MetadataValue::embedded(1)),
            disc_number: Some(MetadataValue::embedded(1)),
            year: Some(MetadataValue::embedded(2023)),
            genre: Some(MetadataValue::embedded("Test Genre".to_string())),
            duration: Some(MetadataValue::embedded(180.0)),
            format: "flac".to_string(),
            path: file_path.clone(),
        };

        let track = Track::new(file_path, metadata);
        assert!(validate_track_metadata(&track).is_err());
    }
}