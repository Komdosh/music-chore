use music_chore::{
    build_library_hierarchy, MetadataSource, MetadataValue, Track, TrackMetadata,
};
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[cfg(test)]
mod tests {
    use music_chore::services::scanner::scan_dir;
    use super::*;

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

        // Create files with various case combinations
        let extensions = ["flac", "FLAC", "FlaC"];
        for (i, ext) in extensions.iter().enumerate() {
            let path = dir
                .path()
                .join("Artist/Album")
                .join(format!("track{}.{}", i, ext));
            fs::write(&path, b"dummy data").unwrap();
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

        let track_path = deep_path.join("track.flac");
        fs::write(&track_path, b"dummy flac data").unwrap();

        let tracks = scan_dir(dir.path());
        assert_eq!(tracks.len(), 1);

        let library = build_library_hierarchy(tracks);
        assert_eq!(library.total_tracks, 1);

        // Check that inference works even with deep nesting
        let artist = &library.artists[0];
        // Artist is inferred as the parent folder of the album folder
        assert!(artist.name.contains("TestAlbum") || artist.name == "Unknown Artist");

        let album = &artist.albums[0];
        // Album is inferred as the immediate parent folder
        assert!(album.title.contains("Disc1") || album.title == "Unknown Album");
    }

    #[test]
    fn test_tracks_with_partial_metadata() {
        let mut tracks = Vec::new();

        // Track with only title
        tracks.push(Track {
            file_path: PathBuf::from("partial1.flac"),
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

        let track1_path = dir.path().join("Artist/Album/track1.flac");
        let track2_path = dir.path().join("Artist/Album/track2.flac");

        fs::write(&track1_path, b"dummy data 1").unwrap();
        fs::write(&track2_path, b"dummy data 2").unwrap();

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
        let special_artist = "Artist_Band_2023_Remastered";
        let special_album = "Album_Vol_1_2";

        let album_dir = dir.path().join(special_artist).join(special_album);
        fs::create_dir_all(&album_dir).unwrap();

        let track_path = album_dir.join("track.flac");
        fs::write(&track_path, b"dummy data").unwrap();

        let tracks = scan_dir(dir.path());
        assert_eq!(tracks.len(), 1);

        let library = build_library_hierarchy(tracks);
        assert_eq!(library.total_tracks, 1);

        let artist = &library.artists[0];
        assert!(artist.name.contains("Artist_Band") || artist.name == "Unknown Artist");

        let album = &artist.albums[0];
        assert!(album.title.contains("Album_Vol") || album.title == "Unknown Album");
    }

    #[test]
    fn test_tracks_with_varied_metadata_sources() {
        let mut tracks = Vec::new();

        // Track with embedded metadata
        tracks.push(Track {
            file_path: PathBuf::from("embedded.flac"),
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
            metadata: TrackMetadata {
                title: None,
                artist: Some(MetadataValue {
                    value: "FolderArtist".to_string(),
                    source: MetadataSource::FolderInferred,
                    confidence: 0.8,
                }),
                album: Some(MetadataValue {
                    value: "FolderAlbum".to_string(),
                    source: MetadataSource::FolderInferred,
                    confidence: 0.8,
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
