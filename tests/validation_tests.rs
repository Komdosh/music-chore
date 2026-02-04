//! Unit tests for validation functionality  
//! Tests the CLI validation functions that are reused by MCP

use music_chore::core::services::validation::validate_tracks;
use music_chore::{MetadataValue, Track, TrackMetadata};
use std::path::PathBuf;

#[test]
fn test_validate_empty_tracks_list() {
    let result = validate_tracks(vec![]);

    assert!(result.valid);
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.warnings.len(), 0);
    assert_eq!(result.summary.total_files, 0);
    assert_eq!(result.summary.valid_files, 0);
}

#[test]
fn test_validate_perfect_tracks() {
    let tracks = vec![
        Track {
            file_path: PathBuf::from("/test/track1.flac"),
            checksum: None,
            metadata: create_basic_metadata("Track 1", 1),
        },
        Track {
            file_path: PathBuf::from("/test/track2.flac"),
            checksum: None,
            metadata: create_basic_metadata("Track 2", 2),
        },
    ];

    let result = validate_tracks(tracks);

    assert!(result.valid);
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.warnings.len(), 0);
    assert_eq!(result.summary.total_files, 2);
    assert_eq!(result.summary.valid_files, 2);
    assert_eq!(result.summary.files_with_errors, 0);
    assert_eq!(result.summary.files_with_warnings, 0);
}

#[test]
fn test_validate_missing_metadata() {
    let tracks = vec![crate::Track {
        file_path: PathBuf::from("/test/track1.flac"),
        checksum: None,
        metadata: TrackMetadata {
            title: None,  // Missing title (error)
            artist: None, // Missing artist (error)
            album: None,  // Missing album (error)
            album_artist: None,
            year: None,         // Missing year (warning)
            track_number: None, // Missing track number (warning)
            disc_number: None,
            genre: None,
            duration: None,
            format: "flac".to_string(),
            path: PathBuf::from("/test/track1.flac"),
        },
    }];

    let result = validate_tracks(tracks);

    assert!(!result.valid);
    assert_eq!(result.errors.len(), 3); // Missing title, artist, album
    assert_eq!(result.warnings.len(), 2); // Missing year, track number
    assert_eq!(result.summary.total_files, 1);
    assert_eq!(result.summary.valid_files, 0);
    assert_eq!(result.summary.files_with_errors, 1);
    assert_eq!(result.summary.files_with_warnings, 1);

    // Check specific errors
    let error_fields: Vec<_> = result.errors.iter().map(|e| &e.field).collect();
    assert!(error_fields.contains(&&"title".to_string()));
    assert!(error_fields.contains(&&"artist".to_string()));
    assert!(error_fields.contains(&&"album".to_string()));

    // Check specific warnings
    let warning_fields: Vec<_> = result.warnings.iter().map(|w| &w.field).collect();
    assert!(warning_fields.contains(&&"year".to_string()));
    assert!(warning_fields.contains(&&"track_number".to_string()));
}

#[test]
fn test_validate_unusual_values() {
    let tracks = vec![crate::Track {
        file_path: PathBuf::from("/test/unusual.flac"),
        checksum: None,
        metadata: TrackMetadata {
            title: Some(MetadataValue::embedded("Valid Title".to_string())),
            artist: Some(MetadataValue::embedded("Valid Artist".to_string())),
            album: Some(MetadataValue::embedded("Valid Album".to_string())),
            album_artist: None,
            year: Some(MetadataValue::embedded(1800)), // Unusual year
            track_number: Some(MetadataValue::embedded(0)), // Unusual track number
            disc_number: None,
            genre: None,
            duration: None,
            format: "flac".to_string(),
            path: PathBuf::from("/test/unusual.flac"),
        },
    }];

    let result = validate_tracks(tracks);

    assert!(result.valid); // Still valid because these are warnings, not errors
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.warnings.len(), 2); // Unusual year and track number
    assert_eq!(result.summary.files_with_warnings, 1);

    // Check specific warnings
    let warning_fields: Vec<_> = result.warnings.iter().map(|w| &w.field).collect();
    assert!(warning_fields.contains(&&"year".to_string()));
    assert!(warning_fields.contains(&&"track_number".to_string()));
}

#[test]
fn test_validate_mixed_quality() {
    let tracks = vec![
        // Good track
        Track {
            file_path: PathBuf::from("/test/good.flac"),
            checksum: None,
            metadata: create_basic_metadata("Good Track", 1),
        },
        // Bad track - missing required fields
        Track {
            file_path: PathBuf::from("/test/bad.flac"),
            checksum: None,
            metadata: TrackMetadata {
                title: None, // Missing title (error)
                artist: Some(MetadataValue::embedded("Artist".to_string())),
                album: Some(MetadataValue::embedded("Album".to_string())),
                album_artist: None,
                year: None,         // Missing year (warning)
                track_number: None, // Missing track number (warning)
                disc_number: None,
                genre: None,
                duration: None,
                format: "flac".to_string(),
                path: PathBuf::from("/test/bad.flac"),
            },
        },
    ];

    let result = validate_tracks(tracks);

    assert!(!result.valid); // Not valid due to missing title
    assert_eq!(result.errors.len(), 1); // Missing title
    assert_eq!(result.warnings.len(), 2); // Missing year, track number
    assert_eq!(result.summary.total_files, 2);
    assert_eq!(result.summary.valid_files, 1); // Only the good track
    assert_eq!(result.summary.files_with_errors, 1);
    assert_eq!(result.summary.files_with_warnings, 1);
}

// Helper function to create basic metadata for testing
fn create_basic_metadata(title: &str, track_number: u32) -> TrackMetadata {
    TrackMetadata {
        title: Some(MetadataValue::embedded(title.to_string())),
        artist: Some(MetadataValue::embedded("Test Artist".to_string())),
        album: Some(MetadataValue::embedded("Test Album".to_string())),
        album_artist: Some(MetadataValue::embedded("Test Artist".to_string())),
        year: Some(MetadataValue::embedded(2023)),
        track_number: Some(MetadataValue::embedded(track_number)),
        disc_number: None,
        genre: Some(MetadataValue::embedded("Rock".to_string())),
        duration: Some(MetadataValue::embedded(180.0)),
        format: "flac".to_string(),
        path: PathBuf::from("/test"),
    }
}
