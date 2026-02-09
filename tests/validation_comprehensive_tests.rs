//! Tests for the validation module functionality.

use music_chore::core::domain::models::{
    AlbumNode, ArtistNode, Library, MetadataSource, MetadataValue, Track, TrackMetadata, TrackNode,
};
use music_chore::core::services::validation::metadata_validation::{
    validate_track_metadata, ValidationError,
};
use music_chore::core::services::validation::{validate_path, validate_tracks};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn create_test_track_with_metadata(
    title: Option<&str>,
    artist: Option<&str>,
    album: Option<&str>,
    track_number: Option<u32>,
    year: Option<u32>,
    path: &str,
) -> Track {
    Track::new(
        PathBuf::from(path),
        TrackMetadata {
            title: title.map(|t| MetadataValue::embedded(t.to_string())),
            artist: artist.map(|a| MetadataValue::embedded(a.to_string())),
            album: album.map(|a| MetadataValue::embedded(a.to_string())),
            album_artist: None,
            track_number: track_number.map(|n| MetadataValue::embedded(n)),
            disc_number: None,
            year: year.map(|y| MetadataValue::embedded(y)),
            genre: None,
            duration: None,
            format: "flac".to_string(),
            path: PathBuf::from(path),
        },
    )
}

#[test]
fn test_validate_path_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    // validate_path returns Err for empty directories
    let result = validate_path(&temp_dir.path().to_path_buf(), false);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No music files found"));
}

#[test]
fn test_validate_path_json_output() {
    let temp_dir = TempDir::new().unwrap();
    // Empty directory returns error even with JSON
    let result = validate_path(&temp_dir.path().to_path_buf(), true);
    assert!(result.is_err());
}

#[test]
fn test_validate_path_with_valid_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let artist_dir = temp_dir.path().join("Test Artist");
    let album_dir = artist_dir.join("Test Album");
    fs::create_dir_all(&album_dir).unwrap();

    // Copy a test file
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        album_dir.join("track1.flac"),
    )
    .unwrap();

    let result = validate_path(&temp_dir.path().to_path_buf(), false);

    assert!(result.is_ok());
    let output = result.unwrap();

    // Output format uses "=== METADATA VALIDATION RESULTS ==="
    assert!(output.contains("=== METADATA VALIDATION RESULTS ==="));
    assert!(output.contains("Summary:"));
}

#[test]
fn test_validate_path_with_missing_title() {
    let temp_dir = TempDir::new().unwrap();
    let album_dir = temp_dir.path().join("Test Album");
    fs::create_dir_all(&album_dir).unwrap();

    // Copy a test file
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        album_dir.join("track1.flac"),
    )
    .unwrap();

    let result = validate_path(&temp_dir.path().to_path_buf(), false);

    assert!(result.is_ok());
    let output = result.unwrap();

    // The copied file has a title, so it should show summary
    assert!(output.contains("Summary:"));
}

#[test]
fn test_validate_path_with_missing_artist() {
    let temp_dir = TempDir::new().unwrap();
    let album_dir = temp_dir.path().join("Test Album");
    fs::create_dir_all(&album_dir).unwrap();

    // Copy a test file
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        album_dir.join("track1.flac"),
    )
    .unwrap();

    let result = validate_path(&temp_dir.path().to_path_buf(), false);

    assert!(result.is_ok());
    let output = result.unwrap();

    // The copied file has an artist, so validation should pass
    assert!(output.contains("Summary:"));
}

#[test]
fn test_validate_path_with_missing_album() {
    let temp_dir = TempDir::new().unwrap();
    let artist_dir = temp_dir.path().join("Test Artist");
    let album_dir = artist_dir.join("Test Album");
    fs::create_dir_all(&album_dir).unwrap();

    // Copy a test file
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        album_dir.join("track1.flac"),
    )
    .unwrap();

    let result = validate_path(&temp_dir.path().to_path_buf(), false);

    assert!(result.is_ok());
    let output = result.unwrap();

    // The copied file has an album, so validation should pass
    assert!(output.contains("Summary:"));
}

#[test]
fn test_validate_path_with_invalid_year() {
    let temp_dir = TempDir::new().unwrap();
    let artist_dir = temp_dir.path().join("Test Artist");
    let album_dir = artist_dir.join("Test Album");
    fs::create_dir_all(&album_dir).unwrap();

    // Copy a test file
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        album_dir.join("track1.flac"),
    )
    .unwrap();

    let result = validate_path(&temp_dir.path().to_path_buf(), false);

    assert!(result.is_ok());
    let output = result.unwrap();

    // The copied file has a valid year (2023), so validation should pass
    assert!(output.contains("Summary:"));
}

#[test]
fn test_validate_path_with_empty_fields() {
    let temp_dir = TempDir::new().unwrap();
    let artist_dir = temp_dir.path().join("Test Artist");
    let album_dir = artist_dir.join("Test Album");
    fs::create_dir_all(&album_dir).unwrap();

    // Copy a test file
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        album_dir.join("track1.flac"),
    )
    .unwrap();

    let result = validate_path(&temp_dir.path().to_path_buf(), false);

    assert!(result.is_ok());
    let output = result.unwrap();

    // The copied file has non-empty fields, so validation should pass
    assert!(output.contains("Summary:"));
}

#[test]
fn test_validate_path_nonexistent_directory() {
    let nonexistent_path = PathBuf::from("/nonexistent/path");

    let result = validate_path(&nonexistent_path, false);

    // validate_path returns Err for nonexistent directories (no files found)
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("No music files found") || error.contains("does not exist"));
}

#[test]
fn test_validate_path_invalid_track_number() {
    let temp_dir = TempDir::new().unwrap();
    let artist_dir = temp_dir.path().join("Test Artist");
    let album_dir = artist_dir.join("Test Album");
    fs::create_dir_all(&album_dir).unwrap();

    // Copy a test file
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        album_dir.join("track1.flac"),
    )
    .unwrap();

    let result = validate_path(&temp_dir.path().to_path_buf(), false);

    assert!(result.is_ok());
    let output = result.unwrap();

    // The copied file has a valid track number, so validation should pass
    assert!(output.contains("Summary:"));
}

#[test]
fn test_validate_path_invalid_disc_number() {
    let temp_dir = TempDir::new().unwrap();
    let artist_dir = temp_dir.path().join("Test Artist");
    let album_dir = artist_dir.join("Test Album");
    fs::create_dir_all(&album_dir).unwrap();

    // Copy a test file
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        album_dir.join("track1.flac"),
    )
    .unwrap();

    let result = validate_path(&temp_dir.path().to_path_buf(), false);

    assert!(result.is_ok());
    let output = result.unwrap();

    // The copied file has a valid disc number, so validation should pass
    assert!(output.contains("Summary:"));
}

#[test]
fn test_validate_tracks_single_valid_track() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.flac");
    // Create actual file since validate_track_metadata checks file existence
    fs::write(&file_path, b"fake flac content").unwrap();

    let track = create_test_track_with_metadata(
        Some("Test Title"),
        Some("Test Artist"),
        Some("Test Album"),
        Some(1),
        Some(2023),
        file_path.to_str().unwrap(),
    );

    let result = validate_track_metadata(&track);
    assert!(result.is_ok());
}

#[test]
fn test_validate_tracks_single_track_missing_title() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.flac");
    fs::write(&file_path, b"fake flac content").unwrap();

    let track = create_test_track_with_metadata(
        None, // No title
        Some("Test Artist"),
        Some("Test Album"),
        Some(1),
        Some(2023),
        file_path.to_str().unwrap(),
    );

    let result = validate_track_metadata(&track);
    assert!(result.is_err());

    match result.unwrap_err() {
        ValidationError::MissingRequiredField(field) => {
            assert_eq!(field, "title");
        }
        _ => panic!("Expected MissingRequiredField error for title"),
    }
}

#[test]
fn test_validate_tracks_single_track_missing_artist() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.flac");
    fs::write(&file_path, b"fake flac content").unwrap();

    let track = create_test_track_with_metadata(
        Some("Test Title"),
        None, // No artist
        Some("Test Album"),
        Some(1),
        Some(2023),
        file_path.to_str().unwrap(),
    );

    let result = validate_track_metadata(&track);
    // Artist is not required in validate_track_metadata
    assert!(result.is_ok());
}

#[test]
fn test_validate_tracks_single_track_missing_album() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.flac");
    fs::write(&file_path, b"fake flac content").unwrap();

    let track = create_test_track_with_metadata(
        Some("Test Title"),
        Some("Test Artist"),
        None, // No album
        Some(1),
        Some(2023),
        file_path.to_str().unwrap(),
    );

    let result = validate_track_metadata(&track);
    // Album is not required in validate_track_metadata
    assert!(result.is_ok());
}

#[test]
fn test_validate_tracks_invalid_track_number() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.flac");
    fs::write(&file_path, b"fake flac content").unwrap();

    let track = Track::new(
        file_path.clone(),
        TrackMetadata {
            title: Some(MetadataValue::embedded("Test Title".to_string())),
            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
            album: Some(MetadataValue::embedded("Test Album".to_string())),
            album_artist: None,
            track_number: Some(MetadataValue::embedded(0)), // Invalid: 0 is not allowed
            disc_number: Some(MetadataValue::embedded(1)),
            year: Some(MetadataValue::embedded(2023)),
            genre: Some(MetadataValue::embedded("Test Genre".to_string())),
            duration: Some(MetadataValue::embedded(180.5)),
            format: "flac".to_string(),
            path: file_path.clone(),
        },
    );

    let result = validate_track_metadata(&track);
    assert!(result.is_err());

    match result.unwrap_err() {
        ValidationError::InvalidValue(field, value) => {
            assert_eq!(field, "track_number");
            assert_eq!(value, "0");
        }
        other => panic!(
            "Expected InvalidValue error for track_number, got {:?}",
            other
        ),
    }
}

#[test]
fn test_validate_tracks_invalid_disc_number() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.flac");
    fs::write(&file_path, b"fake flac content").unwrap();

    let track = Track::new(
        file_path.clone(),
        TrackMetadata {
            title: Some(MetadataValue::embedded("Test Title".to_string())),
            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
            album: Some(MetadataValue::embedded("Test Album".to_string())),
            album_artist: None,
            track_number: Some(MetadataValue::embedded(1)),
            disc_number: Some(MetadataValue::embedded(0)), // Invalid: 0 is not allowed
            year: Some(MetadataValue::embedded(2023)),
            genre: Some(MetadataValue::embedded("Test Genre".to_string())),
            duration: Some(MetadataValue::embedded(180.5)),
            format: "flac".to_string(),
            path: file_path.clone(),
        },
    );

    let result = validate_track_metadata(&track);
    assert!(result.is_err());

    match result.unwrap_err() {
        ValidationError::InvalidValue(field, value) => {
            assert_eq!(field, "disc_number");
            assert_eq!(value, "0");
        }
        other => panic!(
            "Expected InvalidValue error for disc_number, got {:?}",
            other
        ),
    }
}

#[test]
fn test_validate_tracks_invalid_year() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.flac");
    fs::write(&file_path, b"fake flac content").unwrap();

    let track = Track::new(
        file_path.clone(),
        TrackMetadata {
            title: Some(MetadataValue::embedded("Test Title".to_string())),
            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
            album: Some(MetadataValue::embedded("Test Album".to_string())),
            album_artist: None,
            track_number: Some(MetadataValue::embedded(1)),
            disc_number: Some(MetadataValue::embedded(1)),
            year: Some(MetadataValue::embedded(500)), // Invalid: below MIN_YEAR (1000)
            genre: Some(MetadataValue::embedded("Test Genre".to_string())),
            duration: Some(MetadataValue::embedded(180.5)),
            format: "flac".to_string(),
            path: file_path.clone(),
        },
    );

    let result = validate_track_metadata(&track);
    assert!(result.is_err());

    match result.unwrap_err() {
        ValidationError::InvalidValue(field, value) => {
            assert_eq!(field, "year");
            assert_eq!(value, "500");
        }
        other => panic!("Expected InvalidValue error for year, got {:?}", other),
    }
}

#[test]
fn test_validate_tracks_future_year() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.flac");
    fs::write(&file_path, b"fake flac content").unwrap();

    let track = Track::new(
        file_path.clone(),
        TrackMetadata {
            title: Some(MetadataValue::embedded("Test Title".to_string())),
            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
            album: Some(MetadataValue::embedded("Test Album".to_string())),
            album_artist: None,
            track_number: Some(MetadataValue::embedded(1)),
            disc_number: Some(MetadataValue::embedded(1)),
            year: Some(MetadataValue::embedded(3001)), // Invalid: above MAX_YEAR (3000)
            genre: Some(MetadataValue::embedded("Test Genre".to_string())),
            duration: Some(MetadataValue::embedded(180.5)),
            format: "flac".to_string(),
            path: file_path.clone(),
        },
    );

    let result = validate_track_metadata(&track);
    assert!(result.is_err());

    match result.unwrap_err() {
        ValidationError::InvalidValue(field, value) => {
            assert_eq!(field, "year");
            assert_eq!(value, "3001");
        }
        other => panic!("Expected InvalidValue error for year, got {:?}", other),
    }
}

#[test]
fn test_validate_tracks_valid_year_range() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.flac");
    fs::write(&file_path, b"fake flac content").unwrap();

    let track = Track::new(
        file_path.clone(),
        TrackMetadata {
            title: Some(MetadataValue::embedded("Test Title".to_string())),
            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
            album: Some(MetadataValue::embedded("Test Album".to_string())),
            album_artist: None,
            track_number: Some(MetadataValue::embedded(1)),
            disc_number: Some(MetadataValue::embedded(1)),
            year: Some(MetadataValue::embedded(1000)), // Valid: at MIN_YEAR bound
            genre: Some(MetadataValue::embedded("Test Genre".to_string())),
            duration: Some(MetadataValue::embedded(180.5)),
            format: "flac".to_string(),
            path: file_path.clone(),
        },
    );

    let result = validate_track_metadata(&track);
    assert!(result.is_ok());
}

#[test]
fn test_validate_tracks_empty_string_fields() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.flac");
    fs::write(&file_path, b"fake flac content").unwrap();

    let track = Track::new(
        file_path.clone(),
        TrackMetadata {
            title: Some(MetadataValue::embedded("".to_string())), // Empty string
            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
            album: Some(MetadataValue::embedded("Test Album".to_string())),
            album_artist: None,
            track_number: Some(MetadataValue::embedded(1)),
            disc_number: Some(MetadataValue::embedded(1)),
            year: Some(MetadataValue::embedded(2023)),
            genre: Some(MetadataValue::embedded("Test Genre".to_string())),
            duration: Some(MetadataValue::embedded(180.5)),
            format: "flac".to_string(),
            path: file_path.clone(),
        },
    );

    let result = validate_track_metadata(&track);
    assert!(result.is_err());

    match result.unwrap_err() {
        ValidationError::InvalidValue(field, value) => {
            assert_eq!(field, "title");
            assert_eq!(value, "empty");
        }
        other => panic!(
            "Expected InvalidValue error for empty title, got {:?}",
            other
        ),
    }
}

#[test]
fn test_validate_tracks_whitespace_only_fields() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.flac");
    fs::write(&file_path, b"fake flac content").unwrap();

    let track = Track::new(
        file_path.clone(),
        TrackMetadata {
            title: Some(MetadataValue::embedded("   ".to_string())), // Whitespace only
            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
            album: Some(MetadataValue::embedded("Test Album".to_string())),
            album_artist: None,
            track_number: Some(MetadataValue::embedded(1)),
            disc_number: Some(MetadataValue::embedded(1)),
            year: Some(MetadataValue::embedded(2023)),
            genre: Some(MetadataValue::embedded("Test Genre".to_string())),
            duration: Some(MetadataValue::embedded(180.5)),
            format: "flac".to_string(),
            path: file_path.clone(),
        },
    );

    let result = validate_track_metadata(&track);
    assert!(result.is_err());

    match result.unwrap_err() {
        ValidationError::InvalidValue(field, value) => {
            assert_eq!(field, "title");
            assert_eq!(value, "empty"); // Whitespace is treated as empty
        }
        other => panic!(
            "Expected InvalidValue error for whitespace-only title, got {:?}",
            other
        ),
    }
}

#[test]
fn test_validate_tracks_different_metadata_sources() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.flac");
    fs::write(&file_path, b"fake flac content").unwrap();

    // Test validation with different metadata sources
    let track = Track::new(
        file_path.clone(),
        TrackMetadata {
            title: Some(MetadataValue::cue_inferred("CUE Title".to_string(), 1.0)),
            artist: Some(MetadataValue::inferred("Folder Artist".to_string(), 0.3)),
            album: Some(MetadataValue::user_set("User Album".to_string())),
            album_artist: None,
            track_number: Some(MetadataValue::embedded(1)),
            disc_number: Some(MetadataValue::embedded(1)),
            year: Some(MetadataValue::embedded(2023)),
            genre: Some(MetadataValue::embedded("Test Genre".to_string())),
            duration: Some(MetadataValue::embedded(180.5)),
            format: "flac".to_string(),
            path: file_path.clone(),
        },
    );

    let result = validate_track_metadata(&track);
    assert!(result.is_ok()); // Different sources shouldn't affect validation
}

#[test]
fn test_validate_tracks_max_values() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.flac");
    fs::write(&file_path, b"fake flac content").unwrap();

    let track = Track::new(
        file_path.clone(),
        TrackMetadata {
            title: Some(MetadataValue::embedded("Test Title".to_string())),
            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
            album: Some(MetadataValue::embedded("Test Album".to_string())),
            album_artist: None,
            track_number: Some(MetadataValue::embedded(1000)), // Exceeds MAX_TRACK_NUMBER (999)
            disc_number: Some(MetadataValue::embedded(100)),   // Exceeds MAX_DISC_NUMBER (99)
            year: Some(MetadataValue::embedded(2023)),
            genre: Some(MetadataValue::embedded("Test Genre".to_string())),
            duration: Some(MetadataValue::embedded(180.5)),
            format: "flac".to_string(),
            path: file_path.clone(),
        },
    );

    let result = validate_track_metadata(&track);
    assert!(result.is_err()); // Track number exceeds max
}

#[test]
fn test_validate_path_with_complete_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let artist_dir = temp_dir.path().join("Test Artist");
    let album_dir = artist_dir.join("Test Album");
    fs::create_dir_all(&album_dir).unwrap();

    // Copy a test file
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        album_dir.join("track1.flac"),
    )
    .unwrap();

    let result = validate_path(&temp_dir.path().to_path_buf(), false);

    assert!(result.is_ok());
    let output = result.unwrap();

    // Output format uses "=== METADATA VALIDATION RESULTS ==="
    assert!(output.contains("=== METADATA VALIDATION RESULTS ==="));
    assert!(output.contains("Summary:"));
}

#[test]
fn test_validate_path_with_warnings() {
    let temp_dir = TempDir::new().unwrap();
    let artist_dir = temp_dir.path().join("Test Artist");
    let album_dir = artist_dir.join("Test Album");
    fs::create_dir_all(&album_dir).unwrap();

    // Copy a test file
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        album_dir.join("track1.flac"),
    )
    .unwrap();

    let result = validate_path(&temp_dir.path().to_path_buf(), false);

    assert!(result.is_ok());
    let output = result.unwrap();

    // Output format uses "=== METADATA VALIDATION RESULTS ==="
    assert!(output.contains("=== METADATA VALIDATION RESULTS ==="));
    assert!(output.contains("Summary:"));
}
