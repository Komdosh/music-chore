//! Tests for Track checksum functionality

use music_chore::core::domain::models::Track;
use music_chore::core::services::scanner::scan_with_duplicates;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_track_calculate_checksum() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.flac");
    fs::copy("tests/fixtures/flac/simple/track1.flac", &test_file).unwrap();

    let track = music_chore::adapters::audio_formats::read_metadata(&test_file).unwrap();

    // Calculate checksum
    let checksum = track.calculate_checksum().unwrap();

    // Verify it's a valid SHA256 hash (64 hex characters)
    assert_eq!(checksum.len(), 64);
    assert!(checksum.chars().all(|c| c.is_ascii_hexdigit()));

    // Verify the actual SHA256 hash value against pre-calculated known-good value
    // This ensures the checksum algorithm is correct, not just well-formed
    let expected_checksum = "e2b069fbc726ad70d1c65bf1fb7baa547daf4735f4e69e1f1021d295672a63dc";
    assert_eq!(checksum, expected_checksum, "Checksum mismatch - algorithm may have changed");
}

#[test]
fn test_track_checksum_deterministic() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.flac");
    fs::copy("tests/fixtures/flac/simple/track1.flac", &test_file).unwrap();

    let track1 = music_chore::adapters::audio_formats::read_metadata(&test_file).unwrap();
    let track2 = music_chore::adapters::audio_formats::read_metadata(&test_file).unwrap();

    // Same file should produce same checksum
    let checksum1 = track1.calculate_checksum().unwrap();
    let checksum2 = track2.calculate_checksum().unwrap();

    assert_eq!(checksum1, checksum2);
}

#[test]
fn test_track_checksum_different_files() {
    let temp_dir = TempDir::new().unwrap();
    let test_file1 = temp_dir.path().join("test1.flac");
    let test_file2 = temp_dir.path().join("test2.flac");

    // Copy the same file twice, then modify one to make it different
    fs::copy("tests/fixtures/flac/simple/track1.flac", &test_file1).unwrap();
    fs::copy("tests/fixtures/flac/simple/track1.flac", &test_file2).unwrap();

    // Modify the second file slightly to make it different
    let metadata = music_chore::adapters::audio_formats::read_metadata(&test_file2).unwrap();
    let mut new_metadata = metadata.metadata.clone();
    new_metadata.title = Some(music_chore::core::domain::models::MetadataValue::user_set(
        "Modified Title".to_string(),
    ));
    music_chore::adapters::audio_formats::write_metadata(&test_file2, &new_metadata).unwrap();

    let track1 = music_chore::adapters::audio_formats::read_metadata(&test_file1).unwrap();
    let track2 = music_chore::adapters::audio_formats::read_metadata(&test_file2).unwrap();

    // Different files should produce different checksums
    let checksum1 = track1.calculate_checksum().unwrap();
    let checksum2 = track2.calculate_checksum().unwrap();

    assert_ne!(checksum1, checksum2);
}

#[test]
fn test_track_checksum_nonexistent_file() {
    let track = Track::new(
        PathBuf::from("/nonexistent/file.flac"),
        music_chore::core::domain::models::TrackMetadata {
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
            path: PathBuf::from("/nonexistent/file.flac"),
        },
    );

    let result = track.calculate_checksum();
    assert!(result.is_err());
}

#[test]
fn test_duplicate_detection() {
    let temp_dir = TempDir::new().unwrap();

    // Create a directory with duplicate files
    let dir_path = temp_dir.path().join("duplicates");
    fs::create_dir(&dir_path).unwrap();

    let file1 = dir_path.join("track1.flac");
    let file2 = dir_path.join("track2.flac");
    let file3 = dir_path.join("track3.flac");

    // Create two identical files and one different file
    fs::copy("tests/fixtures/flac/simple/track1.flac", &file1).unwrap();
    fs::copy("tests/fixtures/flac/simple/track1.flac", &file2).unwrap(); // Same as track1

    // Create a different file by copying track1 then modifying metadata
    fs::copy("tests/fixtures/flac/simple/track1.flac", &file3).unwrap();
    let metadata = music_chore::adapters::audio_formats::read_metadata(&file3).unwrap();
    let mut new_metadata = metadata.metadata.clone();
    new_metadata.title = Some(music_chore::core::domain::models::MetadataValue::user_set(
        "Different Title".to_string(),
    ));
    music_chore::adapters::audio_formats::write_metadata(&file3, &new_metadata).unwrap();

    let (tracks, duplicates) = scan_with_duplicates(&dir_path);

    // Should find 3 tracks total
    assert_eq!(tracks.len(), 3);
    // Should find exactly 1 duplicate group with 2 identical files
    assert_eq!(duplicates.len(), 1);

    // First duplicate group should have 2 files (track1 and track2 are identical)
    let first_group = &duplicates[0];
    assert_eq!(first_group.len(), 2);
}

#[test]
fn test_duplicate_detection_no_duplicates() {
    let temp_dir = TempDir::new().unwrap();

    // Create a directory with only unique files
    let dir_path = temp_dir.path().join("unique");
    fs::create_dir(&dir_path).unwrap();

    let file1 = dir_path.join("track1.flac");
    let file2 = dir_path.join("track2.flac");

    // Create two different files by copying base file and modifying one
    fs::copy("tests/fixtures/flac/simple/track1.flac", &file1).unwrap();
    fs::copy("tests/fixtures/flac/simple/track1.flac", &file2).unwrap();

    // Modify second file to make it different
    let metadata = music_chore::adapters::audio_formats::read_metadata(&file2).unwrap();
    let mut new_metadata = metadata.metadata.clone();
    new_metadata.title = Some(music_chore::core::domain::models::MetadataValue::user_set(
        "Different Title".to_string(),
    ));
    music_chore::adapters::audio_formats::write_metadata(&file2, &new_metadata).unwrap();

    let (tracks, duplicates) = scan_with_duplicates(&dir_path);

    // Should have tracks but no duplicates
    assert_eq!(tracks.len(), 2);
    assert!(duplicates.is_empty());
}

#[test]
fn test_track_with_checksum() {
    let path = PathBuf::from("/test/file.flac");
    let metadata = music_chore::core::domain::models::TrackMetadata {
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
        path: path.clone(),
    };

    let checksum = "abc123".to_string();
    let track = Track::with_checksum(path.clone(), metadata, checksum.clone());

    assert_eq!(track.file_path, path);
    assert_eq!(track.checksum, Some(checksum));
}
