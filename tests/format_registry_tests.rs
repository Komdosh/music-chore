//! Tests for the audio format registry

use music_chore::adapters::audio_formats::{
    create_audio_registry, get_supported_extensions, is_format_supported, read_metadata,
    write_metadata,
};
use std::path::PathBuf;

#[test]
fn test_registry_supported_extensions() {
    let registry = create_audio_registry();
    let extensions = registry.supported_extensions();

    // Should support FLAC, MP3, WAV, DSF, and WavPack
    assert!(extensions.contains(&"flac".to_string()));
    assert!(extensions.contains(&"mp3".to_string()));
    assert!(extensions.contains(&"wav".to_string()));
    assert!(extensions.contains(&"dsf".to_string()));
    assert!(extensions.contains(&"wv".to_string()));
    assert_eq!(extensions.len(), 5);
}

#[test]
fn test_is_format_supported() {
    // Supported formats
    assert!(is_format_supported(&PathBuf::from("test.flac")));
    assert!(is_format_supported(&PathBuf::from("test.FLAC")));
    assert!(is_format_supported(&PathBuf::from("test.mp3")));
    assert!(is_format_supported(&PathBuf::from("test.MP3")));
    assert!(is_format_supported(&PathBuf::from("test.wav")));
    assert!(is_format_supported(&PathBuf::from("test.WAV")));
    assert!(is_format_supported(&PathBuf::from("test.dsf")));
    assert!(is_format_supported(&PathBuf::from("test.DSF")));
    assert!(is_format_supported(&PathBuf::from("test.wv")));
    assert!(is_format_supported(&PathBuf::from("test.WV")));

    // Unsupported formats
    assert!(!is_format_supported(&PathBuf::from("test.ogg")));
    assert!(!is_format_supported(&PathBuf::from("test.m4a")));
    assert!(!is_format_supported(&PathBuf::from("test.txt")));
    assert!(!is_format_supported(&PathBuf::from("test")));
}

#[test]
fn test_get_supported_extensions() {
    let extensions = get_supported_extensions();

    assert!(extensions.contains(&"flac".to_string()));
    assert!(extensions.contains(&"mp3".to_string()));
    assert!(extensions.contains(&"wav".to_string()));
    assert!(extensions.contains(&"dsf".to_string()));
    assert!(extensions.contains(&"wv".to_string()));
    assert_eq!(extensions.len(), 5);
}

#[test]
fn test_read_metadata_unsupported_format() {
    let path = PathBuf::from("test.ogg");
    let result = read_metadata(&path);
    assert!(result.is_err());
}

#[test]
fn test_read_metadata_nonexistent_file() {
    let path = PathBuf::from("nonexistent.flac");
    let result = read_metadata(&path);
    assert!(result.is_err());
}

#[test]
fn test_write_metadata_unsupported_format() {
    use music_chore::core::domain::models::TrackMetadata;

    let path = PathBuf::from("test.ogg");
    let metadata = TrackMetadata {
        title: None,
        artist: None,
        album: None,
        album_artist: None,
        track_number: None,
        disc_number: None,
        year: None,
        genre: None,
        duration: None,
        format: "ogg".to_string(),
        path: path.clone(),
    };
    let result = write_metadata(&path, &metadata);
    assert!(result.is_err());
}

#[test]
fn test_read_metadata_success_for_flac() {
    use std::fs;
    use tempfile::TempDir;
    let temp_dir = TempDir::new().unwrap();
    let flac_file = temp_dir.path().join("test.flac");
    fs::write(&flac_file, b"dummy flac content").unwrap();

    let result = read_metadata(&flac_file);
    // Expect an error because it's dummy content, but it should be an InvalidFile error
    // indicating the correct handler was found and tried to process it.
    assert!(result.is_err());
    assert!(format!("{:?}", result).contains("InvalidFile"));
}

#[test]
fn test_write_metadata_success_for_flac() {
    use music_chore::core::domain::models::{MetadataValue, TrackMetadata};
    use std::fs;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let flac_file = temp_dir.path().join("test.flac");
    fs::write(&flac_file, b"dummy flac content").unwrap();

    let metadata = TrackMetadata {
        title: Some(MetadataValue::embedded("Test Title".to_string())),
        artist: None,
        album: None,
        album_artist: None,
        track_number: None,
        disc_number: None,
        year: None,
        genre: None,
        duration: None,
        format: "flac".to_string(),
        path: flac_file.clone(),
    };

    let result = write_metadata(&flac_file, &metadata);
    // Expect an error because it's dummy content, but it should be an InvalidFile error
    // indicating the correct handler was found and tried to process it.
    assert!(result.is_err());
    assert!(format!("{:?}", result).contains("InvalidFile"));
}
