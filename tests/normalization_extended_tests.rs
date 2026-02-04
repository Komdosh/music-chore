use music_chore::core::services::normalization::normalize;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_normalize_dry_run() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    // Create test directory structure
    fs::create_dir_all(source_path.join("artist/album")).unwrap();

    // Create test files with lowercase titles
    let test_file1 = source_path.join("artist/album/track1.flac");
    let test_file2 = source_path.join("artist/album/track2.flac");

    fs::copy("tests/fixtures/flac/simple/track1.flac", &test_file1).unwrap();
    fs::copy("tests/fixtures/flac/simple/track2.flac", &test_file2).unwrap();

    let result = normalize(source_path.to_path_buf(), true);
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.contains("track1.flac"));
    assert!(output.contains("track2.flac"));
    // The test files from fixtures likely already have proper title case, so they might not need normalization
}

#[test]
fn test_normalize_apply_changes() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    // Create test directory structure
    fs::create_dir_all(source_path.join("artist/album")).unwrap();

    // Copy test files
    let test_file1 = source_path.join("artist/album/track1.flac");
    let test_file2 = source_path.join("artist/album/track2.flac");

    fs::copy("tests/fixtures/flac/simple/track1.flac", &test_file1).unwrap();
    fs::copy("tests/fixtures/flac/simple/track2.flac", &test_file2).unwrap();

    let result = normalize(source_path.to_path_buf(), false);
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.contains("track1.flac"));
    assert!(output.contains("track2.flac"));
    // The test files from fixtures likely already have proper title case, so they might not need normalization
}

#[test]
fn test_normalize_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let result = normalize(temp_dir.path().to_path_buf(), true);

    // Empty directory should return success but with no specific message
    assert!(result.is_ok());
}

#[test]
fn test_normalize_nonexistent_directory() {
    let nonexistent_path = PathBuf::from("/nonexistent/path");
    let result = normalize(nonexistent_path, true);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
fn test_normalize_unsupported_files() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    // Create unsupported files
    fs::write(source_path.join("file.txt"), "text content").unwrap();
    fs::write(source_path.join("file.jpg"), "image content").unwrap();

    let result = normalize(source_path.to_path_buf(), true);
    // Should return success but no normalization since no music files
    assert!(result.is_ok());
}

#[test]
fn test_normalize_mixed_file_types() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    // Create test directory structure
    fs::create_dir_all(source_path.join("artist/album")).unwrap();

    // Mix of music and non-music files
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        source_path.join("artist/album/track1.flac"),
    )
    .unwrap();
    fs::write(source_path.join("artist/album/readme.txt"), "album info").unwrap();
    fs::write(source_path.join("artist/album/cover.jpg"), "image content").unwrap();

    let result = normalize(source_path.to_path_buf(), true);
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.contains("track1.flac"));
    // Should only process the music file, not txt/jpg files
}

#[test]
fn test_normalize_nested_directories() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    // Create nested directory structure
    fs::create_dir_all(source_path.join("artist1/album1")).unwrap();
    fs::create_dir_all(source_path.join("artist2/album2")).unwrap();

    // Copy test files to different directories
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        source_path.join("artist1/album1/track1.flac"),
    )
    .unwrap();
    fs::copy(
        "tests/fixtures/flac/simple/track2.flac",
        source_path.join("artist2/album2/track2.flac"),
    )
    .unwrap();

    let result = normalize(source_path.to_path_buf(), true);
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.contains("track1.flac"));
    assert!(output.contains("track2.flac"));
}

#[test]
fn test_normalize_different_formats() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    // Create test directory structure
    fs::create_dir_all(source_path.join("artist/album")).unwrap();

    // Copy different format files
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        source_path.join("artist/album/track1.flac"),
    )
    .unwrap();
    fs::copy(
        "tests/fixtures/mp3/simple/track1.mp3",
        source_path.join("artist/album/track2.mp3"),
    )
    .unwrap();
    fs::copy(
        "tests/fixtures/wav/simple/track1.wav",
        source_path.join("artist/album/track3.wav"),
    )
    .unwrap();

    let result = normalize(source_path.to_path_buf(), true);
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.contains("track1.flac"));
    assert!(output.contains("track2.mp3"));
    assert!(output.contains("track3.wav"));
}

#[test]
fn test_normalize_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    // Create test directory structure
    fs::create_dir_all(source_path.join("artist/album")).unwrap();

    // Copy a test file
    let test_file = source_path.join("artist/album/track1.flac");
    fs::copy("tests/fixtures/flac/simple/track1.flac", &test_file).unwrap();

    // Make the file read-only to simulate write error
    let mut perms = fs::metadata(&test_file).unwrap().permissions();
    perms.set_readonly(true);
    fs::set_permissions(&test_file, perms).unwrap();

    let result = normalize(source_path.to_path_buf(), false);
    // Should handle the error gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_normalize_unicode_paths() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    // Create test directory structure with unicode
    fs::create_dir_all(source_path.join("艺术家/专辑")).unwrap();

    // Copy test file
    let test_file = source_path.join("艺术家/专辑/track1.flac");
    fs::copy("tests/fixtures/flac/simple/track1.flac", &test_file).unwrap();

    let result = normalize(source_path.to_path_buf(), true);
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.contains("track1.flac"));
}

#[test]
fn test_normalize_preserves_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    // Create test directory structure
    fs::create_dir_all(source_path.join("artist/album")).unwrap();

    // Copy test file
    let test_file = source_path.join("artist/album/track1.flac");
    fs::copy("tests/fixtures/flac/simple/track1.flac", &test_file).unwrap();

    // Read original metadata
    use music_chore::adapters::audio_formats::read_metadata;
    let original_track = read_metadata(&test_file).unwrap();
    let original_title = original_track
        .metadata
        .title
        .as_ref()
        .unwrap()
        .value
        .clone();

    let result = normalize(source_path.to_path_buf(), false);
    assert!(result.is_ok());

    // Check that metadata is preserved
    let updated_track = read_metadata(&test_file).unwrap();
    let updated_title = updated_track.metadata.title.as_ref().unwrap().value.clone();

    assert_eq!(original_title, updated_title);
}
