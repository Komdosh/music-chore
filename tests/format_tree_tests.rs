use music_chore::core::services::format_tree::{
    emit_structured_output, format_library_output, format_tree_output,
};
use music_chore::{AlbumNode, ArtistNode, Library, MetadataValue, TrackMetadata, TrackNode};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn create_test_library() -> Library {
    let track1_metadata = TrackMetadata {
        title: Some(MetadataValue::embedded("Test Track 1".to_string())),
        artist: Some(MetadataValue::embedded("Test Artist".to_string())),
        album: Some(MetadataValue::embedded("Test Album".to_string())),
        album_artist: None,
        year: Some(MetadataValue::embedded(2023)),
        track_number: Some(MetadataValue::embedded(1)),
        disc_number: Some(MetadataValue::embedded(1)),
        genre: None,
        duration: None,
        format: "FLAC".to_string(),
        path: PathBuf::from("/test/artist1/album1/track1.flac"),
    };

    let track2_metadata = TrackMetadata {
        title: Some(MetadataValue::embedded("Test Track 2".to_string())),
        artist: Some(MetadataValue::embedded("Test Artist".to_string())),
        album: Some(MetadataValue::embedded("Test Album".to_string())),
        album_artist: None,
        year: Some(MetadataValue::embedded(2023)),
        track_number: Some(MetadataValue::embedded(2)),
        disc_number: Some(MetadataValue::embedded(1)),
        genre: None,
        duration: None,
        format: "FLAC".to_string(),
        path: PathBuf::from("/test/artist1/album1/track2.flac"),
    };

    let track1_node = TrackNode {
        file_path: PathBuf::from("/test/artist1/album1/track1.flac"),
        metadata: track1_metadata.clone(),
    };

    let track2_node = TrackNode {
        file_path: PathBuf::from("/test/artist1/album1/track2.flac"),
        metadata: track2_metadata.clone(),
    };

    let album1_files: HashSet<PathBuf> = vec![
        PathBuf::from("/test/artist1/album1/track1.flac"),
        PathBuf::from("/test/artist1/album1/track2.flac"),
    ].into_iter().collect();

    Library {
        artists: vec![ArtistNode {
            name: "Test Artist".to_string(),
            albums: vec![AlbumNode {
                title: "Test Album".to_string(),
                year: Some(2023),
                tracks: vec![track1_node, track2_node],
                files: album1_files,
                path: PathBuf::from("/test/artist1/album1"),
            }],
        }],
        total_tracks: 2,
        total_artists: 1,
        total_albums: 1,
        total_files: 2,
    }
}

#[test]
fn test_format_library_output_basic() {
    let library = create_test_library();
    let output = format_library_output(&library);

    assert!(output.contains("Test Artist"));
    assert!(output.contains("Test Album (2023)"));
    assert!(output.contains("track1.flac"));
    assert!(output.contains("track2.flac"));
    assert!(output.contains("#1"));
    assert!(output.contains("#2"));
    assert!(output.contains("FLAC"));
    assert!(output.contains("🎯"));
}

#[test]
fn test_format_library_output_empty() {
    let library = Library::default();
    let output = format_library_output(&library);

    assert!(output.contains("📊 Library Summary:"));
    assert!(output.contains("Artists: 0"));
    assert!(output.contains("Albums: 0"));
    assert!(output.contains("Tracks: 0"));
}

#[test]
fn test_emit_structured_output_basic() {
    let library = create_test_library();
    let output = emit_structured_output(&library);

    assert!(output.contains("=== MUSIC LIBRARY METADATA ==="));
    assert!(output.contains("Total Artists: 1"));
    assert!(output.contains("Total Albums: 1"));
    assert!(output.contains("Total Tracks: 2"));
    assert!(output.contains("ARTIST: Test Artist"));
    assert!(output.contains("ALBUM: Test Album (2023)"));
    assert!(output.contains("TRACK: \"Test Track 1\""));
    assert!(output.contains("TRACK: \"Test Track 2\""));
}

#[test]
fn test_emit_structured_output_empty() {
    let library = Library::default();
    let output = emit_structured_output(&library);

    assert!(output.contains("=== MUSIC LIBRARY METADATA ==="));
    assert!(output.contains("Total Artists: 0"));
    assert!(output.contains("Total Albums: 0"));
    assert!(output.contains("Total Tracks: 0"));
}

#[test]
fn test_format_tree_output_with_real_files() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    // Create test directory structure
    fs::create_dir_all(source_path.join("artist/album")).unwrap();

    // Copy a test file
    let test_file_source = "tests/fixtures/flac/simple/track1.flac";
    let test_file_dest = source_path.join("artist/album/track1.flac");
    fs::copy(test_file_source, &test_file_dest).unwrap();

    let output = format_tree_output(source_path);

    assert!(output.contains("📊 Library Summary:"));
    assert!(output.contains("Files: 1"));
    assert!(output.contains("Folders:"));
    assert!(output.contains("artist"));
    assert!(output.contains("album"));
    assert!(output.contains("Test Song [🎯] FLAC")); // Updated assertion
}

#[test]
fn test_format_tree_output_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let output = format_tree_output(temp_dir.path());

    assert!(output.contains("📊 Library Summary:"));
    assert!(output.contains("Files: 0"));
    assert!(output.contains("Folders: 0"));
}

#[test]
fn test_format_tree_output_nested_structure() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path();

    // Create nested directory structure
    fs::create_dir_all(source_path.join("artist1/album1")).unwrap();
    fs::create_dir_all(source_path.join("artist2/album2")).unwrap();

    // Copy test files
    let test_file_source = "tests/fixtures/flac/simple/track1.flac";
    fs::copy(
        test_file_source,
        source_path.join("artist1/album1/track1.flac"),
    )
    .unwrap();
    fs::copy(
        test_file_source,
        source_path.join("artist2/album2/track2.flac"),
    )
    .unwrap();

    let output = format_tree_output(source_path);

    assert!(output.contains("Files: 2"));
    assert!(output.contains("artist1"));
    assert!(output.contains("artist2"));
    assert!(output.contains("album1"));
    assert!(output.contains("album2"));
    assert!(output.contains("Test Song [🎯] FLAC")); // Updated assertion for track1
    assert!(output.contains("Test Song [🎯] FLAC")); // Updated assertion for track2 (since it's also track1.flac fixture)
}

#[test]
fn test_format_library_output_multiple_artists() {
    let mut library = create_test_library();

    // Add a second artist
    let track3_metadata = TrackMetadata {
        title: Some(MetadataValue::embedded("Test Track 3".to_string())),
        artist: Some(MetadataValue::embedded("Another Artist".to_string())),
        album: Some(MetadataValue::embedded("Another Album".to_string())),
        album_artist: None,
        year: Some(MetadataValue::embedded(2024)),
        track_number: Some(MetadataValue::embedded(1)),
        disc_number: Some(MetadataValue::embedded(1)),
        genre: None,
        duration: None,
        format: "FLAC".to_string(),
        path: PathBuf::from("/test/artist2/album2/track3.flac"),
    };

    let track3_node = TrackNode {
        file_path: PathBuf::from("/test/artist2/album2/track3.flac"),
        metadata: track3_metadata,
    };

    let album2_files: HashSet<PathBuf> = vec![
        PathBuf::from("/test/artist2/album2/track3.flac"),
    ].into_iter().collect();

    library.artists.push(ArtistNode {
        name: "Another Artist".to_string(),
        albums: vec![AlbumNode {
            title: "Another Album".to_string(),
            year: Some(2024),
            tracks: vec![track3_node],
            files: album2_files,
            path: PathBuf::from("/test/artist2/album2"),
        }],
    });
    library.total_artists = 2;
    library.total_albums = 2;
    library.total_tracks = 3;
    library.total_files = 3; // Updated to reflect the total files

    let output = format_library_output(&library);

    assert!(output.contains("Test Artist"));
    assert!(output.contains("Another Artist"));
    assert!(output.contains("Test Album (2023)"));
    assert!(output.contains("Another Album (2024)"));
}
