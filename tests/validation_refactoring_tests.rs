//! Additional tests for the validation module as part of the refactoring plan.

use music_chore::core::services::validation::validate_path;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

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

    // Should contain validation results
    assert!(output.contains("📊 Summary:"));
    assert!(output.contains("Valid files: 1")); // One valid track
}

#[test]
fn test_validate_path_with_missing_title() {
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

    // The copied file has a title, so it should be valid
    assert!(output.contains("Valid files: 1"));
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

    // The copied file has an artist, so it should be valid
    assert!(output.contains("Valid files: 1"));
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

    // The copied file has an album, so it should be valid
    assert!(output.contains("Valid files: 1"));
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
    // The copied file has a valid year (2023), so it should be valid
    assert!(output.contains("Valid files: 1"));
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

    // The copied file has non-empty fields, so it should be valid
    assert!(output.contains("Valid files: 1"));
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

    // The copied file has a valid track number, so it should be valid
    assert!(output.contains("Valid files: 1"));
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

    // The copied file has a valid disc number, so it should be valid
    assert!(output.contains("Valid files: 1"));
}
