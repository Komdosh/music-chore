//! Integration tests for OGG format support.

use music_chore::adapters::audio_formats::{
    get_supported_extensions, is_format_supported, read_basic_info, read_metadata, write_metadata,
};
use music_chore::core::domain::models::{MetadataValue, TrackMetadata};
use music_chore::core::services::scanner::scan_dir;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_ogg_format_detection() {
    assert!(is_format_supported(&PathBuf::from("track.ogg")));
    assert!(is_format_supported(&PathBuf::from("track.OGG")));
}

#[test]
fn test_ogg_supported_extensions_registry() {
    let extensions = get_supported_extensions();
    assert!(extensions.contains(&"ogg".to_string()));
}

#[test]
fn test_ogg_scan_integration_with_invalid_file_content() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let source_path = temp_dir.path();

    fs::create_dir_all(source_path.join("Artist/Album")).expect("test dirs should be created");
    let ogg_path = source_path.join("Artist/Album/01 - Test Track.ogg");
    fs::write(&ogg_path, "not real ogg bytes").expect("test ogg should be written");

    let tracks = scan_dir(source_path, false);
    assert_eq!(tracks.len(), 1);

    let track = &tracks[0];
    assert_eq!(track.metadata.format, "ogg");
    assert!(track.file_path.to_string_lossy().ends_with(".ogg"));

    // Even when embedded parsing fails, folder inference should populate artist/album.
    assert!(track.metadata.artist.is_some());
    assert!(track.metadata.album.is_some());
}

#[test]
fn test_ogg_scan_skip_metadata_inference() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let source_path = temp_dir.path();

    fs::create_dir_all(source_path.join("Artist/Album")).expect("test dirs should be created");
    let ogg_path = source_path.join("Artist/Album/02 - Another Track.ogg");
    fs::write(&ogg_path, "invalid ogg bytes").expect("test ogg should be written");

    let tracks = scan_dir(source_path, true);
    assert_eq!(tracks.len(), 1);

    let track = &tracks[0];
    assert_eq!(track.metadata.format, "ogg");
    assert_eq!(
        track.metadata.artist.as_ref().map(|a| a.value.as_str()),
        Some("Artist")
    );
    assert_eq!(
        track.metadata.album.as_ref().map(|a| a.value.as_str()),
        Some("Album")
    );
}

#[test]
fn test_ogg_scan_unicode_path_inference() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let source_path = temp_dir.path();

    fs::create_dir_all(source_path.join("José González/álbum"))
        .expect("unicode test dirs should be created");
    let ogg_path = source_path.join("José González/álbum/track.ogg");
    fs::write(&ogg_path, "invalid ogg bytes").expect("test ogg should be written");

    let tracks = scan_dir(source_path, false);
    assert_eq!(tracks.len(), 1);

    let track = &tracks[0];
    assert_eq!(track.metadata.format, "ogg");
    assert_eq!(
        track.metadata.artist.as_ref().map(|a| a.value.as_str()),
        Some("José González")
    );
    assert_eq!(
        track.metadata.album.as_ref().map(|a| a.value.as_str()),
        Some("álbum")
    );
}

#[test]
fn test_ogg_read_metadata_invalid_content_routes_to_ogg_handler() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let ogg_path = temp_dir.path().join("bad.ogg");
    fs::write(&ogg_path, "invalid ogg bytes").expect("test ogg should be written");

    let result = read_metadata(&ogg_path);
    assert!(result.is_err());

    let err_str = result.err().expect("error should exist").to_string();
    assert!(err_str.contains("Invalid file"));
    assert!(err_str.contains("OGG") || err_str.contains("ogg"));
}

#[test]
fn test_ogg_write_metadata_invalid_content_routes_to_ogg_handler() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let ogg_path = temp_dir.path().join("bad.ogg");
    fs::write(&ogg_path, "invalid ogg bytes").expect("test ogg should be written");

    let metadata = TrackMetadata {
        title: Some(MetadataValue::user_set("Title".to_string())),
        artist: Some(MetadataValue::user_set("Artist".to_string())),
        album: Some(MetadataValue::user_set("Album".to_string())),
        album_artist: None,
        track_number: None,
        disc_number: None,
        year: None,
        genre: None,
        duration: None,
        format: "ogg".to_string(),
        path: ogg_path.clone(),
    };

    let result = write_metadata(&ogg_path, &metadata);
    assert!(result.is_err());

    let err_str = result.err().expect("error should exist").to_string();
    assert!(err_str.contains("Invalid file"));
    assert!(err_str.contains("OGG") || err_str.contains("ogg"));
}

#[test]
fn test_ogg_read_basic_info_invalid_content_routes_to_ogg_handler() {
    let temp_dir = TempDir::new().expect("temp dir should be created");
    let ogg_path = temp_dir.path().join("bad.ogg");
    fs::write(&ogg_path, "invalid ogg bytes").expect("test ogg should be written");

    let result = read_basic_info(&ogg_path);
    assert!(result.is_err());

    let err_str = result.err().expect("error should exist").to_string();
    assert!(err_str.contains("Invalid file"));
    assert!(err_str.contains("OGG") || err_str.contains("ogg"));
}
