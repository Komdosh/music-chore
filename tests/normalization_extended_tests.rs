use music_chore::core::services::normalization::{normalize, normalize_genres_in_library};
use music_chore::adapters::audio_formats::{read_metadata, write_metadata};
use music_chore::core::domain::models::{MetadataSource, MetadataValue, TrackMetadata};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

// Generic helper to create an audio file from a fixture and set its metadata
fn create_audio_file(path: &PathBuf, fixture_path: &Path, title: Option<&str>, genre: Option<&str>) {
    fs::copy(fixture_path, path).unwrap();

    let mut track_metadata = read_metadata(path).unwrap().metadata;
    if let Some(t) = title {
        track_metadata.title = Some(MetadataValue::user_set(t.to_string()));
    }
    if let Some(g) = genre {
        track_metadata.genre = Some(MetadataValue::user_set(g.to_string()));
    }
    write_metadata(path, &track_metadata).unwrap();
}

#[test]
fn test_normalize_title_human_output_single_file_and_no_change_summary() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    fs::create_dir_all(source_path.join("artist/album")).unwrap();

    let test_file1 = source_path.join("artist/album/track1.flac");
    let test_file2 = source_path.join("artist/album/track2.flac");

    create_audio_file(&test_file1, &PathBuf::from("tests/fixtures/flac/simple/track1.flac"), Some("Test Song"), None);
    create_audio_file(&test_file2, &PathBuf::from("tests/fixtures/flac/simple/track1.flac"), Some("track2"), None); // Using track1 fixture, setting title to "track2"

    let result = normalize(source_path.to_path_buf(), false); // human output
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.contains("NO CHANGE: Title 'Test Song' already normalized in"));
    assert!(output.contains("NORMALIZED: Title 'track2' -> 'Track2' in"));
    assert!(output.contains("Summary: 1 normalized, 1 no change, 0 errors"));
}

#[test]
fn test_normalize_title_json_output_single_file_and_no_change() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    fs::create_dir_all(source_path.join("artist/album")).unwrap();

    let test_file1 = source_path.join("artist/album/track1.flac");
    let test_file2 = source_path.join("artist/album/track2.flac");

    create_audio_file(&test_file1, &PathBuf::from("tests/fixtures/flac/simple/track1.flac"), Some("Test Song"), None);
    create_audio_file(&test_file2, &PathBuf::from("tests/fixtures/flac/simple/track1.flac"), Some("track2"), None);

    let result = normalize(source_path.to_path_buf(), true); // JSON output
    assert!(result.is_ok());

    let output = result.unwrap();
    let reports: Vec<music_chore::core::services::normalization::TitleNormalizationReport> =
        serde_json::from_str(&output).unwrap();

    assert_eq!(reports.len(), 2);

    let report1 = reports.iter().find(|r| r.original_path.to_str().unwrap().contains("track1.flac")).unwrap();
    assert_eq!(report1.original_title, Some("Test Song".to_string()));
    assert_eq!(report1.normalized_title, Some("Test Song".to_string()));
    assert!(!report1.changed);
    assert!(report1.error.is_none());

    let report2 = reports.iter().find(|r| r.original_path.to_str().unwrap().contains("track2.flac")).unwrap();
    assert_eq!(report2.original_title, Some("track2".to_string()));
    assert_eq!(report2.normalized_title, Some("Track2".to_string()));
    assert!(report2.changed);
    assert!(report2.error.is_none());
}

#[test]
fn test_normalize_genre_human_output_single_file() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    fs::create_dir_all(source_path.join("artist/album")).unwrap();

    let test_file1 = source_path.join("artist/album/track1.flac");
    create_audio_file(&test_file1, &PathBuf::from("tests/fixtures/flac/metadata/test_with_metadata.flac"), None, Some("rock and roll"));

    let result = normalize_genres_in_library(source_path.to_path_buf().as_path(), false); // human output
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.contains("NORMALIZED: Genre 'rock and roll' -> 'Rock' in"));
    assert!(output.contains("Summary: 1 normalized, 0 no change, 0 errors"));
}

#[test]
fn test_normalize_genre_json_output_single_file() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    fs::create_dir_all(source_path.join("artist/album")).unwrap();

    let test_file1 = source_path.join("artist/album/track1.flac");
    create_audio_file(&test_file1, &PathBuf::from("tests/fixtures/flac/metadata/test_with_metadata.flac"), None, Some("hip hop"));

    let result = normalize_genres_in_library(source_path.to_path_buf().as_path(), true); // JSON output
    assert!(result.is_ok());

    let output = result.unwrap();
    let reports: Vec<music_chore::core::services::normalization::GenreNormalizationReport> =
        serde_json::from_str(&output).unwrap();

    assert_eq!(reports.len(), 1);
    let report = &reports[0];
    assert!(report.changed);
    assert_eq!(report.original_genre, Some("hip hop".to_string()));
    assert_eq!(report.normalized_genre, Some("Hip-Hop".to_string()));
    assert!(report.error.is_none());
}


#[test]
fn test_normalize_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let result = normalize(temp_dir.path().to_path_buf(), false); // human output
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Summary: 0 normalized, 0 no change, 0 errors"));

    let result_json = normalize(temp_dir.path().to_path_buf(), true); // JSON output
    assert!(result_json.is_ok());
    let output_json = result_json.unwrap();
    assert_eq!(output_json.trim(), "[]");
}

#[test]
fn test_normalize_nonexistent_directory() {
    let nonexistent_path = PathBuf::from("/nonexistent/path");
    let result = normalize(nonexistent_path, false);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));

    let nonexistent_path_json = PathBuf::from("/nonexistent/path_json");
    let result_json = normalize(nonexistent_path_json, true);
    assert!(result_json.is_err());
    assert!(result_json.unwrap_err().contains("does not exist"));
}

#[test]
fn test_normalize_unsupported_files() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    // Create unsupported files
    fs::write(source_path.join("file.txt"), "text content").unwrap();
    fs::write(source_path.join("file.jpg"), "image content").unwrap();

    let result = normalize(source_path.to_path_buf(), false); // human output
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Summary: 0 normalized, 0 no change, 0 errors"));

    let result_json = normalize(source_path.to_path_buf(), true); // JSON output
    assert!(result_json.is_ok());
    let output_json = result_json.unwrap();
    assert_eq!(output_json.trim(), "[]");
}

#[test]
fn test_normalize_mixed_file_types() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    fs::create_dir_all(source_path.join("artist/album")).unwrap();

    // Mix of music and non-music files
    create_audio_file(&source_path.join("artist/album/track1.flac"), &PathBuf::from("tests/fixtures/flac/simple/track1.flac"), Some("track one"), None);
    fs::write(source_path.join("artist/album/readme.txt"), "album info").unwrap();
    fs::write(source_path.join("artist/album/cover.jpg"), "image content").unwrap();

    let result = normalize(source_path.to_path_buf(), false); // human output
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("NORMALIZED: Title 'track one' -> 'Track One' in"));
    assert!(output.contains("Summary: 1 normalized, 0 no change, 0 errors"));

    let result_json = normalize(source_path.to_path_buf(), true); // JSON output
    assert!(result_json.is_ok());
    let output_json = result_json.unwrap();
    let reports: Vec<music_chore::core::services::normalization::TitleNormalizationReport> =
        serde_json::from_str(&output_json).unwrap();
    assert_eq!(reports.len(), 1);
    assert!(reports[0].changed);
    assert_eq!(reports[0].normalized_title, Some("Track One".to_string()));
}

#[test]
fn test_normalize_title_human_output_nested_directories() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    fs::create_dir_all(source_path.join("artist1/album1")).unwrap();
    fs::create_dir_all(source_path.join("artist2/album2")).unwrap();

    create_audio_file(&source_path.join("artist1/album1/track1.flac"), &PathBuf::from("tests/fixtures/flac/simple/track1.flac"), Some("track a"), None);
    create_audio_file(&source_path.join("artist2/album2/track2.flac"), &PathBuf::from("tests/fixtures/flac/simple/track1.flac"), Some("track b"), None);

    let result = normalize(source_path.to_path_buf(), false); // human output
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.contains("NORMALIZED: Title 'track a' -> 'Track A' in"));
    assert!(output.contains("NORMALIZED: Title 'track b' -> 'Track B' in"));
    assert!(output.contains("Summary: 2 normalized, 0 no change, 0 errors"));
}

#[test]
fn test_normalize_title_json_output_nested_directories() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    fs::create_dir_all(source_path.join("artist1/album1")).unwrap();
    fs::create_dir_all(source_path.join("artist2/album2")).unwrap();

    create_audio_file(&source_path.join("artist1/album1/track1.flac"), &PathBuf::from("tests/fixtures/flac/simple/track1.flac"), Some("track a"), None);
    create_audio_file(&source_path.join("artist2/album2/track2.flac"), &PathBuf::from("tests/fixtures/flac/simple/track1.flac"), Some("track b"), None);

    let result = normalize(source_path.to_path_buf(), true); // JSON output
    assert!(result.is_ok());

    let output = result.unwrap();
    let reports: Vec<music_chore::core::services::normalization::TitleNormalizationReport> =
        serde_json::from_str(&output).unwrap();

    assert_eq!(reports.len(), 2);
    assert!(reports.iter().any(|r| r.original_title == Some("track a".to_string()) && r.normalized_title == Some("Track A".to_string())));
    assert!(reports.iter().any(|r| r.original_title == Some("track b".to_string()) && r.normalized_title == Some("Track B".to_string())));
}

#[test]
fn test_normalize_title_human_output_different_formats() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    fs::create_dir_all(source_path.join("artist/album")).unwrap();

    // Copy different format files and set metadata
    create_audio_file(&source_path.join("artist/album/track1.flac"), &PathBuf::from("tests/fixtures/flac/simple/track1.flac"), Some("flac title"), None);
    create_audio_file(&source_path.join("artist/album/track2.mp3"), &PathBuf::from("tests/fixtures/mp3/simple/track1.mp3"), Some("mp3_title_needs_norm"), None);
    // For WAV, just copy the fixture; title will be inferred from filename "track3.wav"
    fs::copy(&PathBuf::from("tests/fixtures/wav/simple/track1.wav"), &source_path.join("artist/album/track3.wav")).unwrap();

    let result = normalize(source_path.to_path_buf(), false); // human output
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.contains("NORMALIZED: Title 'flac title' -> 'Flac Title' in"));
    assert!(output.contains("NORMALIZED: Title 'mp3_title_needs_norm' -> 'Mp3_Title_Needs_Norm' in"));
    assert!(output.contains("NORMALIZED: Title 'track3' -> 'Track3' in")); // Filename inference for WAV
    assert!(output.contains("Summary: 3 normalized, 0 no change, 0 errors"));
}

#[test]
fn test_normalize_title_json_output_different_formats() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    fs::create_dir_all(source_path.join("artist/album")).unwrap();

    create_audio_file(&source_path.join("artist/album/track1.flac"), &PathBuf::from("tests/fixtures/flac/simple/track1.flac"), Some("flac title"), None);
    create_audio_file(&source_path.join("artist/album/track2.mp3"), &PathBuf::from("tests/fixtures/mp3/simple/track1.mp3"), Some("mp3_title_needs_norm"), None);
    // For WAV, just copy the fixture; title will be inferred from filename "track3.wav"
    fs::copy(&PathBuf::from("tests/fixtures/wav/simple/track1.wav"), &source_path.join("artist/album/track3.wav")).unwrap();

    let result = normalize(source_path.to_path_buf(), true); // JSON output
    assert!(result.is_ok());

    let output = result.unwrap();
    let reports: Vec<music_chore::core::services::normalization::TitleNormalizationReport> =
        serde_json::from_str(&output).unwrap();

    assert_eq!(reports.len(), 3);
    assert!(reports.iter().any(|r| r.original_title == Some("flac title".to_string()) && r.normalized_title == Some("Flac Title".to_string())));
    assert!(reports.iter().any(|r| r.original_title == Some("mp3_title_needs_norm".to_string()) && r.normalized_title == Some("Mp3_Title_Needs_Norm".to_string())));
    assert!(reports.iter().any(|r| r.original_title == Some("track3".to_string()) && r.normalized_title == Some("Track3".to_string()))); // Filename inference for WAV
}

#[test]
fn test_normalize_unicode_paths() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    fs::create_dir_all(source_path.join("艺术家/专辑")).unwrap();

    create_audio_file(&source_path.join("艺术家/专辑/track_unicode.flac"), &PathBuf::from("tests/fixtures/flac/simple/track1.flac"), Some("unicode title"), None);

    let result = normalize(source_path.to_path_buf(), false); // human output
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("NORMALIZED: Title 'unicode title' -> 'Unicode Title' in"));
    assert!(output.contains("Summary: 1 normalized, 0 no change, 0 errors"));

    let result_json = normalize(source_path.to_path_buf(), true); // JSON output
    assert!(result_json.is_ok());
    let output_json = result_json.unwrap();
    let reports: Vec<music_chore::core::services::normalization::TitleNormalizationReport> =
        serde_json::from_str(&output_json).unwrap();
    assert_eq!(reports.len(), 1);
    assert!(reports[0].changed);
    assert_eq!(reports[0].normalized_title, Some("Unicode Title".to_string()));
}