//! Tests for metadata application functionality

use music_chore::services::apply_metadata::write_metadata_by_path;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_write_metadata_requires_apply_or_dry_run() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.flac");
    fs::copy("tests/fixtures/flac/simple/track1.flac", &test_file).unwrap();

    // Neither apply nor dry_run - should error
    let result = write_metadata_by_path(
        &test_file,
        vec!["title=New Title".to_string()],
        false,
        false,
    );
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .contains("Must specify either --apply or --dry-run")
    );
}

#[test]
fn test_write_metadata_prevents_both_flags() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.flac");
    fs::copy("tests/fixtures/flac/simple/track1.flac", &test_file).unwrap();

    // Both apply and dry_run - should error
    let result =
        write_metadata_by_path(&test_file, vec!["title=New Title".to_string()], true, true);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .contains("Cannot use both --apply and --dry-run")
    );
}

#[test]
fn test_write_metadata_nonexistent_file() {
    let path = PathBuf::from("/nonexistent/path/test.flac");

    let result = write_metadata_by_path(&path, vec!["title=New Title".to_string()], false, true);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
fn test_write_metadata_unsupported_format() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ogg");
    fs::write(&test_file, "dummy content").unwrap();

    let result =
        write_metadata_by_path(&test_file, vec!["title=New Title".to_string()], false, true);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unsupported file format"));
}

#[test]
fn test_write_metadata_dry_run() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.flac");
    fs::copy("tests/fixtures/flac/simple/track1.flac", &test_file).unwrap();

    // Dry run should not modify the file
    let result = write_metadata_by_path(
        &test_file,
        vec![
            "title=New Title".to_string(),
            "artist=New Artist".to_string(),
            "album=New Album".to_string(),
        ],
        false,
        true,
    );

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("DRY RUN"));
    assert!(output.contains("title = New Title"));
    assert!(output.contains("artist = New Artist"));
    assert!(output.contains("album = New Album"));
}

#[test]
fn test_write_metadata_apply_all_fields() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.flac");
    fs::copy("tests/fixtures/flac/simple/track1.flac", &test_file).unwrap();

    // Apply should modify the file
    let result = write_metadata_by_path(
        &test_file,
        vec![
            "title=New Title".to_string(),
            "artist=New Artist".to_string(),
            "album=New Album".to_string(),
            "albumartist=New Album Artist".to_string(),
            "tracknumber=5".to_string(),
            "discnumber=2".to_string(),
            "year=2023".to_string(),
            "genre=Rock".to_string(),
        ],
        true,
        false,
    );

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Successfully updated metadata"));

    // Verify the metadata was written
    let track = music_chore::services::formats::read_metadata(&test_file).unwrap();
    assert_eq!(track.metadata.title.unwrap().value, "New Title");
    assert_eq!(track.metadata.artist.unwrap().value, "New Artist");
    assert_eq!(track.metadata.album.unwrap().value, "New Album");
    assert_eq!(
        track.metadata.album_artist.unwrap().value,
        "New Album Artist"
    );
    assert_eq!(track.metadata.track_number.unwrap().value, 5);
    assert_eq!(track.metadata.disc_number.unwrap().value, 2);
    assert_eq!(track.metadata.year.unwrap().value, 2023);
    assert_eq!(track.metadata.genre.unwrap().value, "Rock");
}

#[test]
fn test_write_metadata_invalid_track_number() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.flac");
    fs::copy("tests/fixtures/flac/simple/track1.flac", &test_file).unwrap();

    let result = write_metadata_by_path(
        &test_file,
        vec!["tracknumber=invalid".to_string()],
        false,
        true,
    );

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid track number"));
}

#[test]
fn test_write_metadata_invalid_disc_number() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.flac");
    fs::copy("tests/fixtures/flac/simple/track1.flac", &test_file).unwrap();

    let result = write_metadata_by_path(
        &test_file,
        vec!["discnumber=invalid".to_string()],
        false,
        true,
    );

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid disc number"));
}

#[test]
fn test_write_metadata_invalid_year() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.flac");
    fs::copy("tests/fixtures/flac/simple/track1.flac", &test_file).unwrap();

    let result = write_metadata_by_path(&test_file, vec!["year=invalid".to_string()], false, true);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid year"));
}

#[test]
fn test_write_metadata_unsupported_field() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.flac");
    fs::copy("tests/fixtures/flac/simple/track1.flac", &test_file).unwrap();

    let result = write_metadata_by_path(
        &test_file,
        vec!["unsupported_field=value".to_string()],
        false,
        true,
    );

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unsupported metadata field"));
}

#[test]
fn test_write_metadata_invalid_format() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.flac");
    fs::copy("tests/fixtures/flac/simple/track1.flac", &test_file).unwrap();

    // Missing equals sign
    let result = write_metadata_by_path(&test_file, vec!["invalidformat".to_string()], false, true);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unsupported metadata format"));
}

#[test]
fn test_write_metadata_case_insensitive_field_names() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.flac");
    fs::copy("tests/fixtures/flac/simple/track1.flac", &test_file).unwrap();

    // Test various case combinations
    let result = write_metadata_by_path(
        &test_file,
        vec![
            "TITLE=New Title".to_string(),
            "Artist=New Artist".to_string(),
            "ALBUM=New Album".to_string(),
            "AlbumArtist=New Album Artist".to_string(),
            "track_number=5".to_string(),
            "DISC_NUMBER=2".to_string(),
            "YEAR=2023".to_string(),
            "Genre=Rock".to_string(),
        ],
        true,
        false,
    );

    assert!(result.is_ok());

    // Verify the metadata was written
    let track = music_chore::services::formats::read_metadata(&test_file).unwrap();
    assert_eq!(track.metadata.title.unwrap().value, "New Title");
    assert_eq!(track.metadata.artist.unwrap().value, "New Artist");
    assert_eq!(track.metadata.album.unwrap().value, "New Album");
    assert_eq!(
        track.metadata.album_artist.unwrap().value,
        "New Album Artist"
    );
    assert_eq!(track.metadata.track_number.unwrap().value, 5);
    assert_eq!(track.metadata.disc_number.unwrap().value, 2);
    assert_eq!(track.metadata.year.unwrap().value, 2023);
    assert_eq!(track.metadata.genre.unwrap().value, "Rock");
}
