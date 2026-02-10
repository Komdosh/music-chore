use music_chore::core::services::scanner::{
    scan_dir_with_metadata, scan_tracks, scan_with_duplicates,
};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_scan_dir_with_metadata_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let result = scan_dir_with_metadata(temp_dir.path());

    assert!(result.is_ok());
    let tracks = result.unwrap();
    assert_eq!(tracks.len(), 0);
}

#[test]
fn test_scan_dir_with_metadata_nonexistent_directory() {
    let nonexistent_path = PathBuf::from("/nonexistent/path");
    let result = scan_dir_with_metadata(&nonexistent_path);

    assert!(result.is_ok());
    let tracks = result.unwrap();
    assert_eq!(tracks.len(), 0);
}

#[test]
fn test_scan_with_duplicates_no_duplicates() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    // Create test directory structure
    fs::create_dir_all(source_path.join("artist/album")).unwrap();

    // Copy different test files
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

    let (tracks, duplicates) = scan_with_duplicates(source_path, false);

    assert_eq!(tracks.len(), 2);
    assert_eq!(duplicates.len(), 0);
    assert!(tracks[0].checksum.is_some());
    assert!(tracks[1].checksum.is_some());
    assert_ne!(tracks[0].checksum, tracks[1].checksum);
}

#[test]
fn test_scan_with_duplicates_with_duplicates() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    // Create test directory structure
    fs::create_dir_all(source_path.join("artist/album")).unwrap();

    // Copy the same file twice (identical duplicates)
    let test_file_source = "tests/fixtures/flac/simple/track1.flac";
    fs::copy(
        test_file_source,
        source_path.join("artist/album/track1.flac"),
    )
    .unwrap();
    fs::copy(
        test_file_source,
        source_path.join("artist/album/track1_copy.flac"),
    )
    .unwrap();

    let (tracks, duplicates) = scan_with_duplicates(source_path, false);

    assert_eq!(tracks.len(), 2);
    assert_eq!(duplicates.len(), 1);
    assert_eq!(duplicates[0].len(), 2);

    // Both tracks should have the same checksum
    assert_eq!(tracks[0].checksum, tracks[1].checksum);
}

#[test]
fn test_scan_tracks_json_output() {
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

    let result = scan_tracks(source_path.to_path_buf(), true);
    assert!(result.is_ok());

    let output = result.unwrap();
    // Check that it's valid JSON
    let json_value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(json_value.is_array());
    assert!(output.contains("track1.flac"));
}

#[test]
fn test_scan_tracks_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let result = scan_tracks(temp_dir.path().to_path_buf(), false);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No music files found"));
}

#[test]
fn test_scan_tracks_nonexistent_directory() {
    let nonexistent_path = PathBuf::from("/nonexistent/path");
    let result = scan_tracks(nonexistent_path, false);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No music files found"));
}

#[test]
fn test_scan_with_duplicates_mixed_formats() {
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

    let (tracks, duplicates) = scan_with_duplicates(source_path, false);

    assert_eq!(tracks.len(), 3);
    assert_eq!(duplicates.len(), 0);
    assert!(tracks.iter().all(|t| t.checksum.is_some()));
}

#[test]
fn test_scan_dir_with_metadata_preserves_order() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    // Create test directory structure
    fs::create_dir_all(source_path.join("artist/album")).unwrap();

    // Copy files in a specific order
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        source_path.join("artist/album/z_track.flac"),
    )
    .unwrap();
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        source_path.join("artist/album/a_track.flac"),
    )
    .unwrap();
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        source_path.join("artist/album/b_track.flac"),
    )
    .unwrap();

    let result = scan_dir_with_metadata(source_path);
    assert!(result.is_ok());

    let tracks = result.unwrap();
    assert_eq!(tracks.len(), 3);

    // Check that tracks are sorted by path (BTreeMap ensures this)
    let paths: Vec<String> = tracks
        .iter()
        .map(|t| {
            t.file_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
        })
        .collect();
    assert_eq!(paths, vec!["a_track.flac", "b_track.flac", "z_track.flac"]);
}
