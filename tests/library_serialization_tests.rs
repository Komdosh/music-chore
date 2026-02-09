//! Tests for Library serialization/deserialization as part of the refactoring plan.

use music_chore::core::domain::models::{AlbumNode, ArtistNode, Library, MetadataValue, Track, TrackMetadata, TrackNode, MetadataSource};
use std::collections::HashSet;
use std::path::PathBuf;
use serde_json;

fn create_test_track(title: Option<&str>, artist: Option<&str>, album: Option<&str>, path: &str) -> Track {
    Track::new(
        PathBuf::from(path),
        TrackMetadata {
            title: title.map(|t| MetadataValue::embedded(t.to_string())),
            artist: artist.map(|a| MetadataValue::embedded(a.to_string())),
            album: album.map(|a| MetadataValue::embedded(a.to_string())),
            album_artist: None,
            track_number: None,
            disc_number: None,
            year: None,
            genre: None,
            duration: None,
            format: "flac".to_string(),
            path: PathBuf::from(path),
        },
    )
}

#[test]
fn test_library_serialization() {
    let mut library = Library::new();
    
    // Create a test track
    let track = create_test_track(Some("Test Track"), Some("Test Artist"), Some("Test Album"), "test/artist/album/track.flac");
    
    // Create track node
    let track_node = TrackNode {
        file_path: track.file_path.clone(),
        metadata: track.metadata.clone(),
    };
    
    // Create album node
    let album_node = AlbumNode {
        title: "Test Album".to_string(),
        year: Some(2023),
        tracks: vec![track_node],
        files: vec![PathBuf::from("test/artist/album/track.flac")].into_iter().collect(),
        path: PathBuf::from("test/artist/album"),
    };
    
    // Create artist node
    let artist_node = ArtistNode {
        name: "Test Artist".to_string(),
        albums: vec![album_node],
    };
    
    library.add_artist(artist_node);
    
    // Test serialization
    let json_result = serde_json::to_string_pretty(&library);
    assert!(json_result.is_ok());
    
    let json_str = json_result.unwrap();
    assert!(json_str.contains("Test Artist"));
    assert!(json_str.contains("Test Album"));
    assert!(json_str.contains("Test Track"));
    assert!(json_str.contains("2023"));
}

#[test]
fn test_library_deserialization() {
    let json_data = r#"{
  "artists": [
    {
      "name": "Test Artist",
      "albums": [
        {
          "title": "Test Album",
          "year": 2023,
          "tracks": [
            {
              "file_path": "test/artist/album/track.flac",
              "metadata": {
                "title": {
                  "value": "Test Track",
                  "source": "Embedded",
                  "confidence": 1.0
                },
                "artist": {
                  "value": "Test Artist",
                  "source": "Embedded",
                  "confidence": 1.0
                },
                "album": {
                  "value": "Test Album",
                  "source": "Embedded",
                  "confidence": 1.0
                },
                "album_artist": null,
                "track_number": null,
                "disc_number": null,
                "year": null,
                "genre": null,
                "duration": null,
                "format": "flac",
                "path": "test/artist/album/track.flac"
              }
            }
          ],
          "files": [
            "test/artist/album/track.flac"
          ],
          "path": "test/artist/album"
        }
      ]
    }
  ],
  "total_tracks": 1,
  "total_artists": 1,
  "total_albums": 1,
  "total_files": 1
}"#;
    
    let result: Result<Library, _> = serde_json::from_str(json_data);
    assert!(result.is_ok());
    
    let library = result.unwrap();
    assert_eq!(library.total_artists, 1);
    assert_eq!(library.total_albums, 1);
    assert_eq!(library.total_tracks, 1);
    assert_eq!(library.total_files, 1);
    
    assert_eq!(library.artists[0].name, "Test Artist");
    assert_eq!(library.artists[0].albums[0].title, "Test Album");
    assert_eq!(library.artists[0].albums[0].year, Some(2023));
    assert_eq!(library.artists[0].albums[0].tracks[0].metadata.title.as_ref().unwrap().value, "Test Track");
}

#[test]
fn test_library_serialization_deserialization_roundtrip() {
    let mut original_library = Library::new();
    
    // Create a test track
    let track = create_test_track(Some("Round Trip Track"), Some("Round Trip Artist"), Some("Round Trip Album"), "round/trip/path/track.flac");
    
    // Create track node
    let track_node = TrackNode {
        file_path: track.file_path.clone(),
        metadata: track.metadata.clone(),
    };
    
    // Create album node
    let album_node = AlbumNode {
        title: "Round Trip Album".to_string(),
        year: Some(2024),
        tracks: vec![track_node],
        files: vec![PathBuf::from("round/trip/path/track.flac")].into_iter().collect(),
        path: PathBuf::from("round/trip/path"),
    };
    
    // Create artist node
    let artist_node = ArtistNode {
        name: "Round Trip Artist".to_string(),
        albums: vec![album_node],
    };
    
    original_library.add_artist(artist_node);
    
    // Serialize the library
    let json_result = serde_json::to_string_pretty(&original_library);
    assert!(json_result.is_ok());
    let json_str = json_result.unwrap();
    
    // Deserialize back to library
    let deserialized_library: Library = serde_json::from_str(&json_str).unwrap();
    
    // Compare the libraries
    assert_eq!(original_library.total_artists, deserialized_library.total_artists);
    assert_eq!(original_library.total_albums, deserialized_library.total_albums);
    assert_eq!(original_library.total_tracks, deserialized_library.total_tracks);
    assert_eq!(original_library.total_files, deserialized_library.total_files);
    
    // Check specific content
    assert_eq!(original_library.artists[0].name, deserialized_library.artists[0].name);
    assert_eq!(original_library.artists[0].albums[0].title, deserialized_library.artists[0].albums[0].title);
    assert_eq!(original_library.artists[0].albums[0].year, deserialized_library.artists[0].albums[0].year);
    assert_eq!(
        original_library.artists[0].albums[0].tracks[0].metadata.title.as_ref().unwrap().value,
        deserialized_library.artists[0].albums[0].tracks[0].metadata.title.as_ref().unwrap().value
    );
}

#[test]
fn test_library_empty_serialization() {
    let library = Library::new();
    
    let json_result = serde_json::to_string_pretty(&library);
    assert!(json_result.is_ok());
    
    let json_str = json_result.unwrap();
    assert!(json_str.contains("artists"));
    assert!(json_str.contains("[]")); // Should be empty arrays
    
    let deserialized: Library = serde_json::from_str(&json_str).unwrap();
    assert_eq!(deserialized.total_artists, 0);
    assert_eq!(deserialized.total_albums, 0);
    assert_eq!(deserialized.total_tracks, 0);
    assert_eq!(deserialized.total_files, 0);
    assert_eq!(deserialized.artists.len(), 0);
}

#[test]
fn test_library_multiple_artists_serialization() {
    let mut library = Library::new();
    
    // Create first artist with album
    let track1 = create_test_track(Some("Track 1"), Some("Artist 1"), Some("Album 1"), "artist1/album1/track1.flac");
    let track_node1 = TrackNode {
        file_path: track1.file_path.clone(),
        metadata: track1.metadata.clone(),
    };
    let album_node1 = AlbumNode {
        title: "Album 1".to_string(),
        year: Some(2023),
        tracks: vec![track_node1],
        files: vec![PathBuf::from("artist1/album1/track1.flac")].into_iter().collect(),
        path: PathBuf::from("artist1/album1"),
    };
    let artist_node1 = ArtistNode {
        name: "Artist 1".to_string(),
        albums: vec![album_node1],
    };
    
    // Create second artist with album
    let track2 = create_test_track(Some("Track 2"), Some("Artist 2"), Some("Album 2"), "artist2/album2/track2.flac");
    let track_node2 = TrackNode {
        file_path: track2.file_path.clone(),
        metadata: track2.metadata.clone(),
    };
    let album_node2 = AlbumNode {
        title: "Album 2".to_string(),
        year: Some(2024),
        tracks: vec![track_node2],
        files: vec![PathBuf::from("artist2/album2/track2.flac")].into_iter().collect(),
        path: PathBuf::from("artist2/album2"),
    };
    let artist_node2 = ArtistNode {
        name: "Artist 2".to_string(),
        albums: vec![album_node2],
    };
    
    library.add_artist(artist_node1);
    library.add_artist(artist_node2);
    
    // Test serialization
    let json_result = serde_json::to_string_pretty(&library);
    assert!(json_result.is_ok());
    
    let json_str = json_result.unwrap();
    assert!(json_str.contains("Artist 1"));
    assert!(json_str.contains("Artist 2"));
    assert!(json_str.contains("Album 1"));
    assert!(json_str.contains("Album 2"));
    assert!(json_str.contains("2023"));
    assert!(json_str.contains("2024"));
    
    // Test deserialization
    let deserialized: Library = serde_json::from_str(&json_str).unwrap();
    assert_eq!(deserialized.total_artists, 2);
    assert_eq!(deserialized.total_albums, 2);
    assert_eq!(deserialized.total_tracks, 2);
    assert_eq!(deserialized.total_files, 2);
    
    // Verify both artists exist
    let artist_names: Vec<&String> = deserialized.artists.iter().map(|a| &a.name).collect();
    assert!(artist_names.contains(&&"Artist 1".to_string()));
    assert!(artist_names.contains(&&"Artist 2".to_string()));
}

#[test]
fn test_library_multiple_albums_same_artist_serialization() {
    let mut library = Library::new();
    
    // Create artist with multiple albums
    let track1 = create_test_track(Some("Track 1"), Some("Same Artist"), Some("Album 1"), "same_artist/album1/track1.flac");
    let track_node1 = TrackNode {
        file_path: track1.file_path.clone(),
        metadata: track1.metadata.clone(),
    };
    let album_node1 = AlbumNode {
        title: "Album 1".to_string(),
        year: Some(2023),
        tracks: vec![track_node1],
        files: vec![PathBuf::from("same_artist/album1/track1.flac")].into_iter().collect(),
        path: PathBuf::from("same_artist/album1"),
    };
    
    let track2 = create_test_track(Some("Track 2"), Some("Same Artist"), Some("Album 2"), "same_artist/album2/track2.flac");
    let track_node2 = TrackNode {
        file_path: track2.file_path.clone(),
        metadata: track2.metadata.clone(),
    };
    let album_node2 = AlbumNode {
        title: "Album 2".to_string(),
        year: Some(2024),
        tracks: vec![track_node2],
        files: vec![PathBuf::from("same_artist/album2/track2.flac")].into_iter().collect(),
        path: PathBuf::from("same_artist/album2"),
    };
    
    let artist_node = ArtistNode {
        name: "Same Artist".to_string(),
        albums: vec![album_node1, album_node2],
    };
    
    library.add_artist(artist_node);
    
    // Test serialization
    let json_result = serde_json::to_string_pretty(&library);
    assert!(json_result.is_ok());
    
    let json_str = json_result.unwrap();
    assert!(json_str.contains("Same Artist"));
    assert!(json_str.contains("Album 1"));
    assert!(json_str.contains("Album 2"));
    
    // Test deserialization
    let deserialized: Library = serde_json::from_str(&json_str).unwrap();
    assert_eq!(deserialized.total_artists, 1);
    assert_eq!(deserialized.total_albums, 2);
    assert_eq!(deserialized.total_tracks, 2);
    assert_eq!(deserialized.total_files, 2);
    
    // Verify the artist has both albums
    assert_eq!(deserialized.artists[0].name, "Same Artist");
    assert_eq!(deserialized.artists[0].albums.len(), 2);
    
    let album_titles: Vec<&String> = deserialized.artists[0].albums.iter().map(|a| &a.title).collect();
    assert!(album_titles.contains(&&"Album 1".to_string()));
    assert!(album_titles.contains(&&"Album 2".to_string()));
}

#[test]
fn test_library_multiple_tracks_same_album_serialization() {
    let mut library = Library::new();
    
    // Create album with multiple tracks
    let track1 = create_test_track(Some("Track 1"), Some("Multi Track Artist"), Some("Multi Track Album"), "multi_artist/multi_album/track1.flac");
    let track_node1 = TrackNode {
        file_path: track1.file_path.clone(),
        metadata: track1.metadata.clone(),
    };
    
    let track2 = create_test_track(Some("Track 2"), Some("Multi Track Artist"), Some("Multi Track Album"), "multi_artist/multi_album/track2.flac");
    let track_node2 = TrackNode {
        file_path: track2.file_path.clone(),
        metadata: track2.metadata.clone(),
    };
    
    let album_node = AlbumNode {
        title: "Multi Track Album".to_string(),
        year: Some(2023),
        tracks: vec![track_node1, track_node2],
        files: vec![
            PathBuf::from("multi_artist/multi_album/track1.flac"),
            PathBuf::from("multi_artist/multi_album/track2.flac")
        ].into_iter().collect(),
        path: PathBuf::from("multi_artist/multi_album"),
    };
    
    let artist_node = ArtistNode {
        name: "Multi Track Artist".to_string(),
        albums: vec![album_node],
    };
    
    library.add_artist(artist_node);
    
    // Test serialization
    let json_result = serde_json::to_string_pretty(&library);
    assert!(json_result.is_ok());
    
    let json_str = json_result.unwrap();
    assert!(json_str.contains("Multi Track Artist"));
    assert!(json_str.contains("Multi Track Album"));
    assert!(json_str.contains("Track 1"));
    assert!(json_str.contains("Track 2"));
    
    // Test deserialization
    let deserialized: Library = serde_json::from_str(&json_str).unwrap();
    assert_eq!(deserialized.total_artists, 1);
    assert_eq!(deserialized.total_albums, 1);
    assert_eq!(deserialized.total_tracks, 2);
    assert_eq!(deserialized.total_files, 2);
    
    // Verify the album has both tracks
    assert_eq!(deserialized.artists[0].albums[0].tracks.len(), 2);
    let track_titles: Vec<&String> = deserialized.artists[0].albums[0].tracks.iter()
        .filter_map(|t| t.metadata.title.as_ref())
        .map(|mv| &mv.value)
        .collect();
    assert!(track_titles.contains(&&"Track 1".to_string()));
    assert!(track_titles.contains(&&"Track 2".to_string()));
}

#[test]
fn test_library_metadata_source_serialization() {
    let mut library = Library::new();
    
    // Create a track with different metadata sources
    let track_metadata = TrackMetadata {
        title: Some(MetadataValue::embedded("Embedded Title".to_string())),
        artist: Some(MetadataValue::inferred("Inferred Artist".to_string(), 0.3)),
        album: Some(MetadataValue::user_set("User Set Album".to_string())),
        album_artist: Some(MetadataValue::cue_inferred("CUE Artist".to_string(), 1.0)),
        track_number: Some(MetadataValue::embedded(1)),
        disc_number: Some(MetadataValue::embedded(1)),
        year: Some(MetadataValue::embedded(2023)),
        genre: Some(MetadataValue::inferred("Inferred Genre".to_string(), 0.3)),
        duration: Some(MetadataValue::embedded(180.5)),
        format: "flac".to_string(),
        path: PathBuf::from("test/path/track.flac"),
    };
    
    let track = Track::new(
        PathBuf::from("test/path/track.flac"),
        track_metadata,
    );
    
    let track_node = TrackNode {
        file_path: track.file_path.clone(),
        metadata: track.metadata.clone(),
    };
    
    let album_node = AlbumNode {
        title: "Test Album".to_string(),
        year: Some(2023),
        tracks: vec![track_node],
        files: vec![PathBuf::from("test/path/track.flac")].into_iter().collect(),
        path: PathBuf::from("test/path"),
    };
    
    let artist_node = ArtistNode {
        name: "Test Artist".to_string(),
        albums: vec![album_node],
    };
    
    library.add_artist(artist_node);
    
    // Test serialization
    let json_result = serde_json::to_string_pretty(&library);
    assert!(json_result.is_ok());
    
    let json_str = json_result.unwrap();
    assert!(json_str.contains("\"source\": \"Embedded\""));
    assert!(json_str.contains("\"source\": \"FolderInferred\""));
    assert!(json_str.contains("\"source\": \"UserEdited\""));
    assert!(json_str.contains("\"source\": \"CueInferred\""));
    
    // Test deserialization preserves sources
    let deserialized: Library = serde_json::from_str(&json_str).unwrap();
    let track_metadata = &deserialized.artists[0].albums[0].tracks[0].metadata;
    
    assert_eq!(track_metadata.title.as_ref().unwrap().source, MetadataSource::Embedded);
    assert_eq!(track_metadata.artist.as_ref().unwrap().source, MetadataSource::FolderInferred);
    assert_eq!(track_metadata.album.as_ref().unwrap().source, MetadataSource::UserEdited);
    assert_eq!(track_metadata.album_artist.as_ref().unwrap().source, MetadataSource::CueInferred);
}

#[test]
fn test_library_metadata_confidence_serialization() {
    let mut library = Library::new();
    
    // Create a track with different confidence levels
    let track_metadata = TrackMetadata {
        title: Some(MetadataValue::embedded("High Confidence Title".to_string())),
        artist: Some(MetadataValue::inferred("Low Confidence Artist".to_string(), 0.3)),
        album: Some(MetadataValue::inferred("Medium Confidence Album".to_string(), 0.7)),
        album_artist: None,
        track_number: Some(MetadataValue::embedded(1)),
        disc_number: Some(MetadataValue::embedded(1)),
        year: Some(MetadataValue::embedded(2023)),
        genre: Some(MetadataValue::inferred("Inferred Genre".to_string(), 0.5)),
        duration: Some(MetadataValue::embedded(180.5)),
        format: "flac".to_string(),
        path: PathBuf::from("test/confidence/track.flac"),
    };
    
    let track = Track::new(
        PathBuf::from("test/confidence/track.flac"),
        track_metadata,
    );
    
    let track_node = TrackNode {
        file_path: track.file_path.clone(),
        metadata: track.metadata.clone(),
    };
    
    let album_node = AlbumNode {
        title: "Confidence Test Album".to_string(),
        year: Some(2023),
        tracks: vec![track_node],
        files: vec![PathBuf::from("test/confidence/track.flac")].into_iter().collect(),
        path: PathBuf::from("test/confidence"),
    };
    
    let artist_node = ArtistNode {
        name: "Confidence Test Artist".to_string(),
        albums: vec![album_node],
    };
    
    library.add_artist(artist_node);
    
    // Test serialization includes confidence values
    let json_result = serde_json::to_string_pretty(&library);
    assert!(json_result.is_ok());
    
    let json_str = json_result.unwrap();
    assert!(json_str.contains("\"confidence\": 1.0")); // For embedded
    assert!(json_str.contains("\"confidence\": 0.3")); // For inferred
    assert!(json_str.contains("\"confidence\": 0.7")); // For inferred
    assert!(json_str.contains("\"confidence\": 0.5")); // For inferred
    
    // Test deserialization preserves confidence values
    let deserialized: Library = serde_json::from_str(&json_str).unwrap();
    let track_metadata = &deserialized.artists[0].albums[0].tracks[0].metadata;
    
    assert_eq!(track_metadata.title.as_ref().unwrap().confidence, 1.0);
    assert_eq!(track_metadata.artist.as_ref().unwrap().confidence, 0.3);
    assert_eq!(track_metadata.album.as_ref().unwrap().confidence, 0.7);
    assert_eq!(track_metadata.genre.as_ref().unwrap().confidence, 0.5);
}

#[test]
fn test_library_with_checksum_serialization() {
    let mut library = Library::new();
    
    // Create a track with checksum - note that checksums are not included in the library hierarchy
    // (TrackNode doesn't have checksum field), so we'll test that the library still serializes correctly
    let track = Track::with_checksum(
        PathBuf::from("test/checksum/track.flac"),
        TrackMetadata {
            title: Some(MetadataValue::embedded("Checksum Test Track".to_string())),
            artist: Some(MetadataValue::embedded("Checksum Test Artist".to_string())),
            album: Some(MetadataValue::embedded("Checksum Test Album".to_string())),
            album_artist: None,
            track_number: Some(MetadataValue::embedded(1)),
            disc_number: Some(MetadataValue::embedded(1)),
            year: Some(MetadataValue::embedded(2023)),
            genre: Some(MetadataValue::embedded("Test Genre".to_string())),
            duration: Some(MetadataValue::embedded(180.5)),
            format: "flac".to_string(),
            path: PathBuf::from("test/checksum/track.flac"),
        },
        "abcd1234efgh5678".to_string(),
    );
    
    let track_node = TrackNode {
        file_path: track.file_path.clone(),
        metadata: track.metadata.clone(),
    };
    
    let album_node = AlbumNode {
        title: "Checksum Test Album".to_string(),
        year: Some(2023),
        tracks: vec![track_node],
        files: vec![PathBuf::from("test/checksum/track.flac")].into_iter().collect(),
        path: PathBuf::from("test/checksum"),
    };
    
    let artist_node = ArtistNode {
        name: "Checksum Test Artist".to_string(),
        albums: vec![album_node],
    };
    
    library.add_artist(artist_node);
    
    // Test serialization
    let json_result = serde_json::to_string_pretty(&library);
    assert!(json_result.is_ok());
    
    let json_str = json_result.unwrap();
    // The checksum is not included in the library hierarchy serialization (by design)
    // but the rest of the data should be properly serialized
    assert!(json_str.contains("Checksum Test Track"));
    assert!(json_str.contains("Checksum Test Artist"));
    assert!(json_str.contains("Checksum Test Album"));
    
    // Test deserialization
    let deserialized: Library = serde_json::from_str(&json_str).unwrap();
    assert_eq!(deserialized.artists[0].albums[0].tracks[0].file_path, PathBuf::from("test/checksum/track.flac"));
}