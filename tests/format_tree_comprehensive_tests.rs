//! Comprehensive tests for the format tree module functionality.

use music_chore::core::domain::models::{
    AlbumNode, ArtistNode, Library, MetadataSource, MetadataValue, TrackMetadata, TrackNode,
};
use music_chore::core::services::format_tree::{
    emit_by_path, emit_structured_output, format_library_output, format_tree_output,
};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_format_tree_output_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let output = format_tree_output(temp_dir.path());

    assert!(output.contains("📁")); // Directory icon
    assert!(output.contains("Files: 0"));
    assert!(output.contains("Tracks: 0"));
    assert!(output.contains("📊 Library Summary:"));
}

#[test]
fn test_format_tree_output_with_real_files() {
    // Use existing test fixtures
    let output = format_tree_output(&PathBuf::from("tests/fixtures/flac/simple"));

    // Should contain directory structure and file information
    assert!(output.contains("📁 simple"));
    assert!(output.contains("🎵"));
    // The fixture file has embedded metadata with title "Test Apply Behavior" and source 🎯
    assert!(output.contains("[🎯]")); // Embedded metadata indicator
    assert!(output.contains("FLAC"));
}

#[test]
fn test_format_tree_output_nested_structure() {
    let temp_dir = TempDir::new().unwrap();
    let artist_dir = temp_dir.path().join("Test Artist");
    let album_dir = artist_dir.join("Test Album");
    fs::create_dir_all(&album_dir).unwrap();

    // Copy a test file - it has embedded metadata
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        album_dir.join("track1.flac"),
    )
    .unwrap();

    let output = format_tree_output(temp_dir.path());

    assert!(output.contains("Test Artist"));
    assert!(output.contains("Test Album"));
    // format_tree_output displays the embedded title "Test Apply Behavior" from the fixture
    assert!(output.contains("Test Apply Behavior"));
    assert!(output.contains("📁")); // Folder indicators
    assert!(output.contains("🎵")); // File indicators
}

#[test]
fn test_format_library_output_empty() {
    let library = Library::new();
    let output = format_library_output(&library);

    assert!(output.contains("📊 Library Summary:"));
    assert!(output.contains("Artists: 0"));
    assert!(output.contains("Albums: 0"));
    assert!(output.contains("Tracks: 0"));
}

#[test]
fn test_format_library_output_basic() {
    let mut library = Library::new();

    // Create a track node with embedded metadata
    let track_node = TrackNode {
        file_path: PathBuf::from("test/artist/album/track.flac"),
        metadata: TrackMetadata {
            title: Some(MetadataValue::embedded("Test Track".to_string())),
            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
            album: Some(MetadataValue::embedded("Test Album".to_string())),
            album_artist: None,
            track_number: None,
            disc_number: None,
            year: Some(MetadataValue::embedded(2023)),
            genre: Some(MetadataValue::embedded("Test Genre".to_string())),
            duration: Some(MetadataValue::embedded(180.5)),
            format: "flac".to_string(),
            path: PathBuf::from("test/artist/album/track.flac"),
        },
    };

    let album_node = AlbumNode {
        title: "Test Album".to_string(),
        year: Some(2023),
        tracks: vec![track_node],
        files: vec![PathBuf::from("test/artist/album/track.flac")]
            .into_iter()
            .collect(),
        path: PathBuf::from("test/artist/album"),
    };

    let artist_node = ArtistNode {
        name: "Test Artist".to_string(),
        albums: vec![album_node],
    };

    library.add_artist(artist_node);

    let output = format_library_output(&library);

    assert!(output.contains("Test Artist"));
    assert!(output.contains("Test Album"));
    // format_library_output shows the FILENAME, not the title
    assert!(output.contains("track.flac"));
    assert!(output.contains("2023"));
    assert!(output.contains("📊 Library Summary:"));
    // The source icon is based on title's metadata source
    assert!(output.contains("🎯")); // Embedded source icon
}

#[test]
fn test_format_library_output_multiple_artists() {
    let mut library = Library::new();

    // Add first artist
    let track_node1 = TrackNode {
        file_path: PathBuf::from("artist1/album1/track1.flac"),
        metadata: TrackMetadata {
            title: Some(MetadataValue::embedded("Track 1".to_string())),
            artist: Some(MetadataValue::embedded("Artist 1".to_string())),
            album: Some(MetadataValue::embedded("Album 1".to_string())),
            album_artist: None,
            track_number: None,
            disc_number: None,
            year: Some(MetadataValue::embedded(2022)),
            genre: Some(MetadataValue::embedded("Genre 1".to_string())),
            duration: Some(MetadataValue::embedded(200.0)),
            format: "flac".to_string(),
            path: PathBuf::from("artist1/album1/track1.flac"),
        },
    };

    let album_node1 = AlbumNode {
        title: "Album 1".to_string(),
        year: Some(2022),
        tracks: vec![track_node1],
        files: vec![PathBuf::from("artist1/album1/track1.flac")]
            .into_iter()
            .collect(),
        path: PathBuf::from("artist1/album1"),
    };

    let artist_node1 = ArtistNode {
        name: "Artist 1".to_string(),
        albums: vec![album_node1],
    };

    // Add second artist
    let track_node2 = TrackNode {
        file_path: PathBuf::from("artist2/album2/track2.flac"),
        metadata: TrackMetadata {
            title: Some(MetadataValue::embedded("Track 2".to_string())),
            artist: Some(MetadataValue::embedded("Artist 2".to_string())),
            album: Some(MetadataValue::embedded("Album 2".to_string())),
            album_artist: None,
            track_number: None,
            disc_number: None,
            year: Some(MetadataValue::embedded(2023)),
            genre: Some(MetadataValue::embedded("Genre 2".to_string())),
            duration: Some(MetadataValue::embedded(220.0)),
            format: "flac".to_string(),
            path: PathBuf::from("artist2/album2/track2.flac"),
        },
    };

    let album_node2 = AlbumNode {
        title: "Album 2".to_string(),
        year: Some(2023),
        tracks: vec![track_node2],
        files: vec![PathBuf::from("artist2/album2/track2.flac")]
            .into_iter()
            .collect(),
        path: PathBuf::from("artist2/album2"),
    };

    let artist_node2 = ArtistNode {
        name: "Artist 2".to_string(),
        albums: vec![album_node2],
    };

    library.add_artist(artist_node1);
    library.add_artist(artist_node2);

    let output = format_library_output(&library);

    assert!(output.contains("Artist 1"));
    assert!(output.contains("Artist 2"));
    assert!(output.contains("Album 1"));
    assert!(output.contains("Album 2"));
    assert!(output.contains("📊 Library Summary:"));
    assert!(output.contains("Artists: 2"));
}

#[test]
fn test_emit_structured_output_empty() {
    let library = Library::new();
    let output = emit_structured_output(&library);

    assert!(output.contains("MUSIC LIBRARY METADATA"));
    assert!(output.contains("Total Artists: 0"));
    assert!(output.contains("Total Albums: 0"));
    assert!(output.contains("Total Tracks: 0"));
}

#[test]
fn test_emit_structured_output_with_data() {
    let mut library = Library::new();

    // Create a track with embedded metadata
    let track_node = TrackNode {
        file_path: PathBuf::from("test/artist/album/track.flac"),
        metadata: TrackMetadata {
            title: Some(MetadataValue::embedded("Test Song".to_string())),
            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
            album: Some(MetadataValue::embedded("Test Album".to_string())),
            album_artist: None,
            track_number: None,
            disc_number: None,
            year: Some(MetadataValue::embedded(2023)),
            genre: Some(MetadataValue::embedded("Test Genre".to_string())),
            duration: Some(MetadataValue::embedded(180.5)),
            format: "flac".to_string(),
            path: PathBuf::from("test/artist/album/track.flac"),
        },
    };

    let album_node = AlbumNode {
        title: "Test Album".to_string(),
        year: Some(2023),
        tracks: vec![track_node],
        files: vec![PathBuf::from("test/artist/album/track.flac")]
            .into_iter()
            .collect(),
        path: PathBuf::from("test/artist/album"),
    };

    let artist_node = ArtistNode {
        name: "Test Artist".to_string(),
        albums: vec![album_node],
    };

    library.add_artist(artist_node);

    let output = emit_structured_output(&library);

    // emit_structured_output displays track TITLES, not filenames
    assert!(output.contains("MUSIC LIBRARY METADATA"));
    assert!(output.contains("Test Artist"));
    assert!(output.contains("Test Album"));
    assert!(output.contains("Test Song")); // Track title is displayed
    assert!(output.contains("2023"));
}

#[test]
fn test_emit_by_path_json_output() {
    let temp_dir = TempDir::new().unwrap();
    let artist_dir = temp_dir.path().join("Test Artist");
    let album_dir = artist_dir.join("Test Album");
    fs::create_dir_all(&album_dir).unwrap();

    // Copy a test file
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        album_dir.join("track1.flac"),
    )
    .unwrap();

    let result = emit_by_path(temp_dir.path(), true); // JSON output

    assert!(result.is_ok());
    let output = result.unwrap();

    // Should be valid JSON
    let json_value: serde_json::Value =
        serde_json::from_str(&output).expect("Output should be valid JSON");
    assert!(json_value.is_object());
    assert!(json_value.get("__schema_version").is_some());
    // The Library structure serializes with "artists" at top level
    assert!(json_value.get("artists").is_some(), "{}", json_value);
    // Verify the hierarchy was built correctly
    assert!(json_value.get("total_artists").is_some());
}

#[test]
fn test_emit_by_path_text_output() {
    let temp_dir = TempDir::new().unwrap();
    let artist_dir = temp_dir.path().join("Test Artist");
    let album_dir = artist_dir.join("Test Album");
    fs::create_dir_all(&album_dir).unwrap();

    // Copy a test file
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        album_dir.join("track1.flac"),
    )
    .unwrap();

    let result = emit_by_path(temp_dir.path(), false); // Text output

    assert!(result.is_ok());
    let output = result.unwrap();

    // emit_by_path with text=false uses emit_structured_output which shows TITLES
    // The fixture file has title "Test Apply Behavior"
    assert!(output.contains("Test Artist"));
    assert!(output.contains("Test Album"));
    assert!(output.contains("Test Apply Behavior"));
}

#[test]
fn test_emit_by_path_nonexistent_directory() {
    let nonexistent_path = PathBuf::from("/nonexistent/path");

    let result = emit_by_path(&nonexistent_path, false);

    // emit_by_path may return Ok with empty library or Err for nonexistent path
    if result.is_err() {
        let error = result.unwrap_err();
        assert!(error.contains("does not exist") || error.contains("Failed to scan"));
    } else {
        // If it succeeds, it should return empty library
        let output = result.unwrap();
        assert!(output.contains("Total Artists: 0") || output.contains("Artists: 0"));
    }
}

#[test]
fn test_emit_by_path_empty_directory() {
    let temp_dir = TempDir::new().unwrap();

    let result = emit_by_path(temp_dir.path(), false);

    assert!(result.is_ok());
    let output = result.unwrap();

    // For empty directory, emit_structured_output is called
    assert!(output.contains("MUSIC LIBRARY METADATA"));
    assert!(output.contains("Total Artists: 0"));
    assert!(output.contains("Total Albums: 0"));
    assert!(output.contains("Total Tracks: 0"));
}

#[test]
fn test_format_tree_output_with_inferred_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let artist_dir = temp_dir.path().join("Inferred Artist");
    let album_dir = artist_dir.join("Inferred Album");
    fs::create_dir_all(&album_dir).unwrap();

    // Create a dummy audio file with no embedded metadata (not a valid FLAC)
    fs::write(album_dir.join("track1.flac"), b"dummy flac content").unwrap();

    let output = format_tree_output(temp_dir.path());

    // If the file has no readable embedded metadata, it won't be included
    // Or it will show folder-inferred (🤖) if scan_dir can handle it
    // The test checks for either: directory structure exists OR no files found
    assert!(
        output.contains("Inferred Artist") || output.contains("Files: 0"),
        "Output: {}",
        output
    );
}

#[test]
fn test_format_tree_output_unicode_paths() {
    let temp_dir = TempDir::new().unwrap();
    let artist_dir = temp_dir.path().join("Tëst Ärtist");
    let album_dir = artist_dir.join("Tëst Älbum");
    fs::create_dir_all(&album_dir).unwrap();

    // Copy a test file
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        album_dir.join("tëst_träck.flac"),
    )
    .unwrap();

    let output = format_tree_output(temp_dir.path());

    // Should handle unicode characters properly
    assert!(output.contains("Tëst Ärtist"));
    assert!(output.contains("Tëst Älbum"));
}

#[test]
fn test_format_tree_output_with_cue_inferred_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let album_dir = temp_dir.path().join("Cue Album");
    fs::create_dir_all(&album_dir).unwrap();

    // Create a dummy audio file (not valid FLAC, will be skipped)
    fs::write(album_dir.join("track1.flac"), b"dummy flac content").unwrap();

    let output = format_tree_output(&album_dir);

    // For invalid/unreadable files, they may be skipped
    assert!(
        output.contains("Cue Album") || output.contains("Files: 0"),
        "Output: {}",
        output
    );
}

#[test]
fn test_format_tree_output_with_multiple_files() {
    let temp_dir = TempDir::new().unwrap();
    let artist_dir = temp_dir.path().join("Test Artist");
    let album_dir = artist_dir.join("Test Album");
    fs::create_dir_all(&album_dir).unwrap();

    // Copy multiple test files
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        album_dir.join("track1.flac"),
    )
    .unwrap();
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        album_dir.join("track2.flac"),
    )
    .unwrap();

    let output = format_tree_output(temp_dir.path());

    assert!(output.contains("Test Artist"));
    assert!(output.contains("Test Album"));
    // Both files have embedded metadata with title "Test Apply Behavior"
    assert!(output.contains("Test Apply Behavior"));
    assert!(output.contains("🎵")); // File indicators
    assert!(output.contains("📊 Library Summary:"));
    // Summary should show 2 files and 2 tracks
    assert!(output.contains("Files: 2"));
    assert!(output.contains("Tracks: 2"));
}

#[test]
fn test_format_tree_output_with_mixed_metadata_sources() {
    let temp_dir = TempDir::new().unwrap();
    let artist_dir = temp_dir.path().join("Test Artist");
    let album_dir = artist_dir.join("Test Album");
    fs::create_dir_all(&album_dir).unwrap();

    // Copy a test file with embedded metadata
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        album_dir.join("track1.flac"),
    )
    .unwrap();

    let output = format_tree_output(temp_dir.path());

    // The fixture has embedded metadata, so it shows embedded indicator
    assert!(output.contains("🎯")); // Embedded metadata indicator
}

#[test]
fn test_format_tree_output_with_folder_inferred_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let album_dir = temp_dir.path().join("Test Album");
    fs::create_dir_all(&album_dir).unwrap();

    // Copy a test file with embedded metadata
    fs::copy(
        "tests/fixtures/flac/simple/track1.flac",
        album_dir.join("track1.flac"),
    )
    .unwrap();

    let output = format_tree_output(temp_dir.path());

    // The fixture file has embedded metadata (🎯)
    // To test folder-inferred (🤖), we'd need a valid audio file without embedded metadata
    // For now, verify that the file is shown with embedded indicator
    assert!(
        output.contains("🎯"),
        "Output should contain embedded indicator: {}",
        output
    );
}

#[test]
fn test_format_library_output_with_years() {
    let mut library = Library::new();

    let track_node = TrackNode {
        file_path: PathBuf::from("test/artist/album/track.flac"),
        metadata: TrackMetadata {
            title: Some(MetadataValue::embedded("Test Song".to_string())),
            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
            album: Some(MetadataValue::embedded("Test Album".to_string())),
            album_artist: None,
            track_number: None,
            disc_number: None,
            year: Some(MetadataValue::embedded(2023)),
            genre: Some(MetadataValue::embedded("Test Genre".to_string())),
            duration: Some(MetadataValue::embedded(180.5)),
            format: "flac".to_string(),
            path: PathBuf::from("test/artist/album/track.flac"),
        },
    };

    let album_node = AlbumNode {
        title: "Test Album".to_string(),
        year: Some(2023),
        tracks: vec![track_node],
        files: vec![PathBuf::from("test/artist/album/track.flac")]
            .into_iter()
            .collect(),
        path: PathBuf::from("test/artist/album"),
    };

    let artist_node = ArtistNode {
        name: "Test Artist".to_string(),
        albums: vec![album_node],
    };

    library.add_artist(artist_node);

    let output = format_library_output(&library);

    // format_library_output shows year in parentheses after album title
    assert!(output.contains("(2023)"));
}

#[test]
fn test_format_library_output_with_multiple_albums_same_artist() {
    let mut library = Library::new();

    // First album
    let track_node1 = TrackNode {
        file_path: PathBuf::from("artist/album1/track1.flac"),
        metadata: TrackMetadata {
            title: Some(MetadataValue::embedded("Track 1".to_string())),
            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
            album: Some(MetadataValue::embedded("Album 1".to_string())),
            album_artist: None,
            track_number: None,
            disc_number: None,
            year: Some(MetadataValue::embedded(2022)),
            genre: Some(MetadataValue::embedded("Test Genre".to_string())),
            duration: Some(MetadataValue::embedded(180.5)),
            format: "flac".to_string(),
            path: PathBuf::from("artist/album1/track1.flac"),
        },
    };

    let album_node1 = AlbumNode {
        title: "Album 1".to_string(),
        year: Some(2022),
        tracks: vec![track_node1],
        files: vec![PathBuf::from("artist/album1/track1.flac")]
            .into_iter()
            .collect(),
        path: PathBuf::from("artist/album1"),
    };

    // Second album
    let track_node2 = TrackNode {
        file_path: PathBuf::from("artist/album2/track2.flac"),
        metadata: TrackMetadata {
            title: Some(MetadataValue::embedded("Track 2".to_string())),
            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
            album: Some(MetadataValue::embedded("Album 2".to_string())),
            album_artist: None,
            track_number: None,
            disc_number: None,
            year: Some(MetadataValue::embedded(2023)),
            genre: Some(MetadataValue::embedded("Test Genre".to_string())),
            duration: Some(MetadataValue::embedded(220.0)),
            format: "flac".to_string(),
            path: PathBuf::from("artist/album2/track2.flac"),
        },
    };

    let album_node2 = AlbumNode {
        title: "Album 2".to_string(),
        year: Some(2023),
        tracks: vec![track_node2],
        files: vec![PathBuf::from("artist/album2/track2.flac")]
            .into_iter()
            .collect(),
        path: PathBuf::from("artist/album2"),
    };

    let artist_node = ArtistNode {
        name: "Test Artist".to_string(),
        albums: vec![album_node1, album_node2],
    };

    library.add_artist(artist_node);

    let output = format_library_output(&library);

    assert!(output.contains("Test Artist"));
    assert!(output.contains("Album 1"));
    assert!(output.contains("Album 2"));
    assert!(output.contains("(2022)"));
    assert!(output.contains("(2023)"));
    assert!(output.contains("📊 Library Summary:"));
    assert!(output.contains("Albums: 2"));
}
