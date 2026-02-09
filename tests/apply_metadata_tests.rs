//! Tests for the apply metadata module functionality.

use music_chore::core::services::apply_metadata::write_metadata_by_path;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn create_test_flac_file(temp_dir: &TempDir) -> PathBuf {
    let source_path = "tests/fixtures/flac/simple/track1.flac";
    let dest_path = temp_dir.path().join("test_track.flac");

    // Copy the test fixture to our temp directory
    fs::copy(source_path, &dest_path).expect("Failed to copy test fixture");

    dest_path
}

#[test]
fn test_write_metadata_by_path_dry_run() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_flac_file(&temp_dir);

    // Test dry_run mode - should not modify the file
    let result = write_metadata_by_path(
        &test_file,
        vec![
            "title=New Title".to_string(),
            "artist=New Artist".to_string(),
        ],
        false, // apply = false
        true,  // dry_run = true
    );

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("DRY RUN: Would set title = New Title"));
    assert!(output.contains("DRY RUN: Would set artist = New Artist"));
    assert!(output.contains("DRY RUN: No changes made to file:"));

    // Verify the file was not actually modified by checking that original metadata is still there
    let original_track = music_chore::adapters::audio_formats::read_metadata(&test_file).unwrap();
    assert_eq!(
        original_track.metadata.title.as_ref().unwrap().value,
        "Test Apply Behavior"
    );
}

#[test]
fn test_write_metadata_by_path_apply_changes() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_flac_file(&temp_dir);

    // Test apply mode - should modify the file
    let result = write_metadata_by_path(
        &test_file,
        vec!["title=New Title".to_string()],
        true,  // apply = true
        false, // dry_run = false
    );

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Successfully updated metadata:"));

    // Verify the file was actually modified
    let updated_track = music_chore::adapters::audio_formats::read_metadata(&test_file).unwrap();
    assert_eq!(
        updated_track.metadata.title.as_ref().unwrap().value,
        "New Title"
    );
}

#[test]
fn test_write_metadata_by_path_both_flags_error() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_flac_file(&temp_dir);

    // Test both apply and dry_run flags - should return error
    let result = write_metadata_by_path(
        &test_file,
        vec!["title=New Title".to_string()],
        true, // apply = true
        true, // dry_run = true
    );

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Cannot use both"));
}

#[test]
fn test_write_metadata_by_path_nonexistent_file() {
    let nonexistent_path = PathBuf::from("/nonexistent/path/test.flac");

    // Test with nonexistent file - should return error
    let result = write_metadata_by_path(
        &nonexistent_path,
        vec!["title=New Title".to_string()],
        true,  // apply = true
        false, // dry_run = false
    );

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("File not found"));
}

#[test]
fn test_write_metadata_by_path_unsupported_format() {
    let temp_dir = TempDir::new().unwrap();
    let unsupported_file = temp_dir.path().join("test.txt");
    fs::write(&unsupported_file, "dummy content").unwrap();

    // Test with unsupported file format - should return error
    let result = write_metadata_by_path(
        &unsupported_file,
        vec!["title=New Title".to_string()],
        true,  // apply = true
        false, // dry_run = false
    );

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Unsupported audio format"));
}

#[test]
fn test_write_metadata_by_path_invalid_metadata_field() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_flac_file(&temp_dir);

    // Test with invalid metadata field - should return error
    let result = write_metadata_by_path(
        &test_file,
        vec!["invalid_field=Some Value".to_string()],
        true,  // apply = true
        false, // dry_run = false
    );

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Invalid value"));
}

#[test]
fn test_write_metadata_by_path_invalid_track_number() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_flac_file(&temp_dir);

    // Test with invalid track number - should return error
    let result = write_metadata_by_path(
        &test_file,
        vec!["tracknumber=invalid".to_string()],
        true,  // apply = true
        false, // dry_run = false
    );

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Invalid value 'invalid' for field 'tracknumber'"));
}

#[test]
fn test_write_metadata_by_path_invalid_year() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_flac_file(&temp_dir);

    // Test with invalid year - should return error
    let result = write_metadata_by_path(
        &test_file,
        vec!["year=invalid".to_string()],
        true,  // apply = true
        false, // dry_run = false
    );

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Invalid value 'invalid' for field 'year'"));
}

#[test]
fn test_write_metadata_by_path_invalid_disc_number() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_flac_file(&temp_dir);

    // Test with invalid disc number - should return error
    let result = write_metadata_by_path(
        &test_file,
        vec!["discnumber=invalid".to_string()],
        true,  // apply = true
        false, // dry_run = false
    );

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Invalid value 'invalid' for field 'discnumber'"));
}

#[test]
fn test_write_metadata_by_path_default_dry_run() {
    // Test ID: AMD003
    // Given: Copy of fixture file
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_flac_file(&temp_dir);

    // Verify original title
    let original = music_chore::adapters::audio_formats::read_metadata(&test_file).unwrap();
    let original_title = original.metadata.title.as_ref().unwrap().value.clone();

    // When: Calling with both flags false (default should be dry-run)
    let result = write_metadata_by_path(
        &test_file,
        vec!["title=New Title".to_string()],
        false, // apply = false
        false, // dry_run = false
    );

    // Then: Acts as dry run, file not modified
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("DRY RUN"));

    // Verify file was NOT modified
    let updated = music_chore::adapters::audio_formats::read_metadata(&test_file).unwrap();
    assert_eq!(
        updated.metadata.title.as_ref().unwrap().value,
        original_title
    );
}

#[test]
fn test_write_metadata_by_path_multiple_fields() {
    // Test ID: AMD007
    // Given: Copy of fixture file
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_flac_file(&temp_dir);

    // When: Calling with multiple fields
    let result = write_metadata_by_path(
        &test_file,
        vec![
            "title=New Title".to_string(),
            "artist=New Artist".to_string(),
            "album=New Album".to_string(),
            "genre=New Genre".to_string(),
        ],
        true,  // apply = true
        false, // dry_run = false
    );

    // Then: All fields updated
    assert!(result.is_ok());
    assert!(result.unwrap().contains("Successfully updated"));

    let updated = music_chore::adapters::audio_formats::read_metadata(&test_file).unwrap();
    assert_eq!(updated.metadata.title.as_ref().unwrap().value, "New Title");
    assert_eq!(
        updated.metadata.artist.as_ref().unwrap().value,
        "New Artist"
    );
    assert_eq!(updated.metadata.album.as_ref().unwrap().value, "New Album");
    assert_eq!(updated.metadata.genre.as_ref().unwrap().value, "New Genre");

    // Note: MetadataSource is not persisted through file I/O, so we only verify values
}

#[test]
fn test_write_metadata_by_path_invalid_field_format() {
    // Test ID: AMD008
    // Given: Copy of fixture file
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_flac_file(&temp_dir);

    // When: Calling with invalid format (no equals sign)
    let result = write_metadata_by_path(
        &test_file,
        vec!["invalid-format-no-equals".to_string()],
        true,  // apply = true
        false, // dry_run = false
    );

    // Then: Returns error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Invalid value") || error_msg.contains("format"));
}

#[test]
fn test_write_metadata_by_path_update_track_number() {
    // Test updating track number field
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_flac_file(&temp_dir);

    let result = write_metadata_by_path(
        &test_file,
        vec!["tracknumber=5".to_string()],
        true,  // apply = true
        false, // dry_run = false
    );

    assert!(result.is_ok());

    let updated = music_chore::adapters::audio_formats::read_metadata(&test_file).unwrap();
    assert_eq!(updated.metadata.track_number.as_ref().unwrap().value, 5);
    // Note: MetadataSource is not persisted through file I/O
}

#[test]
fn test_write_metadata_by_path_update_year() {
    // Test updating year field
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_flac_file(&temp_dir);

    let result = write_metadata_by_path(
        &test_file,
        vec!["year=2024".to_string()],
        true,  // apply = true
        false, // dry_run = false
    );

    assert!(result.is_ok());

    let updated = music_chore::adapters::audio_formats::read_metadata(&test_file).unwrap();
    assert_eq!(updated.metadata.year.as_ref().unwrap().value, 2024);
    // Note: MetadataSource is not persisted through file I/O
}

#[test]
fn test_write_metadata_by_path_update_disc_number() {
    // Test updating disc number field
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_flac_file(&temp_dir);

    let result = write_metadata_by_path(
        &test_file,
        vec!["discnumber=2".to_string()],
        true,  // apply = true
        false, // dry_run = false
    );

    assert!(result.is_ok());

    let updated = music_chore::adapters::audio_formats::read_metadata(&test_file).unwrap();
    assert_eq!(updated.metadata.disc_number.as_ref().unwrap().value, 2);
    // Note: MetadataSource is not persisted through file I/O
}

#[test]
fn test_write_metadata_by_path_update_album_artist() {
    // Test updating album_artist field (with alias albumartist)
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_flac_file(&temp_dir);

    // Test with album_artist key
    let result = write_metadata_by_path(
        &test_file,
        vec!["album_artist=Compilation Artist".to_string()],
        true,  // apply = true
        false, // dry_run = false
    );

    assert!(result.is_ok());

    let updated = music_chore::adapters::audio_formats::read_metadata(&test_file).unwrap();
    assert_eq!(
        updated.metadata.album_artist.as_ref().unwrap().value,
        "Compilation Artist"
    );
    // Note: MetadataSource is not persisted through file I/O
}
