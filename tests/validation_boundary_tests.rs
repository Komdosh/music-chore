//! Tests for validation module boundary conditions and edge cases

use music_chore::core::domain::models::{MetadataSource, MetadataValue, Track, TrackMetadata};
use music_chore::core::services::validation::validate_tracks;
use std::path::PathBuf;

fn create_test_track_with_year(year: Option<u32>) -> Track {
    Track::new(
        PathBuf::from("test.flac"),
        TrackMetadata {
            title: Some(MetadataValue::embedded("Test Title".to_string())),
            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
            album: Some(MetadataValue::embedded("Test Album".to_string())),
            album_artist: None,
            track_number: Some(MetadataValue::embedded(1)),
            disc_number: None,
            year: year.map(|y| MetadataValue::embedded(y)),
            genre: None,
            duration: None,
            format: "flac".to_string(),
            path: PathBuf::from("test.flac"),
        },
    )
}

fn create_test_track_with_track_number(track_number: Option<u32>) -> Track {
    Track::new(
        PathBuf::from("test.flac"),
        TrackMetadata {
            title: Some(MetadataValue::embedded("Test Title".to_string())),
            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
            album: Some(MetadataValue::embedded("Test Album".to_string())),
            album_artist: None,
            track_number: track_number.map(|t| MetadataValue::embedded(t)),
            disc_number: None,
            year: Some(MetadataValue::embedded(2020)),
            genre: None,
            duration: None,
            format: "flac".to_string(),
            path: PathBuf::from("test.flac"),
        },
    )
}

fn create_test_track_with_disc_number(disc_number: Option<u32>) -> Track {
    Track::new(
        PathBuf::from("test.flac"),
        TrackMetadata {
            title: Some(MetadataValue::embedded("Test Title".to_string())),
            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
            album: Some(MetadataValue::embedded("Test Album".to_string())),
            album_artist: None,
            track_number: Some(MetadataValue::embedded(1)),
            disc_number: disc_number.map(|d| MetadataValue::embedded(d)),
            year: Some(MetadataValue::embedded(2020)),
            genre: None,
            duration: None,
            format: "flac".to_string(),
            path: PathBuf::from("test.flac"),
        },
    )
}

#[test]
fn test_validate_year_boundary_minimum_valid() {
    // Year 1900 should be valid (boundary)
    let track = create_test_track_with_year(Some(1900));
    let result = validate_tracks(vec![track]);

    assert!(result.valid);
    assert!(result.warnings.is_empty());
}

#[test]
fn test_validate_year_boundary_below_minimum() {
    // Year 1899 should produce warning
    let track = create_test_track_with_year(Some(1899));
    let result = validate_tracks(vec![track]);

    assert!(result.valid); // Still valid, just warning
    assert!(!result.warnings.is_empty());

    let year_warnings: Vec<_> = result
        .warnings
        .iter()
        .filter(|w| w.field == "year")
        .collect();
    assert!(!year_warnings.is_empty());
}

#[test]
fn test_validate_year_boundary_maximum_valid() {
    // Year 2100 should be valid (boundary)
    let track = create_test_track_with_year(Some(2100));
    let result = validate_tracks(vec![track]);

    assert!(result.valid);
    assert!(result.warnings.is_empty());
}

#[test]
fn test_validate_year_boundary_above_maximum() {
    // Year 2101 should produce warning
    let track = create_test_track_with_year(Some(2101));
    let result = validate_tracks(vec![track]);

    assert!(result.valid); // Still valid, just warning
    assert!(!result.warnings.is_empty());

    let year_warnings: Vec<_> = result
        .warnings
        .iter()
        .filter(|w| w.field == "year")
        .collect();
    assert!(!year_warnings.is_empty());
}

#[test]
fn test_validate_year_zero() {
    // Year 0 should produce warning
    let track = create_test_track_with_year(Some(0));
    let result = validate_tracks(vec![track]);

    assert!(result.valid);
    assert!(!result.warnings.is_empty());
}

#[test]
fn test_validate_year_far_future() {
    // Year 9999 should produce warning
    let track = create_test_track_with_year(Some(9999));
    let result = validate_tracks(vec![track]);

    assert!(result.valid);
    assert!(!result.warnings.is_empty());
}

#[test]
fn test_validate_track_number_zero() {
    // Track number 0 should produce warning
    let track = create_test_track_with_track_number(Some(0));
    let result = validate_tracks(vec![track]);

    assert!(result.valid);
    assert!(!result.warnings.is_empty());

    let track_warnings: Vec<_> = result
        .warnings
        .iter()
        .filter(|w| w.field == "track_number")
        .collect();
    assert!(!track_warnings.is_empty());
}

#[test]
fn test_validate_track_number_one() {
    // Track number 1 should be valid
    let track = create_test_track_with_track_number(Some(1));
    let result = validate_tracks(vec![track]);

    assert!(result.valid);
    assert!(result.warnings.is_empty());
}

#[test]
fn test_validate_track_number_ninety_nine() {
    // Track number 99 should be valid (high but reasonable)
    let track = create_test_track_with_track_number(Some(99));
    let result = validate_tracks(vec![track]);

    assert!(result.valid);
    assert!(result.warnings.is_empty());
}

#[test]
fn test_validate_track_number_one_hundred() {
    // Track number 100 should produce warning
    let track = create_test_track_with_track_number(Some(100));
    let result = validate_tracks(vec![track]);

    assert!(result.valid);
    assert!(!result.warnings.is_empty());

    let track_warnings: Vec<_> = result
        .warnings
        .iter()
        .filter(|w| w.field == "track_number")
        .collect();
    assert!(!track_warnings.is_empty());
}

#[test]
fn test_validate_disc_number_one() {
    // Disc number 1 should be valid (disc number is not validated for warnings)
    let track = create_test_track_with_disc_number(Some(1));
    let result = validate_tracks(vec![track]);

    assert!(result.valid);
}

#[test]
fn test_validate_empty_title() {
    let track = Track::new(
        PathBuf::from("test.flac"),
        TrackMetadata {
            title: Some(MetadataValue::embedded("".to_string())),
            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
            album: Some(MetadataValue::embedded("Test Album".to_string())),
            album_artist: None,
            track_number: None,
            disc_number: None,
            year: None,
            genre: None,
            duration: None,
            format: "flac".to_string(),
            path: PathBuf::from("test.flac"),
        },
    );

    let result = validate_tracks(vec![track]);
    assert!(!result.valid);

    let title_errors: Vec<_> = result
        .errors
        .iter()
        .filter(|e| e.field == "title")
        .collect();
    assert!(!title_errors.is_empty());
}

#[test]
fn test_validate_whitespace_only_title() {
    let track = Track::new(
        PathBuf::from("test.flac"),
        TrackMetadata {
            title: Some(MetadataValue::embedded("   ".to_string())),
            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
            album: Some(MetadataValue::embedded("Test Album".to_string())),
            album_artist: None,
            track_number: None,
            disc_number: None,
            year: None,
            genre: None,
            duration: None,
            format: "flac".to_string(),
            path: PathBuf::from("test.flac"),
        },
    );

    let result = validate_tracks(vec![track]);
    assert!(!result.valid);
}

#[test]
fn test_validate_missing_required_fields() {
    let track = Track::new(
        PathBuf::from("test.flac"),
        TrackMetadata {
            title: None,
            artist: None,
            album: None,
            album_artist: None,
            track_number: None,
            disc_number: None,
            year: None,
            genre: None,
            duration: None,
            format: "flac".to_string(),
            path: PathBuf::from("test.flac"),
        },
    );

    let result = validate_tracks(vec![track]);
    assert!(!result.valid);
    assert_eq!(result.errors.len(), 3); // title, artist, album
}

#[test]
fn test_validate_perfect_metadata() {
    let track = Track::new(
        PathBuf::from("test.flac"),
        TrackMetadata {
            title: Some(MetadataValue::embedded("Perfect Title".to_string())),
            artist: Some(MetadataValue::embedded("Perfect Artist".to_string())),
            album: Some(MetadataValue::embedded("Perfect Album".to_string())),
            album_artist: Some(MetadataValue::embedded("Perfect Album Artist".to_string())),
            track_number: Some(MetadataValue::embedded(5)),
            disc_number: Some(MetadataValue::embedded(1)),
            year: Some(MetadataValue::embedded(2020)),
            genre: Some(MetadataValue::embedded("Rock".to_string())),
            duration: Some(MetadataValue::embedded(180.5)),
            format: "flac".to_string(),
            path: PathBuf::from("test.flac"),
        },
    );

    let result = validate_tracks(vec![track]);
    assert!(result.valid);
    assert!(result.warnings.is_empty());
    assert!(result.errors.is_empty());
}
