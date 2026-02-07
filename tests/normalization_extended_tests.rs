use music_chore::core::services::normalization::{normalize_and_format, TitleNormalizationReport, GenreNormalizationReport, CombinedNormalizationReport};
use music_chore::adapters::audio_formats::{read_metadata, write_metadata};
use music_chore::core::domain::models::{MetadataSource, MetadataValue, TrackMetadata};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use serde_json;

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
fn test_normalize_combined_human_output_single_file_and_no_change_summary() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    fs::create_dir_all(source_path.join("artist/album")).unwrap();

    let test_file1 = source_path.join("artist/album/track1.flac"); // "Test Song", "Rock" - no change expected
    let test_file2 = source_path.join("artist/album/track2.flac"); // "track2", "punk" - should be normalized

    create_audio_file(&test_file1, &PathBuf::from("tests/fixtures/flac/simple/track1.flac"), Some("Test Song"), Some("Rock"));
    create_audio_file(&test_file2, &PathBuf::from("tests/fixtures/flac/simple/track1.flac"), Some("track2"), Some("punk"));

    let result = normalize_and_format(source_path.to_path_buf(), false); // human output
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.contains("--- Title Normalization ---"));
    assert!(output.contains("NO CHANGE: Title 'Test Song' already normalized in"));
    assert!(output.contains("NORMALIZED: Title 'track2' -> 'Track2' in"));
    assert!(output.contains("Title Summary: 1 normalized, 1 no change, 0 errors"));

    assert!(output.contains("--- Genre Normalization ---"));
    assert!(output.contains("NO CHANGE: Genre 'Rock' already normalized in"));
    assert!(output.contains("NORMALIZED: Genre 'punk' -> 'Punk' in"));
    assert!(output.contains("Genre Summary: 1 normalized, 1 no change, 0 errors"));
}

#[test]
fn test_normalize_combined_json_output_single_file_and_no_change() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    fs::create_dir_all(source_path.join("artist/album")).unwrap();

    let test_file1 = source_path.join("artist/album/track1.flac"); // "Test Song", "Rock" - no change expected
    let test_file2 = source_path.join("artist/album/track2.flac"); // "track2", "hip hop" - should be normalized

    create_audio_file(&test_file1, &PathBuf::from("tests/fixtures/flac/simple/track1.flac"), Some("Test Song"), Some("Rock"));
    create_audio_file(&test_file2, &PathBuf::from("tests/fixtures/flac/simple/track1.flac"), Some("track2"), Some("hip hop"));

    let result = normalize_and_format(source_path.to_path_buf(), true); // JSON output
    assert!(result.is_ok());

    let output = result.unwrap();
    let combined_report: CombinedNormalizationReport = serde_json::from_str(&output).unwrap();

    assert_eq!(combined_report.title_reports.len(), 2);
    assert_eq!(combined_report.genre_reports.len(), 2);

    let title_report1 = combined_report.title_reports.iter().find(|r| r.original_path.to_str().unwrap().contains("track1.flac")).unwrap();
    assert_eq!(title_report1.original_title, Some("Test Song".to_string()));
    assert_eq!(title_report1.normalized_title, Some("Test Song".to_string()));
    assert!(!title_report1.changed);
    assert!(title_report1.error.is_none());

    let title_report2 = combined_report.title_reports.iter().find(|r| r.original_path.to_str().unwrap().contains("track2.flac")).unwrap();
    assert_eq!(title_report2.original_title, Some("track2".to_string()));
    assert_eq!(title_report2.normalized_title, Some("Track2".to_string()));
    assert!(title_report2.changed);
    assert!(title_report2.error.is_none());

    let genre_report1 = combined_report.genre_reports.iter().find(|r| r.original_path.to_str().unwrap().contains("track1.flac")).unwrap();
    assert_eq!(genre_report1.original_genre, Some("Rock".to_string()));
    assert_eq!(genre_report1.normalized_genre, Some("Rock".to_string()));
    assert!(!genre_report1.changed);
    assert!(genre_report1.error.is_none());

    let genre_report2 = combined_report.genre_reports.iter().find(|r| r.original_path.to_str().unwrap().contains("track2.flac")).unwrap();
    assert_eq!(genre_report2.original_genre, Some("hip hop".to_string()));
    assert_eq!(genre_report2.normalized_genre, Some("Hip-Hopa".to_string())); // Note: The genre alias might need adjusting if "Hip-Hopa" is not expected.
    assert!(genre_report2.changed);
    assert!(genre_report2.error.is_none());
}

#[test]
fn test_normalize_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let result = normalize_and_format(temp_dir.path().to_path_buf(), false); // human output
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Title Summary: 0 normalized, 0 no change, 0 errors"));
    assert!(output.contains("Genre Summary: 0 normalized, 0 no change, 0 errors"));

    let result_json = normalize_and_format(temp_dir.path().to_path_buf(), true); // JSON output
    assert!(result_json.is_ok());
    let output_json = result_json.unwrap();
    let combined_report: CombinedNormalizationReport = serde_json::from_str(&output_json).unwrap();
    assert_eq!(combined_report.title_reports.len(), 0);
    assert_eq!(combined_report.genre_reports.len(), 0);
}

#[test]
fn test_normalize_nonexistent_directory() {
    let nonexistent_path = PathBuf::from("/nonexistent/path");
    let result = normalize_and_format(nonexistent_path, false);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));

    let nonexistent_path_json = PathBuf::from("/nonexistent/path_json");
    let result_json = normalize_and_format(nonexistent_path_json, true);
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

    let result = normalize_and_format(source_path.to_path_buf(), false); // human output
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Title Summary: 0 normalized, 0 no change, 0 errors"));
    assert!(output.contains("Genre Summary: 0 normalized, 0 no change, 0 errors"));

    let result_json = normalize_and_format(source_path.to_path_buf(), true); // JSON output
    assert!(result_json.is_ok());
    let output_json = result_json.unwrap();
    let combined_report: CombinedNormalizationReport = serde_json::from_str(&output_json).unwrap();
    assert_eq!(combined_report.title_reports.len(), 0);
    assert_eq!(combined_report.genre_reports.len(), 0);
}

#[test]
fn test_normalize_mixed_file_types() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    fs::create_dir_all(source_path.join("artist/album")).unwrap();

    // Mix of music and non-music files
    create_audio_file(&source_path.join("artist/album/track1.flac"), &PathBuf::from("tests/fixtures/flac/simple/track1.flac"), Some("track one"), Some("blues"));
    fs::write(source_path.join("artist/album/readme.txt"), "album info").unwrap();
    fs::write(source_path.join("artist/album/cover.jpg"), "image content").unwrap();

    let result = normalize_and_format(source_path.to_path_buf(), false); // human output
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("NORMALIZED: Title 'track one' -> 'Track One' in"));
    assert!(output.contains("Title Summary: 1 normalized, 0 no change, 0 errors"));
    assert!(output.contains("NORMALIZED: Genre 'blues' -> 'Blues' in"));
    assert!(output.contains("Genre Summary: 1 normalized, 0 no change, 0 errors"));

    let result_json = normalize_and_format(source_path.to_path_buf(), true); // JSON output
    assert!(result_json.is_ok());
    let output_json = result_json.unwrap();
    let combined_report: CombinedNormalizationReport = serde_json::from_str(&output_json).unwrap();
    assert_eq!(combined_report.title_reports.len(), 1);
    assert_eq!(combined_report.genre_reports.len(), 1);
    assert!(combined_report.title_reports[0].changed);
    assert_eq!(combined_report.title_reports[0].normalized_title, Some("Track One".to_string()));
    assert!(combined_report.genre_reports[0].changed);
    assert_eq!(combined_report.genre_reports[0].normalized_genre, Some("Blues".to_string()));
}

#[test]
fn test_normalize_combined_human_output_nested_directories() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    fs::create_dir_all(source_path.join("artist1/album1")).unwrap();
    fs::create_dir_all(source_path.join("artist2/album2")).unwrap();

    create_audio_file(&source_path.join("artist1/album1/track1.flac"), &PathBuf::from("tests/fixtures/flac/simple/track1.flac"), Some("track a"), Some("jazz"));
    create_audio_file(&source_path.join("artist2/album2/track2.flac"), &PathBuf::from("tests/fixtures/flac/simple/track1.flac"), Some("track b"), Some("metal"));

    let result = normalize_and_format(source_path.to_path_buf(), false); // human output
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.contains("NORMALIZED: Title 'track a' -> 'Track A' in"));
    assert!(output.contains("NORMALIZED: Title 'track b' -> 'Track B' in"));
    assert!(output.contains("Title Summary: 2 normalized, 0 no change, 0 errors"));
    assert!(output.contains("NORMALIZED: Genre 'jazz' -> 'Jazz' in"));
    assert!(output.contains("NORMALIZED: Genre 'metal' -> 'Metal' in"));
    assert!(output.contains("Genre Summary: 2 normalized, 0 no change, 0 errors"));
}

#[test]
fn test_normalize_combined_json_output_nested_directories() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    fs::create_dir_all(source_path.join("artist1/album1")).unwrap();
    fs::create_dir_all(source_path.join("artist2/album2")).unwrap();

    create_audio_file(&source_path.join("artist1/album1/track1.flac"), &PathBuf::from("tests/fixtures/flac/simple/track1.flac"), Some("track a"), Some("jazz"));
    create_audio_file(&source_path.join("artist2/album2/track2.flac"), &PathBuf::from("tests/fixtures/flac/simple/track1.flac"), Some("track b"), Some("metal"));

    let result = normalize_and_format(source_path.to_path_buf(), true); // JSON output
    assert!(result.is_ok());

    let output = result.unwrap();
    let combined_report: CombinedNormalizationReport = serde_json::from_str(&output).unwrap();

    assert_eq!(combined_report.title_reports.len(), 2);
    assert_eq!(combined_report.genre_reports.len(), 2);
    assert!(combined_report.title_reports.iter().any(|r| r.original_title == Some("track a".to_string()) && r.normalized_title == Some("Track A".to_string())));
    assert!(combined_report.title_reports.iter().any(|r| r.original_title == Some("track b".to_string()) && r.normalized_title == Some("Track B".to_string())));
    assert!(combined_report.genre_reports.iter().any(|r| r.original_genre == Some("jazz".to_string()) && r.normalized_genre == Some("Jazz".to_string())));
    assert!(combined_report.genre_reports.iter().any(|r| r.original_genre == Some("metal".to_string()) && r.normalized_genre == Some("Metal".to_string())));
}

#[test]
fn test_normalize_combined_human_output_different_formats() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    fs::create_dir_all(source_path.join("artist/album")).unwrap();

    // Copy different format files and set metadata
    create_audio_file(&source_path.join("artist/album/track1.flac"), &PathBuf::from("tests/fixtures/flac/simple/track1.flac"), Some("flac title"), Some("rock"));
    create_audio_file(&source_path.join("artist/album/track2.mp3"), &PathBuf::from("tests/fixtures/mp3/simple/track1.mp3"), Some("mp3_title_needs_norm"), Some("hip hop"));
    // For WAV, just copy the fixture; title will be inferred from filename "track3.wav"
    fs::copy(&PathBuf::from("tests/fixtures/wav/simple/track1.wav"), &source_path.join("artist/album/track3.wav")).unwrap();
    // For WAV, metadata cannot be written easily without existing tags, so rely on filename inference and no genre
    let mut track3_metadata = read_metadata(&source_path.join("artist/album/track3.wav")).unwrap().metadata;
    track3_metadata.title = Some(MetadataValue::user_set("wav title needs norm".to_string()));
    track3_metadata.genre = Some(MetadataValue::user_set("blues".to_string()));
    write_metadata(&source_path.join("artist/album/track3.wav"), &track3_metadata).unwrap();


    let result = normalize_and_format(source_path.to_path_buf(), false); // human output
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.contains("NORMALIZED: Title 'flac title' -> 'Flac Title' in"));
    assert!(output.contains("NORMALIZED: Title 'mp3_title_needs_norm' -> 'Mp3_Title_Needs_Norm' in"));
    assert!(output.contains("NORMALIZED: Title 'wav title needs norm' -> 'Wav Title Needs Norm' in"));
    assert!(output.contains("Title Summary: 3 normalized, 0 no change, 0 errors"));

    assert!(output.contains("NORMALIZED: Genre 'rock' -> 'Rock' in"));
    assert!(output.contains("NORMALIZED: Genre 'hip hop' -> 'Hip-Hopa' in"));
    assert!(output.contains("NORMALIZED: Genre 'blues' -> 'Blues' in"));
    assert!(output.contains("Genre Summary: 3 normalized, 0 no change, 0 errors"));
}

#[test]
fn test_normalize_combined_json_output_different_formats() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    fs::create_dir_all(source_path.join("artist/album")).unwrap();

    create_audio_file(&source_path.join("artist/album/track1.flac"), &PathBuf::from("tests/fixtures/flac/simple/track1.flac"), Some("flac title"), Some("rock"));
    create_audio_file(&source_path.join("artist/album/track2.mp3"), &PathBuf::from("tests/fixtures/mp3/simple/track1.mp3"), Some("mp3_title_needs_norm"), Some("hip hop"));
    fs::copy(&PathBuf::from("tests/fixtures/wav/simple/track1.wav"), &source_path.join("artist/album/track3.wav")).unwrap();
    let mut track3_metadata = read_metadata(&source_path.join("artist/album/track3.wav")).unwrap().metadata;
    track3_metadata.title = Some(MetadataValue::user_set("wav title needs norm".to_string()));
    track3_metadata.genre = Some(MetadataValue::user_set("country".to_string()));
    write_metadata(&source_path.join("artist/album/track3.wav"), &track3_metadata).unwrap();


    let result = normalize_and_format(source_path.to_path_buf(), true); // JSON output
    assert!(result.is_ok());

    let output = result.unwrap();
    let combined_report: CombinedNormalizationReport = serde_json::from_str(&output).unwrap();

    assert_eq!(combined_report.title_reports.len(), 3);
    assert!(combined_report.title_reports.iter().any(|r| r.original_title == Some("flac title".to_string()) && r.normalized_title == Some("Flac Title".to_string())));
    assert!(combined_report.title_reports.iter().any(|r| r.original_title == Some("mp3_title_needs_norm".to_string()) && r.normalized_title == Some("Mp3_Title_Needs_Norm".to_string())));
    assert!(combined_report.title_reports.iter().any(|r| r.original_title == Some("wav title needs norm".to_string()) && r.normalized_title == Some("Wav Title Needs Norm".to_string())));

    assert_eq!(combined_report.genre_reports.len(), 3);
    assert!(combined_report.genre_reports.iter().any(|r| r.original_genre == Some("rock".to_string()) && r.normalized_genre == Some("Rock".to_string())));
    assert!(combined_report.genre_reports.iter().any(|r| r.original_genre == Some("hip hop".to_string()) && r.normalized_genre == Some("Hip-Hopa".to_string())));
    assert!(combined_report.genre_reports.iter().any(|r| r.original_genre == Some("country".to_string()) && r.normalized_genre == Some("Country".to_string())));
}

#[test]
fn test_normalize_unicode_paths() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    fs::create_dir_all(source_path.join("艺术家/专辑")).unwrap();

    create_audio_file(&source_path.join("艺术家/专辑/track_unicode.flac"), &PathBuf::from("tests/fixtures/flac/simple/track1.flac"), Some("unicode title"), Some("world music"));

    let result = normalize_and_format(source_path.to_path_buf(), false); // human output
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("NORMALIZED: Title 'unicode title' -> 'Unicode Title' in"));
    assert!(output.contains("Title Summary: 1 normalized, 0 no change, 0 errors"));
    assert!(output.contains("NORMALIZED: Genre 'world music' -> 'World' in"));
    assert!(output.contains("Genre Summary: 1 normalized, 0 no change, 0 errors"));

    let result_json = normalize_and_format(source_path.to_path_buf(), true); // JSON output
    assert!(result_json.is_ok());
    let output_json = result_json.unwrap();
    let combined_report: CombinedNormalizationReport = serde_json::from_str(&output_json).unwrap();
    assert_eq!(combined_report.title_reports.len(), 1);
    assert!(combined_report.title_reports[0].changed);
    assert_eq!(combined_report.title_reports[0].normalized_title, Some("Unicode Title".to_string()));
    assert_eq!(combined_report.genre_reports.len(), 1);
    assert!(combined_report.genre_reports[0].changed);
    assert_eq!(combined_report.genre_reports[0].normalized_genre, Some("World".to_string()));
}