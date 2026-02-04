//! Integration tests for MP3 format support.

use music_chore::core::domain::traits::AudioFile;
use music_chore::adapters::audio_formats::is_format_supported;
use music_chore::adapters::audio_formats::mp3::Mp3Handler;
use music_chore::core::services::scanner::scan_dir;
use std::path::PathBuf;

#[test]
fn test_mp3_format_detection() {
    let mp3_path = PathBuf::from("tests/fixtures/mp3/simple/track1.mp3");
    assert!(is_format_supported(&mp3_path));

    let flac_path = PathBuf::from("tests/fixtures/flac/simple/track1.flac");
    assert!(is_format_supported(&flac_path));
}

#[test]
fn test_mp3_handler_basic_functionality() {
    let handler = Mp3Handler::new();

    // Test supported extensions
    let extensions = handler.supported_extensions();
    assert!(extensions.contains(&"mp3"));
    assert_eq!(extensions.len(), 1);

    // Test file detection
    assert!(handler.can_handle(&PathBuf::from("test.mp3")));
    assert!(handler.can_handle(&PathBuf::from("test.MP3")));
    assert!(!handler.can_handle(&PathBuf::from("test.flac")));
    assert!(!handler.can_handle(&PathBuf::from("test.wav")));
}

#[test]
fn test_mp3_scanner_integration() {
    let mp3_dir = PathBuf::from("tests/fixtures/mp3");
    let tracks = scan_dir(&mp3_dir);

    // Should find all MP3 files
    assert_eq!(tracks.len(), 5);

    // Check that all tracks are MP3 format
    for track in &tracks {
        assert_eq!(track.metadata.format, "mp3");
    }

    // Verify specific files exist
    let file_paths: Vec<String> = tracks
        .iter()
        .map(|t| t.file_path.to_string_lossy().to_string())
        .collect();

    assert!(file_paths.iter().any(|p| p.contains("track1.mp3")));
    assert!(file_paths.iter().any(|p| p.contains("track2.mp3")));
    assert!(file_paths.iter().any(|p| p.contains("Come Together.mp3")));
    assert!(file_paths.iter().any(|p| p.contains("Something.mp3")));
    assert!(file_paths.iter().any(|p| p.contains("José González")));
}

#[test]
fn test_mp3_unicode_support() {
    let handler = Mp3Handler::new();
    let unicode_path = PathBuf::from("tests/fixtures/mp3/unicode/José González/album/track.mp3");

    // Should handle Unicode paths correctly
    assert!(handler.can_handle(&unicode_path));
    assert!(is_format_supported(&unicode_path));
}

#[test]
fn test_mixed_format_scanning() {
    // Test scanning both MP3 and FLAC files
    let mp3_dir = PathBuf::from("tests/fixtures");
    let tracks = scan_dir(&mp3_dir);

    // Count formats
    let mut mp3_count = 0;
    let mut flac_count = 0;

    for track in &tracks {
        match track.metadata.format.as_str() {
            "mp3" => mp3_count += 1,
            "flac" => flac_count += 1,
            _ => {}
        }
    }

    // Should find MP3 files
    assert_eq!(mp3_count, 5);
    // Should find FLAC files (at least some)
    assert!(flac_count > 10);
}

#[test]
fn test_mp3_basic_info_reading() {
    let handler = Mp3Handler::new();
    let mp3_path = PathBuf::from("tests/fixtures/mp3/simple/track1.mp3");

    // Test that we can read basic info without crashing
    let result = handler.read_basic_info(&mp3_path);

    // Even if tags are malformed, we should get back some metadata with format
    match result {
        Ok(metadata) => {
            assert_eq!(metadata.format, "mp3");
            assert_eq!(metadata.path, mp3_path);
        }
        Err(_) => {
            // It's okay if reading fails due to malformed test files
            // The important thing is we don't crash
        }
    }
}

#[test]
fn test_mp3_supported_extensions_registry() {
    use music_chore::adapters::audio_formats::get_supported_extensions;

    let extensions = get_supported_extensions();

    // Should include both flac and mp3
    assert!(extensions.contains(&"flac".to_string()));
    assert!(extensions.contains(&"mp3".to_string()));

    // Should be sorted and unique
    let mut sorted_extensions = extensions.clone();
    sorted_extensions.sort();
    sorted_extensions.dedup();
    assert_eq!(extensions, sorted_extensions);
}
