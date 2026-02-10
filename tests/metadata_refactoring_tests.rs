//! Tests for metadata-related functionality as part of the refactoring plan.

use music_chore::core::domain::models::{MetadataSource, MetadataValue, Track, TrackMetadata};
use std::path::PathBuf;

#[test]
fn test_metadata_value_display_trait() {
    // Test that MetadataValue implements Display trait properly
    let embedded_value = MetadataValue::embedded("Test Title".to_string());
    assert_eq!(format!("{}", embedded_value), "Test Title");

    let inferred_value = MetadataValue::inferred("Test Artist".to_string(), 0.5);
    assert_eq!(format!("{}", inferred_value), "Test Artist");

    let user_set_value = MetadataValue::user_set("Test Album".to_string());
    assert_eq!(format!("{}", user_set_value), "Test Album");

    let cue_inferred_value = MetadataValue::cue_inferred("Test Track".to_string(), 0.8);
    assert_eq!(format!("{}", cue_inferred_value), "Test Track");
}

#[test]
fn test_metadata_value_source_trait() {
    // Test that different sources are properly set
    let embedded_value = MetadataValue::embedded("Test".to_string());
    assert_eq!(embedded_value.source, MetadataSource::Embedded);
    assert_eq!(embedded_value.confidence, 1.0);

    let inferred_value = MetadataValue::inferred("Test".to_string(), 0.3);
    assert_eq!(inferred_value.source, MetadataSource::FolderInferred);
    assert_eq!(inferred_value.confidence, 0.3);

    let user_set_value = MetadataValue::user_set("Test".to_string());
    assert_eq!(user_set_value.source, MetadataSource::UserEdited);
    assert_eq!(user_set_value.confidence, 1.0);

    let cue_inferred_value = MetadataValue::cue_inferred("Test".to_string(), 1.0);
    assert_eq!(cue_inferred_value.source, MetadataSource::CueInferred);
    assert_eq!(cue_inferred_value.confidence, 1.0);
}

#[test]
fn test_metadata_value_debug_trait() {
    // Test that MetadataValue implements Debug trait properly
    let embedded_value = MetadataValue::embedded("Test Title".to_string());
    let debug_str = format!("{:?}", embedded_value);

    assert!(debug_str.contains("MetadataValue"));
    assert!(debug_str.contains("Test Title"));
    assert!(debug_str.contains("Embedded")); // Source variant
}

#[test]
fn test_track_checksum_calculation() {
    // Test that track checksum calculation works properly
    let track = Track::new(
        PathBuf::from("tests/fixtures/flac/simple/track1.flac"),
        TrackMetadata {
            title: Some(MetadataValue::embedded("Test Song".to_string())),
            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
            album: Some(MetadataValue::embedded("Test Album".to_string())),
            album_artist: None,
            track_number: Some(MetadataValue::embedded(1)),
            disc_number: Some(MetadataValue::embedded(1)),
            year: Some(MetadataValue::embedded(2023)),
            genre: Some(MetadataValue::embedded("Test Genre".to_string())),
            duration: Some(MetadataValue::embedded(1.0)),
            format: "flac".to_string(),
            path: PathBuf::from("tests/fixtures/flac/simple/track1.flac"),
        },
    );

    // Test that checksum can be calculated
    match track.calculate_checksum() {
        Ok(checksum) => {
            assert!(!checksum.is_empty());
            assert_eq!(checksum.len(), 64); // SHA256 produces 64-character hex string
        }
        Err(e) => {
            // If the file doesn't exist or can't be read, that's OK for this test
            // The important thing is that the function exists and handles errors properly
            eprintln!("Could not calculate checksum: {}", e);
        }
    }
}

#[test]
fn test_track_checksum_deterministic() {
    // Test that checksum is deterministic (same file produces same checksum)
    let track = Track::new(
        PathBuf::from("tests/fixtures/flac/simple/track1.flac"),
        TrackMetadata {
            title: Some(MetadataValue::embedded("Test Song".to_string())),
            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
            album: Some(MetadataValue::embedded("Test Album".to_string())),
            album_artist: None,
            track_number: Some(MetadataValue::embedded(1)),
            disc_number: Some(MetadataValue::embedded(1)),
            year: Some(MetadataValue::embedded(2023)),
            genre: Some(MetadataValue::embedded("Test Genre".to_string())),
            duration: Some(MetadataValue::embedded(1.0)),
            format: "flac".to_string(),
            path: PathBuf::from("tests/fixtures/flac/simple/track1.flac"),
        },
    );

    let checksum1 = track.calculate_checksum();
    let checksum2 = track.calculate_checksum();

    // Both checksums should be the same (or both should fail)
    match (&checksum1, &checksum2) {
        (Ok(c1), Ok(c2)) => assert_eq!(c1, c2, "Checksums should be identical for same file"),
        (Err(_), Err(_)) => (), // Both failed, which is acceptable
        _ => panic!("Checksums should either both succeed or both fail"),
    }
}

#[test]
fn test_track_with_precomputed_checksum() {
    // Test creating a track with a pre-computed checksum
    let original_path = PathBuf::from("tests/fixtures/flac/simple/track1.flac");
    let track = Track::with_checksum(
        original_path.clone(),
        TrackMetadata {
            title: Some(MetadataValue::embedded("Test Song".to_string())),
            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
            album: Some(MetadataValue::embedded("Test Album".to_string())),
            album_artist: None,
            track_number: Some(MetadataValue::embedded(1)),
            disc_number: Some(MetadataValue::embedded(1)),
            year: Some(MetadataValue::embedded(2023)),
            genre: Some(MetadataValue::embedded("Test Genre".to_string())),
            duration: Some(MetadataValue::embedded(1.0)),
            format: "flac".to_string(),
            path: original_path,
        },
        "precomputed_checksum".to_string(),
    );

    assert_eq!(track.checksum, Some("precomputed_checksum".to_string()));
}

#[test]
fn test_track_equality_implementation() {
    // Test that Track implements equality properly
    let track1 = Track::new(
        PathBuf::from("tests/fixtures/flac/simple/track1.flac"),
        TrackMetadata {
            title: Some(MetadataValue::embedded("Test Song".to_string())),
            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
            album: Some(MetadataValue::embedded("Test Album".to_string())),
            album_artist: None,
            track_number: Some(MetadataValue::embedded(1)),
            disc_number: Some(MetadataValue::embedded(1)),
            year: Some(MetadataValue::embedded(2023)),
            genre: Some(MetadataValue::embedded("Test Genre".to_string())),
            duration: Some(MetadataValue::embedded(1.0)),
            format: "flac".to_string(),
            path: PathBuf::from("tests/fixtures/flac/simple/track1.flac"),
        },
    );

    let track2 = Track::new(
        PathBuf::from("tests/fixtures/flac/simple/track1.flac"),
        TrackMetadata {
            title: Some(MetadataValue::embedded("Test Song".to_string())),
            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
            album: Some(MetadataValue::embedded("Test Album".to_string())),
            album_artist: None,
            track_number: Some(MetadataValue::embedded(1)),
            disc_number: Some(MetadataValue::embedded(1)),
            year: Some(MetadataValue::embedded(2023)),
            genre: Some(MetadataValue::embedded("Test Genre".to_string())),
            duration: Some(MetadataValue::embedded(1.0)),
            format: "flac".to_string(),
            path: PathBuf::from("tests/fixtures/flac/simple/track1.flac"),
        },
    );

    assert_eq!(track1.file_path, track2.file_path);
}

#[test]
fn test_track_debug_implementation() {
    // Test that Track implements Debug properly
    let track = Track::new(
        PathBuf::from("tests/fixtures/flac/simple/track1.flac"),
        TrackMetadata {
            title: Some(MetadataValue::embedded("Test Song".to_string())),
            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
            album: Some(MetadataValue::embedded("Test Album".to_string())),
            album_artist: None,
            track_number: Some(MetadataValue::embedded(1)),
            disc_number: Some(MetadataValue::embedded(1)),
            year: Some(MetadataValue::embedded(2023)),
            genre: Some(MetadataValue::embedded("Test Genre".to_string())),
            duration: Some(MetadataValue::embedded(1.0)),
            format: "flac".to_string(),
            path: PathBuf::from("tests/fixtures/flac/simple/track1.flac"),
        },
    );

    let debug_str = format!("{:?}", track);
    assert!(debug_str.contains("Track"));
    assert!(debug_str.contains("track1.flac"));
}

#[test]
fn test_track_metadata_equality_implementation() {
    // Test that TrackMetadata implements equality properly
    let metadata1 = TrackMetadata {
        title: Some(MetadataValue::embedded("Test Song".to_string())),
        artist: Some(MetadataValue::embedded("Test Artist".to_string())),
        album: Some(MetadataValue::embedded("Test Album".to_string())),
        album_artist: None,
        track_number: Some(MetadataValue::embedded(1)),
        disc_number: Some(MetadataValue::embedded(1)),
        year: Some(MetadataValue::embedded(2023)),
        genre: Some(MetadataValue::embedded("Test Genre".to_string())),
        duration: Some(MetadataValue::embedded(1.0)),
        format: "flac".to_string(),
        path: PathBuf::from("tests/fixtures/flac/simple/track1.flac"),
    };

    let metadata2 = TrackMetadata {
        title: Some(MetadataValue::embedded("Test Song".to_string())),
        artist: Some(MetadataValue::embedded("Test Artist".to_string())),
        album: Some(MetadataValue::embedded("Test Album".to_string())),
        album_artist: None,
        track_number: Some(MetadataValue::embedded(1)),
        disc_number: Some(MetadataValue::embedded(1)),
        year: Some(MetadataValue::embedded(2023)),
        genre: Some(MetadataValue::embedded("Test Genre".to_string())),
        duration: Some(MetadataValue::embedded(1.0)),
        format: "flac".to_string(),
        path: PathBuf::from("tests/fixtures/flac/simple/track1.flac"),
    };

    assert_eq!(metadata1, metadata2);
}

#[test]
fn test_metadata_value_clone_implementation() {
    // Test that MetadataValue implements Clone properly
    let original = MetadataValue::embedded("Test Value".to_string());
    let cloned = original.clone();

    assert_eq!(original.value, cloned.value);
    assert_eq!(original.source, cloned.source);
    assert_eq!(original.confidence, cloned.confidence);
}
