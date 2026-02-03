use music_chore::{
    build_library_hierarchy, AlbumNode, ArtistNode, Library, MetadataSource, MetadataValue,
    TrackNode,
};
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

// Test the tree command functionality via CLI integration
#[cfg(test)]
mod tests {
    use super::*;
    use music_chore::services::scanner::scan_dir;

    fn create_test_library() -> Library {
        let mut library = Library::new();

        // Create first artist with multiple albums
        let artist1 = ArtistNode {
            name: "Test Artist".to_string(),
            albums: vec![AlbumNode {
                title: "First Album".to_string(),
                year: Some(2023),
                tracks: vec![TrackNode {
                    file_path: PathBuf::from("Test Artist/First Album/01 Track.flac"),
                    metadata: music_chore::TrackMetadata {
                        title: Some(MetadataValue {
                            value: "First Track".to_string(),
                            source: MetadataSource::Embedded,
                            confidence: 1.0,
                        }),
                        artist: Some(MetadataValue {
                            value: "Test Artist".to_string(),
                            source: MetadataSource::Embedded,
                            confidence: 1.0,
                        }),
                        album: Some(MetadataValue {
                            value: "First Album".to_string(),
                            source: MetadataSource::Embedded,
                            confidence: 1.0,
                        }),
                        album_artist: None,
                        track_number: Some(MetadataValue {
                            value: 1,
                            source: MetadataSource::Embedded,
                            confidence: 1.0,
                        }),
                        disc_number: None,
                        year: Some(MetadataValue {
                            value: 2023,
                            source: MetadataSource::Embedded,
                            confidence: 1.0,
                        }),
                        genre: None,
                        duration: Some(MetadataValue {
                            value: 180.5,
                            source: MetadataSource::Embedded,
                            confidence: 1.0,
                        }),
                        format: "flac".to_string(),
                        path: PathBuf::from("Test Artist/First Album/01 Track.flac"),
                    },
                }],
                path: PathBuf::from("Test Artist/First Album"),
            }],
        };

        library.add_artist(artist1);
        library
    }

    #[test]
    fn test_library_structure_creation() {
        let library = create_test_library();

        assert_eq!(library.total_artists, 1);
        assert_eq!(library.total_albums, 1);
        assert_eq!(library.total_tracks, 1);

        let artist = &library.artists[0];
        assert_eq!(artist.name, "Test Artist");

        let album = &artist.albums[0];
        assert_eq!(album.title, "First Album");
        assert_eq!(album.year, Some(2023));

        let track = &album.tracks[0];
        assert_eq!(track.metadata.title.as_ref().unwrap().value, "First Track");
        assert_eq!(track.metadata.track_number.as_ref().unwrap().value, 1);
        assert_eq!(track.metadata.duration.as_ref().unwrap().value, 180.5);
    }

    #[test]
    fn test_tree_command_with_real_files() {
        let dir = tempdir().unwrap();

        // Create directory structure
        let artist_dir = dir.path().join("ArtistA");
        let album_dir = artist_dir.join("Album1");
        fs::create_dir_all(&album_dir).unwrap();

        // Create test files
        let track1 = album_dir.join("track1.flac");
        let track2 = album_dir.join("track2.flac");
        fs::write(&track1, b"dummy flac data").unwrap();
        fs::write(&track2, b"dummy flac data").unwrap();

        // Test scan and hierarchy building
        let tracks = scan_dir(dir.path());
        let library = build_library_hierarchy(tracks);

        assert_eq!(library.total_tracks, 2);
        assert_eq!(library.total_artists, 1);
        assert_eq!(library.total_albums, 1);

        let artist = &library.artists[0];
        assert!(artist.name.contains("ArtistA") || artist.name == "Unknown Artist");

        let album = &artist.albums[0];
        assert!(album.title.contains("Album1") || album.title == "Unknown Album");
        assert_eq!(album.tracks.len(), 2);
    }

    #[test]
    fn test_tree_with_missing_metadata() {
        let dir = tempdir().unwrap();

        // Create file without proper directory structure
        let orphan_file = dir.path().join("orphan.flac");
        fs::write(&orphan_file, b"dummy flac data").unwrap();

        let tracks = scan_dir(dir.path());
        let library = build_library_hierarchy(tracks);

        assert_eq!(library.total_tracks, 1);
        assert_eq!(library.total_artists, 1);
        assert_eq!(library.total_albums, 1);

        let artist = &library.artists[0];
        // Artist is inferred from parent directory (or a fallback)
        assert!(artist.name.len() > 0);

        let album = &artist.albums[0];
        // Album is inferred from immediate parent directory
        assert!(album.title.len() > 0);
        assert_eq!(album.tracks.len(), 1);
    }

    #[test]
    fn test_tree_json_serialization() {
        let library = create_test_library();

        // Test that library can be serialized to JSON without errors
        let json_result = serde_json::to_string_pretty(&library);
        assert!(json_result.is_ok());

        let json_str = json_result.unwrap();

        // Test that JSON contains expected structure
        assert!(json_str.contains("artists"));
        assert!(json_str.contains("Test Artist"));
        assert!(json_str.contains("First Album"));
        assert!(json_str.contains("total_tracks"));
        assert!(json_str.contains("total_artists"));
        assert!(json_str.contains("total_albums"));

        // Test that it can be deserialized back
        let deserialized: Result<Library, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());

        let restored_library = deserialized.unwrap();
        assert_eq!(restored_library.total_artists, library.total_artists);
        assert_eq!(restored_library.total_albums, library.total_albums);
        assert_eq!(restored_library.total_tracks, library.total_tracks);
    }

    #[test]
    fn test_tree_with_multiple_artists_and_albums() {
        let dir = tempdir().unwrap();

        // Create multiple artists and albums
        for artist_name in ["ArtistA", "ArtistB"] {
            for album_name in ["Album1", "Album2"] {
                let album_dir = dir.path().join(artist_name).join(album_name);
                fs::create_dir_all(&album_dir).unwrap();

                for track_name in ["track1.flac", "track2.flac"] {
                    let track_path = album_dir.join(track_name);
                    fs::write(&track_path, b"dummy flac data").unwrap();
                }
            }
        }

        let tracks = scan_dir(dir.path());
        let library = build_library_hierarchy(tracks);

        assert_eq!(library.total_tracks, 8); // 2 artists * 2 albums * 2 tracks
        assert_eq!(library.total_artists, 2);
        assert_eq!(library.total_albums, 4);

        // Verify each artist has 2 albums
        for artist in &library.artists {
            assert_eq!(artist.albums.len(), 2);

            // Verify each album has 2 tracks
            for album in &artist.albums {
                assert_eq!(album.tracks.len(), 2);
            }
        }
    }

    #[test]
    fn test_tree_with_varied_file_formats() {
        let dir = tempdir().unwrap();

        let artist_dir = dir.path().join("TestArtist");
        let album_dir = artist_dir.join("TestAlbum");
        fs::create_dir_all(&album_dir).unwrap();

        // Create files with different formats
        let formats = ["flac", "FLAC", "mp3", "wav"];
        for (i, format) in formats.iter().enumerate() {
            let track_path = album_dir.join(format!("track{}.{}", i, format));
            fs::write(&track_path, b"dummy data").unwrap();
        }

        let tracks = scan_dir(dir.path());
        let library = build_library_hierarchy(tracks);

        // Should find FLAC and MP3 files (case insensitive)
        assert_eq!(library.total_tracks, 3); // flac, FLAC, and mp3 (wav not supported)

        let tracks = &library.artists[0].albums[0].tracks;
        let track_formats: Vec<_> = tracks.iter().map(|t| &t.metadata.format).collect();

        assert!(track_formats.contains(&&"flac".to_string()));
        assert!(track_formats.contains(&&"mp3".to_string()));
    }
}
