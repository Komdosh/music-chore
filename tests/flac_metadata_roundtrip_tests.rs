//! Integration tests for FLAC metadata read/write roundtrip
//! Verifies that metadata written to a file can be read back correctly

use music_chore::adapters::audio_formats::{read_metadata, write_metadata};
use music_chore::core::domain::models::{MetadataSource, MetadataValue, TrackMetadata};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a TrackMetadata with all fields set
fn create_full_metadata() -> TrackMetadata {
    TrackMetadata {
        title: Some(MetadataValue::user_set("Test Song Title".to_string())),
        artist: Some(MetadataValue::user_set("Test Artist Name".to_string())),
        album: Some(MetadataValue::user_set("Test Album Name".to_string())),
        album_artist: Some(MetadataValue::user_set("Test Album Artist".to_string())),
        track_number: Some(MetadataValue::user_set(5)),
        disc_number: Some(MetadataValue::user_set(2)),
        year: Some(MetadataValue::user_set(2024)),
        genre: Some(MetadataValue::user_set("Test Genre".to_string())),
        duration: None, // Duration is read-only
        format: "flac".to_string(),
        path: PathBuf::from("test.flac"),
    }
}

#[test]
fn test_flac_metadata_roundtrip_all_fields() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.flac");

    // Copy fixture to temp directory
    fs::copy("tests/fixtures/flac/simple/track1.flac", &test_file).unwrap();

    // Create metadata with all fields
    let original_metadata = create_full_metadata();

    // Write metadata to file
    write_metadata(&test_file, &original_metadata).unwrap();

    // Read metadata back
    let track = read_metadata(&test_file).unwrap();
    let read_metadata = track.metadata;

    // Verify all fields match
    assert_eq!(
        read_metadata.title.as_ref().unwrap().value,
        "Test Song Title"
    );
    assert_eq!(
        read_metadata.artist.as_ref().unwrap().value,
        "Test Artist Name"
    );
    assert_eq!(
        read_metadata.album.as_ref().unwrap().value,
        "Test Album Name"
    );
    assert_eq!(
        read_metadata.album_artist.as_ref().unwrap().value,
        "Test Album Artist"
    );
    assert_eq!(read_metadata.track_number.as_ref().unwrap().value, 5);
    assert_eq!(read_metadata.disc_number.as_ref().unwrap().value, 2);
    assert_eq!(read_metadata.year.as_ref().unwrap().value, 2024);
    assert_eq!(read_metadata.genre.as_ref().unwrap().value, "Test Genre");
}

#[test]
fn test_flac_metadata_partial_update() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.flac");

    // Copy fixture to temp directory
    fs::copy("tests/fixtures/flac/simple/track1.flac", &test_file).unwrap();

    // Read original metadata
    let original = read_metadata(&test_file).unwrap();
    let original_title = original.metadata.title.as_ref().unwrap().value.clone();
    let original_artist = original.metadata.artist.as_ref().unwrap().value.clone();

    // Update only the album field
    let mut new_metadata = original.metadata.clone();
    new_metadata.album = Some(MetadataValue::user_set("Updated Album".to_string()));

    // Write only album change
    write_metadata(&test_file, &new_metadata).unwrap();

    // Read back and verify
    let updated = read_metadata(&test_file).unwrap();

    // Title and artist should remain unchanged
    assert_eq!(
        updated.metadata.title.as_ref().unwrap().value,
        original_title
    );
    assert_eq!(
        updated.metadata.artist.as_ref().unwrap().value,
        original_artist
    );
    // Album should be updated
    assert_eq!(
        updated.metadata.album.as_ref().unwrap().value,
        "Updated Album"
    );
}

#[test]
fn test_flac_metadata_unicode_values() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.flac");

    // Copy fixture to temp directory
    fs::copy("tests/fixtures/flac/simple/track1.flac", &test_file).unwrap();

    // Create metadata with unicode characters
    let mut metadata = create_full_metadata();
    metadata.title = Some(MetadataValue::user_set("日本語タイトル".to_string()));
    metadata.artist = Some(MetadataValue::user_set("艺术家".to_string()));
    metadata.album = Some(MetadataValue::user_set("Альбом".to_string()));
    metadata.genre = Some(MetadataValue::user_set("Électronique".to_string()));

    // Write metadata
    write_metadata(&test_file, &metadata).unwrap();

    // Read back and verify unicode is preserved
    let track = read_metadata(&test_file).unwrap();

    assert_eq!(
        track.metadata.title.as_ref().unwrap().value,
        "日本語タイトル"
    );
    assert_eq!(track.metadata.artist.as_ref().unwrap().value, "艺术家");
    assert_eq!(track.metadata.album.as_ref().unwrap().value, "Альбом");
    assert_eq!(track.metadata.genre.as_ref().unwrap().value, "Électronique");
}

#[test]
fn test_flac_metadata_special_characters() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.flac");

    // Copy fixture to temp directory
    fs::copy("tests/fixtures/flac/simple/track1.flac", &test_file).unwrap();

    // Create metadata with special characters
    let mut metadata = create_full_metadata();
    metadata.title = Some(MetadataValue::user_set(
        "Song & Artist (feat. Someone) [Remix]".to_string(),
    ));
    metadata.artist = Some(MetadataValue::user_set(
        "Band/Group \"The Best\"".to_string(),
    ));

    // Write metadata
    write_metadata(&test_file, &metadata).unwrap();

    // Read back and verify special characters are preserved
    let track = read_metadata(&test_file).unwrap();

    assert_eq!(
        track.metadata.title.as_ref().unwrap().value,
        "Song & Artist (feat. Someone) [Remix]"
    );
    assert_eq!(
        track.metadata.artist.as_ref().unwrap().value,
        "Band/Group \"The Best\""
    );
}

#[test]
fn test_flac_metadata_boundary_years() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.flac");

    // Copy fixture to temp directory
    fs::copy("tests/fixtures/flac/simple/track1.flac", &test_file).unwrap();

    // Test minimum valid year (1900)
    let mut metadata = create_full_metadata();
    metadata.year = Some(MetadataValue::user_set(1900));
    write_metadata(&test_file, &metadata).unwrap();
    let track = read_metadata(&test_file).unwrap();
    assert_eq!(track.metadata.year.as_ref().unwrap().value, 1900);

    // Test maximum valid year (2100)
    metadata.year = Some(MetadataValue::user_set(2100));
    write_metadata(&test_file, &metadata).unwrap();
    let track = read_metadata(&test_file).unwrap();
    assert_eq!(track.metadata.year.as_ref().unwrap().value, 2100);

    // Test year 0 (should still be stored, validation is separate)
    metadata.year = Some(MetadataValue::user_set(0));
    write_metadata(&test_file, &metadata).unwrap();
    let track = read_metadata(&test_file).unwrap();
    assert_eq!(track.metadata.year.as_ref().unwrap().value, 0);

    // Test year 9999 (should still be stored, validation is separate)
    metadata.year = Some(MetadataValue::user_set(9999));
    write_metadata(&test_file, &metadata).unwrap();
    let track = read_metadata(&test_file).unwrap();
    assert_eq!(track.metadata.year.as_ref().unwrap().value, 9999);
}

#[test]
fn test_flac_metadata_boundary_track_numbers() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.flac");

    // Copy fixture to temp directory
    fs::copy("tests/fixtures/flac/simple/track1.flac", &test_file).unwrap();

    // Test track number 0
    let mut metadata = create_full_metadata();
    metadata.track_number = Some(MetadataValue::user_set(0));
    write_metadata(&test_file, &metadata).unwrap();
    let track = read_metadata(&test_file).unwrap();
    assert_eq!(track.metadata.track_number.as_ref().unwrap().value, 0);

    // Test track number 1
    metadata.track_number = Some(MetadataValue::user_set(1));
    write_metadata(&test_file, &metadata).unwrap();
    let track = read_metadata(&test_file).unwrap();
    assert_eq!(track.metadata.track_number.as_ref().unwrap().value, 1);

    // Test track number 99
    metadata.track_number = Some(MetadataValue::user_set(99));
    write_metadata(&test_file, &metadata).unwrap();
    let track = read_metadata(&test_file).unwrap();
    assert_eq!(track.metadata.track_number.as_ref().unwrap().value, 99);

    // Test track number 255 (u8 max)
    metadata.track_number = Some(MetadataValue::user_set(255));
    write_metadata(&test_file, &metadata).unwrap();
    let track = read_metadata(&test_file).unwrap();
    assert_eq!(track.metadata.track_number.as_ref().unwrap().value, 255);
}

#[test]
fn test_flac_read_nonexistent_file() {
    let result = read_metadata(PathBuf::from("/nonexistent/file.flac").as_path());
    assert!(result.is_err());
}

#[test]
fn test_flac_read_unsupported_format() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "not an audio file").unwrap();

    let result = read_metadata(&test_file);
    assert!(result.is_err());
}
