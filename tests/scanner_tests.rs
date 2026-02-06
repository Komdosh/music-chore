use music_chore::core::services::scanner::{scan_dir, scan_dir_paths};
use std::path::Path;

#[test]
fn test_scan_simple_directory() {
    let fixture_path = Path::new("tests/fixtures/flac/simple");
    if !fixture_path.exists() {
        return; // Skip test if fixtures don't exist
    }

    let tracks = scan_dir(fixture_path);

    assert_eq!(tracks.len(), 2);

    // Check deterministic ordering (should be sorted)
    let paths: Vec<_> = tracks.iter().map(|t| &t.file_path).collect();
    assert!(paths[0] < paths[1]);

    // Verify all files have .flac extension
    for track in &tracks {
        assert_eq!(track.metadata.format, "flac");
        assert!(
            track
                .file_path
                .extension()
                .unwrap()
                .eq_ignore_ascii_case("flac")
        );
    }
}

#[test]
fn test_scan_nested_directory() {
    let fixture_path = Path::new("tests/fixtures/flac/nested");
    if !fixture_path.exists() {
        return; // Skip test if fixtures don't exist
    }

    let tracks = scan_dir(fixture_path);

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
        assert!(
            path.extension()
                .unwrap()
                .to_str()
                .unwrap()
                .eq_ignore_ascii_case("flac")
        );
    }
}
