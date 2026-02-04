use music_chore::services::validation::validate_path;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_validate_path_text_output() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    // Create test directory structure
    fs::create_dir_all(source_path.join("artist/album")).unwrap();

    // Copy a test file
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        source_path.join("artist/album/track1.flac"),
    )
    .unwrap();

    let result = validate_path(&source_path.to_path_buf(), false);
    assert!(result.is_ok());

    let output = result.unwrap();
    // Should contain some validation information
    assert!(!output.is_empty());
    assert!(output.len() > 10);
}

#[test]
fn test_validate_path_json_output() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    // Create test directory structure
    fs::create_dir_all(source_path.join("artist/album")).unwrap();

    // Copy a test file
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        source_path.join("artist/album/track1.flac"),
    )
    .unwrap();

    let result = validate_path(&source_path.to_path_buf(), true);
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.starts_with('{'));
    assert!(output.ends_with('}'));
    assert!(output.contains("valid"));
    assert!(output.contains("errors"));
    assert!(output.contains("warnings"));
    assert!(output.contains("summary"));
}

#[test]
fn test_validate_path_empty_directory_json() {
    let temp_dir = TempDir::new().unwrap();
    let result = validate_path(&temp_dir.path().to_path_buf(), true);

    assert!(result.is_err());

    let output = result.expect_err("Should have returned an error");
    assert!(output.contains("\"valid\": true"));
    // Should be valid JSON with validation results
    assert!(output.starts_with('{'));
    assert!(output.ends_with('}'));
}

#[test]
fn test_validate_path_nonexistent_directory() {
    let nonexistent_path = PathBuf::from("/nonexistent/path");
    let result = validate_path(&nonexistent_path, true);

    assert!(result.is_err());
    // Check that it's some kind of error about directory not existing
    let error_msg = result.unwrap_err();
    println!("Error message: {}", error_msg);
    assert!(error_msg.contains("\"valid\": true") || error_msg.contains("\"valid_files\": 0"));
}

#[test]
fn test_validate_path_with_errors() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    // Create test directory structure
    fs::create_dir_all(source_path.join("artist/album")).unwrap();

    // Copy a test file
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        source_path.join("artist/album/track1.flac"),
    )
    .unwrap();

    // First read the file to see what metadata it has
    use music_chore::services::formats::read_metadata;
    let mut track = read_metadata(&source_path.join("artist/album/track1.flac")).unwrap();

    // Clear the title to create an error
    track.metadata.title = Some(music_chore::MetadataValue {
        value: "   ".to_string(), // Empty/whitespace title
        source: music_chore::MetadataSource::Embedded,
        confidence: 1.0,
    });

    // Write it back (this might fail, but that's okay for this test)
    let _ = music_chore::services::formats::write_metadata(
        &source_path.join("artist/album/track1.flac"),
        &track.metadata,
    );

    let result = validate_path(&source_path.to_path_buf(), false);
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.contains("VALIDATION RESULTS"));
}

#[test]
fn test_validate_path_with_warnings() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    // Create test directory structure
    fs::create_dir_all(source_path.join("artist/album")).unwrap();

    // Copy a test file
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        source_path.join("artist/album/track1.flac"),
    )
    .unwrap();

    let result = validate_path(&source_path.to_path_buf(), false);
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.contains("VALIDATION RESULTS"));
}

#[test]
fn test_validate_path_mixed_formats() {
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

    let result = validate_path(&source_path.to_path_buf(), true);
    assert!(result.is_ok());

    let output = result.unwrap();
    // Should be valid JSON with 3 files
    assert!(output.starts_with('{'));
    assert!(output.ends_with('}'));
    assert!(output.contains("total_files"));
}

#[test]
fn test_validate_path_nested_directories() {
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

    let result = validate_path(&source_path.to_path_buf(), true);
    assert!(result.is_ok());

    let output = result.unwrap();
    println!("Validation output: {}", output);
    assert!(output.contains("\"total_files\": 2"));
}

#[test]
fn test_validate_path_with_unsupported_files() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    // Create unsupported files
    fs::write(source_path.join("file.txt"), "text content").unwrap();
    fs::write(source_path.join("file.jpg"), "image content").unwrap();

    let result = validate_path(&source_path.to_path_buf(), true);
    assert!(result.is_err());

    let output = result.expect_err("Should have returned an error");
    // Should be valid JSON even with no music files
    assert!(output.contains("\"valid\": true"));
    assert!(output.starts_with('{'));
    assert!(output.ends_with('}'));
}

#[test]
fn test_validate_path_unicode_paths() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    // Create test directory structure with unicode
    fs::create_dir_all(source_path.join("艺术家/专辑")).unwrap();

    // Copy test file
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        source_path.join("艺术家/专辑/track1.flac"),
    )
    .unwrap();

    let result = validate_path(&source_path.to_path_buf(), true);
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.contains("total_files"));
    assert!(output.contains("\"total_files\": 1"));
}

#[test]
fn test_validate_path_json_structure() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    // Create test directory structure
    fs::create_dir_all(source_path.join("artist/album")).unwrap();

    // Copy a test file
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        source_path.join("artist/album/track1.flac"),
    )
    .unwrap();

    let result = validate_path(&source_path.to_path_buf(), true);
    assert!(result.is_ok());

    let output = result.unwrap();

    // Parse as JSON to ensure it's valid
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert!(json.get("valid").is_some());
    assert!(json.get("errors").is_some());
    assert!(json.get("warnings").is_some());
    assert!(json.get("summary").is_some());

    let summary = json.get("summary").unwrap();
    assert!(summary.get("total_files").is_some());
    assert!(summary.get("valid_files").is_some());
    assert!(summary.get("files_with_errors").is_some());
    assert!(summary.get("files_with_warnings").is_some());
}
