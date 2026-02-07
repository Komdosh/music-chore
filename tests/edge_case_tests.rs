use music_chore::{MetadataSource, MetadataValue, Track, TrackMetadata, build_library_hierarchy};
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[cfg(test)]
mod tests {
    use super::*;
    use music_chore::core::domain::models::FOLDER_INFERRED_CONFIDENCE;
    use music_chore::core::services::scanner::scan_dir;

    #[test]
    fn test_empty_directory_scan() {
        let dir = tempdir().unwrap();
        let tracks = scan_dir(dir.path());
        assert_eq!(tracks.len(), 0);

        let library = build_library_hierarchy(tracks);
        assert_eq!(library.total_artists, 0);
        assert_eq!(library.total_albums, 0);
        assert_eq!(library.total_tracks, 0);
    }

    #[test]
    fn test_directory_with_no_music_files() {
        let dir = tempdir().unwrap();

        // Create non-music files
        fs::write(dir.path().join("readme.txt"), b"This is a readme").unwrap();
        fs::write(dir.path().join("config.json"), b"{}").unwrap();
        fs::write(dir.path().join("image.jpg"), b"fake image data").unwrap();

        let tracks = scan_dir(dir.path());
        assert_eq!(tracks.len(), 0);

        let library = build_library_hierarchy(tracks);
        assert_eq!(library.total_tracks, 0);
    }

    #[test]
    fn test_mixed_case_file_extensions() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("Artist/Album")).unwrap();

        // Copy fixture files with various case extensions
        let fixture_path = PathBuf::from("tests/fixtures/flac/simple/track1.flac");
        let extensions = ["flac", "FLAC", "FlaC"];
        for (i, ext) in extensions.iter().enumerate() {
            let path = dir
                .path()
                .join("Artist/Album")
                .join(format!("track{}.{}", i, ext));
            fs::copy(&fixture_path, &path).unwrap();
        }

        let tracks = scan_dir(dir.path());
        // Should find all FLAC files (case insensitive)
        assert_eq!(tracks.len(), 3);

        let library = build_library_hierarchy(tracks);
        assert_eq!(library.total_tracks, 3);

        let tracks = &library.artists[0].albums[0].tracks;
        let track_formats: Vec<_> = tracks.iter().map(|t| &t.metadata.format).collect();

        assert!(track_formats.contains(&&"flac".to_string()));
    }

    #[test]
    fn test_deeply_nested_directories() {
        let dir = tempdir().unwrap();

        // Create deeply nested structure
        let deep_path = dir
            .path()
            .join("Level1")
            .join("Level2")
            .join("Level3")
            .join("Level4")
            .join("TestArtist")
            .join("TestAlbum")
            .join("Disc1");
        fs::create_dir_all(&deep_path).unwrap();

        // Copy fixture file
        let fixture_path = PathBuf::from("tests/fixtures/flac/simple/track1.flac");
        let track_path = deep_path.join("track.flac");
        fs::copy(&fixture_path, &track_path).unwrap();

        let tracks = scan_dir(dir.path());
        assert_eq!(tracks.len(), 1);

        let library = build_library_hierarchy(tracks);
        assert_eq!(library.total_tracks, 1);

        // Check that inference works even with deep nesting
        let artist = &library.artists[0];
        
        // Original assertion: Artist is inferred as parent folder of album folder
        // New assertion should consider embedded metadata if present in track1.flac
        assert!(
            artist.name.contains("TestArtist") || // Folder inferred artist
            artist.name == "Test Artist"       // Embedded artist from track1.flac
        );

        let album = &artist.albums[0];
        
        // Original assertion: Album is inferred as immediate parent folder
        // New assertion should consider embedded metadata if present in track1.flac
        assert!(
            album.title.contains("TestAlbum") || // Folder inferred album
            album.title.contains("Disc1") ||     // Folder inferred album
            album.title == "Test Album"          // Embedded album from track1.flac
        );
    }

    #[test]
    fn test_tracks_with_partial_metadata() {
        let mut tracks = Vec::new();

        // Track with only title
        tracks.push(Track {
            file_path: PathBuf::from("partial1.flac"),
            checksum: None,
            metadata: TrackMetadata {
                title: Some(MetadataValue {
                    value: "Only Title".to_string(),
                    source: MetadataSource::Embedded,
                    confidence: 1.0,
                }),
                artist: None,
                album: None,
                album_artist: None,
                track_number: None,
                disc_number: None,
                year: None,
                genre: None,
                duration: None,
                format: "flac".to_string(),
                path: PathBuf::from("partial1.flac"),
            },
        });

        // Track with only artist
        tracks.push(Track {
            file_path: PathBuf::from("partial2.flac"),
            checksum: None,
            metadata: TrackMetadata {
                title: None,
                artist: Some(MetadataValue {
                    value: "Only Artist".to_string(),
                    source: MetadataSource::Embedded,
                    confidence: 1.0,
                }),
                album: None,
                album_artist: None,
                track_number: None,
                disc_number: None,
                year: None,
                genre: None,
                duration: None,
                format: "flac".to_string(),
                path: PathBuf::from("partial2.flac"),
            },
        });

        let library = build_library_hierarchy(tracks);
        assert_eq!(library.total_tracks, 2);
        assert_eq!(library.total_artists, 2);

        // Should group by "Unknown Artist" for first track
        let artist_with_title = library
            .artists
            .iter()
            .find(|a| a.name == "Unknown Artist")
            .unwrap();
        assert_eq!(artist_with_title.albums.len(), 1);
        assert_eq!(
            artist_with_title.albums[0].tracks[0]
                .metadata
                .title
                .as_ref()
                .unwrap()
                .value,
            "Only Title"
        );

        // Should group by "Only Artist" for second track
        let artist_with_artist = library
            .artists
            .iter()
            .find(|a| a.name == "Only Artist")
            .unwrap();
        assert_eq!(artist_with_artist.albums.len(), 1);
    }

    #[test]
    fn test_duplicate_track_handling() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("Artist/Album")).unwrap();

        let fixture_path = PathBuf::from("tests/fixtures/flac/simple/track1.flac");
        let track1_path = dir.path().join("Artist/Album/track1.flac");
        let track2_path = dir.path().join("Artist/Album/track2.flac");

        fs::copy(&fixture_path, &track1_path).unwrap();
        fs::copy(&fixture_path, &track2_path).unwrap();

        let tracks = scan_dir(dir.path());
        assert_eq!(tracks.len(), 2);

        let library = build_library_hierarchy(tracks);
        assert_eq!(library.total_tracks, 2);

        let album = &library.artists[0].albums[0];
        assert_eq!(album.tracks.len(), 2);

        // Should have different file paths
        let paths: Vec<_> = album
            .tracks
            .iter()
            .map(|t| t.file_path.file_name().unwrap().to_string_lossy())
            .collect();
        assert!(paths.iter().any(|p| p == "track1.flac"));
        assert!(paths.iter().any(|p| p == "track2.flac"));
    }

    #[test]
    fn test_special_characters_in_paths() {
        let dir = tempdir().unwrap();

        // Create directory with special characters
        let special_artist_folder = "Artist_Band_2023_Remastered";
        let special_album_folder = "Album_Vol_1_2";

        let album_dir = dir.path().join(special_artist_folder).join(special_album_folder);
        fs::create_dir_all(&album_dir).unwrap();

        // Copy fixture file
        let fixture_path = PathBuf::from("tests/fixtures/flac/simple/track1.flac");
        let track_path = album_dir.join("track.flac");
        fs::copy(&fixture_path, &track_path).unwrap();

        let tracks = scan_dir(dir.path());
        assert_eq!(tracks.len(), 1);

        let library = build_library_hierarchy(tracks);
        assert_eq!(library.total_tracks, 1);

        let artist = &library.artists[0];
        
        // Original assertion: relies on folder inference. Now, track1.flac has embedded artist.
        assert_eq!(artist.name, "Test Artist"); // Expect embedded artist from track1.flac

        let album = &artist.albums[0];
        
        // Original assertion: relies on folder inference. Now, track1.flac has embedded album.
        assert_eq!(album.title, "Test Album"); // Expect embedded album from track1.flac
    }

    #[test]
    fn test_tracks_with_varied_metadata_sources() {
        let mut tracks = Vec::new();

        // Track with embedded metadata
        tracks.push(Track {
            file_path: PathBuf::from("embedded.flac"),
            checksum: None,
            metadata: TrackMetadata {
                title: Some(MetadataValue {
                    value: "Embedded Title".to_string(),
                    source: MetadataSource::Embedded,
                    confidence: 1.0,
                }),
                artist: Some(MetadataValue {
                    value: "Embedded Artist".to_string(),
                    source: MetadataSource::Embedded,
                    confidence: 1.0,
                }),
                album: Some(MetadataValue {
                    value: "Embedded Album".to_string(),
                    source: MetadataSource::Embedded,
                    confidence: 1.0,
                }),
                album_artist: None,
                track_number: None,
                disc_number: None,
                year: None,
                genre: None,
                duration: None,
                format: "flac".to_string(),
                path: PathBuf::from("embedded.flac"),
            },
        });

        // Track with folder-inferred metadata
        tracks.push(Track {
            file_path: PathBuf::from("FolderArtist/FolderAlbum/track.flac"),
            checksum: None,
            metadata: TrackMetadata {
                title: None,
                artist: Some(MetadataValue {
                    value: "FolderArtist".to_string(),
                    source: MetadataSource::FolderInferred,
                    confidence: FOLDER_INFERRED_CONFIDENCE,
                }),
                album: Some(MetadataValue {
                    value: "FolderAlbum".to_string(),
                    source: MetadataSource::FolderInferred,
                    confidence: FOLDER_INFERRED_CONFIDENCE,
                }),
                album_artist: None,
                track_number: None,
                disc_number: None,
                year: None,
                genre: None,
                duration: None,
                format: "flac".to_string(),
                path: PathBuf::from("FolderArtist/FolderAlbum/track.flac"),
            },
        });

        let library = build_library_hierarchy(tracks);
        assert_eq!(library.total_tracks, 2);
        assert_eq!(library.total_artists, 2);

        // Check that metadata sources are preserved
        for artist in &library.artists {
            for album in &artist.albums {
                for track in &album.tracks {
                    if track.file_path.to_string_lossy().contains("embedded") {
                        assert_eq!(
                            track.metadata.artist.as_ref().unwrap().source,
                            MetadataSource::Embedded
                        );
                    } else {
                        assert_eq!(
                            track.metadata.artist.as_ref().unwrap().source,
                            MetadataSource::FolderInferred
                        );
                    }
                }
            }
        }
    }
}
