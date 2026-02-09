//! Tests for the library module functionality.

use music_chore::core::domain::models::{
    AlbumNode, ArtistNode, Library, MetadataSource, MetadataValue, Track, TrackMetadata,
};
use music_chore::core::services::library::build_library_hierarchy;
use std::collections::HashSet;
use std::path::PathBuf;

fn create_test_track(
    artist: Option<&str>,
    album: Option<&str>,
    title: Option<&str>,
    path: &str,
) -> Track {
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
fn test_build_library_hierarchy_empty() {
    let tracks = vec![];
    let library = build_library_hierarchy(tracks);

    assert_eq!(library.artists.len(), 0);
    assert_eq!(library.total_tracks, 0);
    assert_eq!(library.total_artists, 0);
    assert_eq!(library.total_albums, 0);
    assert_eq!(library.total_files, 0);
}

#[test]
fn test_build_library_hierarchy_single_artist_single_album() {
    let tracks = vec![
        create_test_track(
            Some("Test Artist"),
            Some("Test Album"),
            Some("Track 1"),
            "test/artist/album/track1.flac",
        ),
        create_test_track(
            Some("Test Artist"),
            Some("Test Album"),
            Some("Track 2"),
            "test/artist/album/track2.flac",
        ),
    ];

    let library = build_library_hierarchy(tracks);

    assert_eq!(library.artists.len(), 1);
    assert_eq!(library.total_artists, 1);
    assert_eq!(library.total_albums, 1);
    assert_eq!(library.total_tracks, 2);
    assert_eq!(library.total_files, 2);

    let artist = &library.artists[0];
    assert_eq!(artist.name, "Test Artist");
    assert_eq!(artist.albums.len(), 1);

    let album = &artist.albums[0];
    assert_eq!(album.title, "Test Album");
    assert_eq!(album.tracks.len(), 2);
    assert_eq!(album.files.len(), 2);

    // Verify that file paths are properly collected in album.files
    let expected_files: HashSet<PathBuf> = [
        PathBuf::from("test/artist/album/track1.flac"),
        PathBuf::from("test/artist/album/track2.flac"),
    ]
    .iter()
    .cloned()
    .collect();

    assert_eq!(album.files, expected_files);
}

#[test]
fn test_build_library_hierarchy_multiple_artists() {
    let tracks = vec![
        create_test_track(
            Some("Artist 1"),
            Some("Album 1"),
            Some("Track 1"),
            "artist1/album1/track1.flac",
        ),
        create_test_track(
            Some("Artist 1"),
            Some("Album 1"),
            Some("Track 2"),
            "artist1/album1/track2.flac",
        ),
        create_test_track(
            Some("Artist 2"),
            Some("Album 2"),
            Some("Track 3"),
            "artist2/album2/track3.flac",
        ),
    ];

    let library = build_library_hierarchy(tracks);

    assert_eq!(library.artists.len(), 2);
    assert_eq!(library.total_artists, 2);
    assert_eq!(library.total_albums, 2); // 1 album per artist
    assert_eq!(library.total_tracks, 3);
    assert_eq!(library.total_files, 3);

    // Verify artist 1
    let artist1 = library
        .artists
        .iter()
        .find(|a| a.name == "Artist 1")
        .unwrap();
    assert_eq!(artist1.albums.len(), 1);
    assert_eq!(artist1.albums[0].tracks.len(), 2);

    // Verify artist 2
    let artist2 = library
        .artists
        .iter()
        .find(|a| a.name == "Artist 2")
        .unwrap();
    assert_eq!(artist2.albums.len(), 1);
    assert_eq!(artist2.albums[0].tracks.len(), 1);
}

#[test]
fn test_build_library_hierarchy_multiple_albums_same_artist() {
    let tracks = vec![
        create_test_track(
            Some("Test Artist"),
            Some("Album 1"),
            Some("Track 1"),
            "test/artist/album1/track1.flac",
        ),
        create_test_track(
            Some("Test Artist"),
            Some("Album 1"),
            Some("Track 2"),
            "test/artist/album1/track2.flac",
        ),
        create_test_track(
            Some("Test Artist"),
            Some("Album 2"),
            Some("Track 3"),
            "test/artist/album2/track3.flac",
        ),
    ];

    let library = build_library_hierarchy(tracks);

    assert_eq!(library.artists.len(), 1);
    assert_eq!(library.total_artists, 1);
    assert_eq!(library.total_albums, 2); // Same artist, 2 albums
    assert_eq!(library.total_tracks, 3);
    assert_eq!(library.total_files, 3);

    let artist = &library.artists[0];
    assert_eq!(artist.name, "Test Artist");
    assert_eq!(artist.albums.len(), 2);

    // Verify each album has correct tracks
    let album1 = artist.albums.iter().find(|a| a.title == "Album 1").unwrap();
    assert_eq!(album1.tracks.len(), 2);

    let album2 = artist.albums.iter().find(|a| a.title == "Album 2").unwrap();
    assert_eq!(album2.tracks.len(), 1);
}

#[test]
fn test_build_library_hierarchy_unknown_artist_fallback() {
    let tracks = vec![
        create_test_track(None, Some("Album 1"), Some("Track 1"), "album1/track1.flac"),
        create_test_track(None, Some("Album 1"), Some("Track 2"), "album1/track2.flac"),
    ];

    let library = build_library_hierarchy(tracks);

    assert_eq!(library.artists.len(), 1);
    assert_eq!(library.total_artists, 1);
    assert_eq!(library.total_albums, 1);
    assert_eq!(library.total_tracks, 2);
    assert_eq!(library.total_files, 2);

    let artist = &library.artists[0];
    assert_eq!(artist.name, "Unknown Artist");
    assert_eq!(artist.albums.len(), 1);

    let album = &artist.albums[0];
    assert_eq!(album.title, "Album 1");
    assert_eq!(album.tracks.len(), 2);
}

#[test]
fn test_build_library_hierarchy_unknown_album_fallback() {
    let tracks = vec![
        create_test_track(
            Some("Test Artist"),
            None,
            Some("Track 1"),
            "test/artist/track1.flac",
        ),
        create_test_track(
            Some("Test Artist"),
            None,
            Some("Track 2"),
            "test/artist/track2.flac",
        ),
    ];

    let library = build_library_hierarchy(tracks);

    assert_eq!(library.artists.len(), 1);
    assert_eq!(library.total_artists, 1);
    assert_eq!(library.total_albums, 1);
    assert_eq!(library.total_tracks, 2);
    assert_eq!(library.total_files, 2);

    let artist = &library.artists[0];
    assert_eq!(artist.name, "Test Artist");
    assert_eq!(artist.albums.len(), 1);

    let album = &artist.albums[0];
    assert_eq!(album.title, "Unknown Album");
    assert_eq!(album.tracks.len(), 2);
}

#[test]
fn test_build_library_hierarchy_preserves_metadata() {
    let tracks = vec![create_test_track(
        Some("Test Artist"),
        Some("Test Album"),
        Some("Test Title"),
        "test/path/track.flac",
    )];

    let library = build_library_hierarchy(tracks);

    assert_eq!(library.artists.len(), 1);

    let artist = &library.artists[0];
    assert_eq!(artist.name, "Test Artist");
    assert_eq!(artist.albums.len(), 1);

    let album = &artist.albums[0];
    assert_eq!(album.title, "Test Album");
    assert_eq!(album.tracks.len(), 1);

    let track_node = &album.tracks[0];
    assert_eq!(
        track_node.metadata.title.as_ref().unwrap().value,
        "Test Title"
    );
    assert_eq!(
        track_node.metadata.artist.as_ref().unwrap().value,
        "Test Artist"
    );
    assert_eq!(
        track_node.metadata.album.as_ref().unwrap().value,
        "Test Album"
    );
    assert_eq!(track_node.metadata.format, "flac");
    assert_eq!(track_node.file_path, PathBuf::from("test/path/track.flac"));
}

#[test]
fn test_build_library_hierarchy_counts_correctly() {
    let tracks = vec![
        create_test_track(
            Some("Artist A"),
            Some("Album A1"),
            Some("Track A1-1"),
            "artist_a/album_a1/track1.flac",
        ),
        create_test_track(
            Some("Artist A"),
            Some("Album A1"),
            Some("Track A1-2"),
            "artist_a/album_a1/track2.flac",
        ),
        create_test_track(
            Some("Artist A"),
            Some("Album A2"),
            Some("Track A2-1"),
            "artist_a/album_a2/track1.flac",
        ),
        create_test_track(
            Some("Artist B"),
            Some("Album B1"),
            Some("Track B1-1"),
            "artist_b/album_b1/track1.flac",
        ),
        create_test_track(
            Some("Artist B"),
            Some("Album B1"),
            Some("Track B1-2"),
            "artist_b/album_b1/track2.flac",
        ),
        create_test_track(
            Some("Artist B"),
            Some("Album B1"),
            Some("Track B1-3"),
            "artist_b/album_b1/track3.flac",
        ),
    ];

    let library = build_library_hierarchy(tracks);

    assert_eq!(library.total_artists, 2); // Artist A and Artist B
    assert_eq!(library.total_albums, 3); // Album A1, Album A2, Album B1
    assert_eq!(library.total_tracks, 6); // 6 total tracks
    assert_eq!(library.total_files, 6); // 6 total files

    // Verify per-artist counts
    let artist_a = library
        .artists
        .iter()
        .find(|a| a.name == "Artist A")
        .unwrap();
    assert_eq!(artist_a.albums.len(), 2); // A1 and A2

    let artist_b = library
        .artists
        .iter()
        .find(|a| a.name == "Artist B")
        .unwrap();
    assert_eq!(artist_b.albums.len(), 1); // B1
}

#[test]
fn test_build_library_hierarchy_with_different_metadata_sources() {
    let mut track1 = create_test_track(
        Some("Test Artist"),
        Some("Test Album"),
        Some("Test Title"),
        "path/track1.flac",
    );
    track1.metadata.artist = Some(MetadataValue::inferred("Inferred Artist".to_string(), 0.3));

    let tracks = vec![track1];

    let library = build_library_hierarchy(tracks);

    assert_eq!(library.artists.len(), 1);

    let artist = &library.artists[0];
    assert_eq!(artist.name, "Inferred Artist");
    assert_eq!(artist.albums.len(), 1);

    let album = &artist.albums[0];
    assert_eq!(album.title, "Test Album");
    assert_eq!(album.tracks.len(), 1);

    let track_node = &album.tracks[0];
    assert_eq!(
        track_node.metadata.artist.as_ref().unwrap().value,
        "Inferred Artist"
    );
    assert_eq!(
        track_node.metadata.artist.as_ref().unwrap().source,
        MetadataSource::FolderInferred
    );
    assert_eq!(track_node.metadata.artist.as_ref().unwrap().confidence, 0.3);
}

#[test]
fn test_library_default_creation() {
    let library = Library::new();

    assert_eq!(library.artists.len(), 0);
    assert_eq!(library.total_tracks, 0);
    assert_eq!(library.total_artists, 0);
    assert_eq!(library.total_albums, 0);
    assert_eq!(library.total_files, 0);
}

#[test]
fn test_library_add_artist() {
    let mut library = Library::new();

    let artist = ArtistNode {
        name: "Test Artist".to_string(),
        albums: vec![AlbumNode {
            title: "Test Album".to_string(),
            year: Some(2023),
            tracks: vec![],
            files: HashSet::new(),
            path: PathBuf::from("test/artist/test_album"),
        }],
    };

    library.add_artist(artist);

    assert_eq!(library.total_artists, 1);
    assert_eq!(library.total_albums, 1);
    assert_eq!(library.total_tracks, 0); // No tracks in the album
    assert_eq!(library.total_files, 0); // No files in the album
}

#[test]
fn test_build_hierarchy_preserves_year() {
    // Test ID: LIB007
    // Given: Tracks with year metadata
    let tracks = vec![
        create_test_track(
            Some("Artist A"),
            Some("Album 2020"),
            Some("Track 1"),
            "/music/artist_a/album_2020/track1.flac",
        ),
        create_test_track(
            Some("Artist A"),
            Some("Album 2020"),
            Some("Track 2"),
            "/music/artist_a/album_2020/track2.flac",
        ),
    ];

    // Manually set the year since create_test_track doesn't support it
    let mut tracks_with_year = tracks;
    for track in &mut tracks_with_year {
        track.metadata.year = Some(MetadataValue::embedded(2020));
    }

    // When: Calling build_library_hierarchy(tracks)
    let library = build_library_hierarchy(tracks_with_year);

    // Then: Year should be preserved in AlbumNode
    assert_eq!(library.artists[0].albums[0].year, Some(2020));
}

#[test]
fn test_library_serialization_roundtrip() {
    // Test ID: LB008
    // Given: Library with multiple artists and albums
    let tracks = vec![
        create_test_track(
            Some("Artist A"),
            Some("Album X"),
            Some("Track 1"),
            "/music/artist_a/album_x/track1.flac",
        ),
        create_test_track(
            Some("Artist B"),
            Some("Album Y"),
            Some("Track 2"),
            "/music/artist_b/album_y/track1.flac",
        ),
    ];

    let original = build_library_hierarchy(tracks);

    // When: Serialize to JSON and deserialize back
    let json = serde_json::to_string(&original).expect("Failed to serialize");
    let restored: Library = serde_json::from_str(&json).expect("Failed to deserialize");

    // Then: Original and restored should match
    assert_eq!(original.total_artists, restored.total_artists);
    assert_eq!(original.total_albums, restored.total_albums);
    assert_eq!(original.total_tracks, restored.total_tracks);
    assert_eq!(original.total_files, restored.total_files);
    assert_eq!(original.artists.len(), restored.artists.len());

    // Verify artist names preserved
    let original_names: HashSet<_> = original.artists.iter().map(|a| &a.name).collect();
    let restored_names: HashSet<_> = restored.artists.iter().map(|a| &a.name).collect();
    assert_eq!(original_names, restored_names);
}

#[test]
fn test_build_hierarchy_mixed_metadata() {
    // Test: Mixed scenarios - some with metadata, some without
    let tracks = vec![
        // Track with full metadata
        create_test_track(
            Some("Artist A"),
            Some("Album A"),
            Some("Full Metadata Track"),
            "/music/artist_a/album_a/track1.flac",
        ),
        // Track with no metadata
        create_test_track(
            None,
            None,
            Some("No Metadata Track"),
            "/music/unknown/track2.flac",
        ),
        // Track with partial metadata (artist only)
        create_test_track(
            Some("Artist B"),
            None,
            Some("Partial Track"),
            "/music/artist_b/track3.flac",
        ),
    ];

    // When: Calling build_library_hierarchy(tracks)
    let library = build_library_hierarchy(tracks);

    // Then: Should handle all cases correctly
    assert_eq!(library.total_artists, 3);

    // Find each artist and verify
    let artist_a = library
        .artists
        .iter()
        .find(|a| a.name == "Artist A")
        .unwrap();
    assert_eq!(artist_a.albums[0].title, "Album A");

    let unknown_artist = library
        .artists
        .iter()
        .find(|a| a.name == "Unknown Artist")
        .unwrap();
    assert_eq!(unknown_artist.albums[0].title, "Unknown Album");

    let artist_b = library
        .artists
        .iter()
        .find(|a| a.name == "Artist B")
        .unwrap();
    assert_eq!(artist_b.albums[0].title, "Unknown Album");
}
