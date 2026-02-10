use music_chore::core::services::scanner::{scan_dir, scan_dir_with_depth};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_scan_dir_warns_on_unsupported_format() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    fs::create_dir_all(source_path.join("album")).unwrap();

    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        source_path.join("album/track1.flac"),
    )
    .unwrap();

    fs::write(
        source_path.join("album/also_unsupported.ogg"),
        "fake ogg content",
    )
    .unwrap();

    let tracks = scan_dir(source_path, false);

    assert_eq!(tracks.len(), 1);
    assert!(
        tracks[0]
            .file_path
            .to_string_lossy()
            .contains("track1.flac")
    );
}

#[test]
fn test_scan_dir_with_depth_warns_on_unsupported_format() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    fs::create_dir_all(source_path.join("album")).unwrap();

    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        source_path.join("album/track1.flac"),
    )
    .unwrap();

    let tracks = scan_dir_with_depth(source_path, Some(5));

    assert_eq!(tracks.len(), 1);
}

#[test]
fn test_scan_dir_no_warn_on_non_audio_files() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    fs::create_dir_all(source_path.join("album")).unwrap();

    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        source_path.join("album/track1.flac"),
    )
    .unwrap();

    fs::write(source_path.join("photo.jpg"), "fake jpg").unwrap();
    fs::write(source_path.join("readme.txt"), "fake txt").unwrap();
    fs::write(source_path.join("document.pdf"), "fake pdf").unwrap();

    let tracks = scan_dir(source_path, false);

    assert_eq!(tracks.len(), 1);
}

#[test]
fn test_scan_dir_multiple_unsupported_formats() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    fs::create_dir_all(source_path.join("album")).unwrap();

    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        source_path.join("album/track1.flac"),
    )
    .unwrap();

    fs::write(source_path.join("album/track.ogg"), "ogg").unwrap();
    fs::write(source_path.join("album/track.m4a"), "m4a").unwrap();
    fs::write(source_path.join("album/track.aiff"), "aiff").unwrap();

    let tracks = scan_dir(source_path, false);

    assert_eq!(tracks.len(), 1);
}

#[test]
fn test_scan_dir_nested_unsupported_formats() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    fs::create_dir_all(source_path.join("album/subdir")).unwrap();

    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        source_path.join("album/track1.flac"),
    )
    .unwrap();
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        source_path.join("album/subdir/track2.flac"),
    )
    .unwrap();
    fs::write(
        source_path.join("album/subdir/nested_unsupported.dsf"),
        "dsf",
    )
    .unwrap();

    fs::write(source_path.join("album/unsupported.ogg"), "ogg").unwrap();

    let tracks = scan_dir(source_path, false);

    assert_eq!(tracks.len(), 3);
}

#[test]
fn test_scan_dir_only_unsupported_formats() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    fs::create_dir_all(source_path.join("album")).unwrap();

    fs::write(source_path.join("album/track.vvs"), "vvs").unwrap();
    fs::write(source_path.join("album/track.ogg"), "ogg").unwrap();

    let tracks = scan_dir(source_path, false);

    assert_eq!(tracks.len(), 0);
}

#[test]
fn test_scan_dir_mixed_supported_and_unsupported() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    fs::create_dir_all(source_path.join("album")).unwrap();

    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        source_path.join("album/track1.flac"),
    )
    .unwrap();
    fs::copy(
        "tests/fixtures/mp3/simple/track1.mp3",
        source_path.join("album/track2.mp3"),
    )
    .unwrap();
    fs::copy(
        "tests/fixtures/wav/simple/track1.wav",
        source_path.join("album/track3.wav"),
    )
    .unwrap();

    fs::write(source_path.join("album/unsupported.xyz"), "xyz").unwrap();

    let tracks = scan_dir(source_path, false);

    assert_eq!(tracks.len(), 3);
}

#[test]
fn test_scan_dir_with_depth_limits_warnings() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    fs::create_dir_all(source_path.join("level1/level2")).unwrap();

    // Track at root level
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        source_path.join("track_root.flac"),
    )
    .unwrap();
    // Track at level 1
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        source_path.join("level1/track_l1.flac"),
    )
    .unwrap();
    // Track at level 2
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        source_path.join("level1/level2/track_l2.flac"),
    )
    .unwrap();

    fs::write(source_path.join("level1/unsupported.ogg"), "ogg").unwrap();

    // Depth 0: immediate files only (root level files)
    let tracks_depth_0 = scan_dir_with_depth(source_path, Some(0));
    assert_eq!(tracks_depth_0.len(), 1);

    // Depth 1: root + level1
    let tracks_depth_1 = scan_dir_with_depth(source_path, Some(1));
    assert_eq!(tracks_depth_1.len(), 2);

    // Depth 2: root + level1 + level2
    let tracks_depth_2 = scan_dir_with_depth(source_path, Some(2));
    assert_eq!(tracks_depth_2.len(), 3);
}
