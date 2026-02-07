use music_chore::core::services::scanner::scan_tracks;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_scan_tracks_text_output_with_inferred_metadata() {
    let temp_dir = tempdir().unwrap();
    let source_path = temp_dir.path();

    // Create a folder structure for inference
    let artist_dir = source_path.join("Inferred Artist");
    let album_dir = artist_dir.join("Inferred Album");
    fs::create_dir_all(&album_dir).unwrap();

    // Create a dummy audio file
    let track_file = album_dir.join("01 - Inferred Track.flac");
    fs::write(&track_file, b"dummy content").unwrap();

    let result = scan_tracks(source_path.to_path_buf(), false);
    assert!(result.is_ok());

    let output = result.unwrap();
    // Expecting "01 - Inferred Track.flac [🤖]" or similar
    assert!(
        output.contains("01 - Inferred Track.flac [🤖]"),
        "Output did not contain expected inferred track name. Full output:\n{}",
        output
    );
}

#[test]
fn test_scan_tracks_text_output_with_cue_inferred_metadata() {
    let temp_dir = tempdir().unwrap();
    let source_path = temp_dir.path();

    let album_dir = source_path.join("Artist").join("Album");
    fs::create_dir_all(&album_dir).unwrap();

    // Create dummy audio files
    let track1_flac = album_dir.join("01 - Track One.flac");
    fs::write(&track1_flac, b"dummy flac content").unwrap();

    // Create a CUE file that references these audio files
    let cue_content = format!(
        r#"REM GENRE "Electronic"
REM DATE "2023"
PERFORMER "Artist Name"
TITLE "Album Title"
FILE "{}" WAVE
  TRACK 01 AUDIO
    TITLE "Track One CUE"
    PERFORMER "Artist One CUE"
    INDEX 01 00:00:00"#,
        track1_flac.file_name().unwrap().to_str().unwrap(),
    );
    let cue_file = album_dir.join("Album.cue");
    fs::write(&cue_file, cue_content).unwrap();

    let result = scan_tracks(source_path.to_path_buf(), false);
    assert!(result.is_ok());

    let output = result.unwrap();
    // Expecting "Track One CUE (01 - Track One.flac) [📄]"
    assert!(
        output.contains("Track One CUE (01 - Track One.flac) [📄]"),
        "Output did not contain expected CUE-inferred track name. Full output:\n{}",
        output
    );
}

#[test]
fn test_scan_tracks_text_output_with_embedded_metadata() {
    let temp_dir = tempdir().unwrap();
    let source_path = temp_dir.path();

    // Use an existing fixture with embedded metadata
    let fixture_path = PathBuf::from("tests/fixtures/flac/metadata/test_with_metadata.flac");
    let target_path = source_path.join("test_with_metadata.flac");
    fs::copy(&fixture_path, &target_path).unwrap();

    let result = scan_tracks(source_path.to_path_buf(), false);
    assert!(result.is_ok());

    let output = result.unwrap();
    // Expecting "Test Track With Metadata [🎯]" or similar, with the embedded title
    assert!(
        output.contains("Test Track With Metadata [🎯]"),
        "Output did not contain expected embedded track name. Full output:\n{}",
        output
    );
}
