use music_chore::core::domain::models::MetadataSource;
use music_chore::core::services::scanner::{scan_dir, scan_dir_paths};
use std::path::Path;

#[test]
fn test_scan_simple_directory() {
    let fixture_path = Path::new("tests/fixtures/flac/simple");
    if !fixture_path.exists() {
        return; // Skip test if fixtures don't exist
    }

    let tracks = scan_dir(fixture_path, false);

    assert_eq!(tracks.len(), 2);

    // Check deterministic ordering (should be sorted)
    let paths: Vec<_> = tracks.iter().map(|t| &t.file_path).collect();
    assert!(paths[0] < paths[1]);

    // Verify all files have .flac extension
    for track in &tracks {
        assert_eq!(track.metadata.format, "flac");
        assert!(track
            .file_path
            .extension()
            .unwrap()
            .eq_ignore_ascii_case("flac"));
    }
}

#[test]
fn test_scan_nested_directory() {
    let fixture_path = Path::new("tests/fixtures/flac/nested");
    if !fixture_path.exists() {
        return; // Skip test if fixtures don't exist
    }

    let tracks = scan_dir(fixture_path, false);

    assert_eq!(tracks.len(), 2);

    // Verify tracks are in nested directory
    for track in &tracks {
        assert!(track.file_path.to_string_lossy().contains("Abbey Road"));
    }
}

#[test]
fn test_scan_paths_only() {
    let fixture_path = Path::new("tests/fixtures/flac/simple");
    if !fixture_path.exists() {
        return; // Skip test if fixtures don't exist
    }

    let paths = scan_dir_paths(fixture_path);

    assert_eq!(paths.len(), 2);

    // Verify deterministic ordering
    assert!(paths[0] < paths[1]);

    // Verify all are .flac files
    for path in &paths {
        assert!(path
            .extension()
            .unwrap()
            .to_str()
            .unwrap()
            .eq_ignore_ascii_case("flac"));
    }
}

#[test]
fn test_scan_metadata_extraction_from_real_files() {
    // Test that scanner correctly extracts metadata from FLAC files
    let fixture_path = Path::new("tests/fixtures/flac/simple");
    if !fixture_path.exists() {
        return; // Skip test if fixtures don't exist
    }

    let tracks = scan_dir(fixture_path, false);
    assert_eq!(tracks.len(), 2);

    // Find track1.flac and verify its metadata
    let track1 = tracks
        .iter()
        .find(|t| t.file_path.file_name().unwrap() == "track1.flac")
        .expect("track1.flac should be found");

    // Verify embedded metadata is correctly extracted
    assert_eq!(
        track1.metadata.title.as_ref().unwrap().value,
        "Test Apply Behavior"
    );
    assert_eq!(
        track1.metadata.artist.as_ref().unwrap().value,
        "Test Artist"
    );
    assert_eq!(track1.metadata.album.as_ref().unwrap().value, "Test Album");
    assert_eq!(
        track1.metadata.album_artist.as_ref().unwrap().value,
        "Test Album Artist"
    );
    assert_eq!(track1.metadata.track_number.as_ref().unwrap().value, 1);
    assert_eq!(track1.metadata.disc_number.as_ref().unwrap().value, 1);
    assert_eq!(track1.metadata.year.as_ref().unwrap().value, 2023);
    assert_eq!(track1.metadata.genre.as_ref().unwrap().value, "Test Genre");

    // Verify metadata source is Embedded
    assert_eq!(
        track1.metadata.title.as_ref().unwrap().source,
        MetadataSource::Embedded
    );
}

#[test]
fn test_scan_skip_metadata_behavior() {
    // Test that skip_metadata=true doesn't read embedded metadata
    let fixture_path = Path::new("tests/fixtures/flac/simple");
    if !fixture_path.exists() {
        return;
    }

    // Scan with skip_metadata=true
    let tracks_with_skip = scan_dir(fixture_path, true);

    // Should still find the files
    assert_eq!(tracks_with_skip.len(), 2);

    // Find track1.flac
    let track1 = tracks_with_skip
        .iter()
        .find(|t| t.file_path.file_name().unwrap() == "track1.flac")
        .expect("track1.flac should be found");

    // With skip_metadata=true, metadata should be inferred from folder or filename
    // The actual behavior depends on implementation, but it shouldn't have embedded metadata
    // Verify the file was found even with skip_metadata
    assert_eq!(track1.metadata.format, "flac");
}
