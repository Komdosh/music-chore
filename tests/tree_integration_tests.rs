// Test the tree command functionality via CLI integration
#[cfg(test)]
mod tests {
    use music_chore::core::services::scanner::scan_dir;
    use music_chore::{
        AlbumNode, ArtistNode, Library, MetadataSource, MetadataValue, TrackNode,
        build_library_hierarchy,
    };
    use std::collections::HashSet;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn create_test_library() -> Library {
        let mut library = Library::new();

        let track_node_for_album1 = TrackNode {
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
        };

        let album1_files: HashSet<PathBuf> = vec![
            PathBuf::from("Test Artist/First Album/01 Track.flac"),
        ].into_iter().collect();

        // Create first artist with multiple albums
        let artist1 = ArtistNode {
            name: "Test Artist".to_string(),
            albums: vec![AlbumNode {
                title: "First Album".to_string(),
                year: Some(2023),
                tracks: vec![track_node_for_album1],
                files: album1_files,
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

        // Copy test files from fixtures
        let fixture_path = PathBuf::from("tests/fixtures/flac/simple/track1.flac");
        let track1 = album_dir.join("track1.flac");
        let track2 = album_dir.join("track2.flac");
        fs::copy(&fixture_path, &track1).unwrap();
        fs::copy(&fixture_path, &track2).unwrap();

        // Test scan and hierarchy building
        let tracks = scan_dir(dir.path(), false);
        let library = build_library_hierarchy(tracks);

        assert_eq!(library.total_tracks, 2);
        assert_eq!(library.total_artists, 1);
        assert_eq!(library.total_albums, 1);

        let artist = &library.artists[0];
        assert_eq!(artist.name, "Test Artist"); // Expect embedded artist from track1.flac

        let album = &artist.albums[0];
        assert_eq!(album.title, "Test Album"); // Expect embedded album from track1.flac
        assert_eq!(album.tracks.len(), 2);
    }

    #[test]
    fn test_tree_with_missing_metadata() {
        let dir = tempdir().unwrap();

        // Copy file without proper directory structure
        let fixture_path = PathBuf::from("tests/fixtures/flac/simple/track1.flac");
        let orphan_file = dir.path().join("orphan.flac");
        fs::copy(&fixture_path, &orphan_file).unwrap();

        let tracks = scan_dir(dir.path(), false);
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

        // Copy fixture files for multiple artists and albums
        // Use the truly untagged FLAC file to ensure folder inference for artist/album
        let untagged_fixture_path = PathBuf::from("tests/fixtures/artist_bracket/Some guy [FLAC]/05. Shard/no_metadata.flac");

        for artist_name in ["ArtistA", "ArtistB"] {
            for album_name in ["Album1", "Album2"] {
                let album_dir = dir.path().join(artist_name).join(album_name);
                fs::create_dir_all(&album_dir).unwrap();

                for (_i, track_filename) in ["track1.flac", "track2.flac"].iter().enumerate() { // Use two untagged FLACs
                    let track_path = album_dir.join(track_filename);
                    fs::copy(&untagged_fixture_path, &track_path).unwrap();
                }
            }
        }

        let tracks = scan_dir(dir.path(), false);
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

        // Use the truly untagged FLAC file for all formats
        let untagged_fixture_path = PathBuf::from("tests/fixtures/artist_bracket/Some guy [FLAC]/05. Shard/no_metadata.flac");

        let track1 = album_dir.join("track1.flac");
        let track2 = album_dir.join("track2.FLAC");
        let track3 = album_dir.join("track3.mp3"); // Copy untagged FLAC as mp3
        let track4 = album_dir.join("track4.wav"); // Copy untagged FLAC as wav

        fs::copy(&untagged_fixture_path, &track1).unwrap();
        fs::copy(&untagged_fixture_path, &track2).unwrap();
        fs::copy(&untagged_fixture_path, &track3).unwrap();
        fs::copy(&untagged_fixture_path, &track4).unwrap();

        let tracks = scan_dir(dir.path(), false);
        let library = build_library_hierarchy(tracks);

        // Should find FLAC, MP3, and WAV files (case insensitive)
        assert_eq!(library.total_tracks, 4); // flac, FLAC, mp3, and wav
        assert_eq!(library.total_artists, 1); // All untagged, so one folder inferred artist
        assert_eq!(library.total_albums, 1);  // All untagged, so one folder inferred album


        let tracks = &library.artists[0].albums[0].tracks;
        let track_formats: Vec<_> = tracks.iter().map(|t| &t.metadata.format).collect();

        assert!(track_formats.contains(&&"flac".to_string()));
        assert!(track_formats.contains(&&"mp3".to_string()));
        assert!(track_formats.contains(&&"wav".to_string()));
    }
}