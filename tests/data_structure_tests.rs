use music_chore::{
    AlbumNode, ArtistNode, Library, MetadataSource, MetadataValue, Track, TrackMetadata, TrackNode,
};
use serde_json;
use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use music_chore::domain::models::FOLDER_INFERRED_CONFIDENCE;
    use super::*;

    fn create_test_metadata_value<T: Clone>(
        value: T,
        source: MetadataSource,
        confidence: f32,
    ) -> MetadataValue<T> {
        MetadataValue {
            value,
            source,
            confidence,
        }
    }

    #[test]
    fn test_metadata_value_creation() {
        let mv =
            create_test_metadata_value("Test Title".to_string(), MetadataSource::Embedded, 1.0);

        assert_eq!(mv.value, "Test Title");
        assert_eq!(mv.source, MetadataSource::Embedded);
        assert_eq!(mv.confidence, 1.0);
    }

    #[test]
    fn test_track_metadata_creation() {
        let metadata = TrackMetadata {
            title: Some(create_test_metadata_value(
                "Test Track".to_string(),
                MetadataSource::Embedded,
                1.0,
            )),
            artist: Some(create_test_metadata_value(
                "Test Artist".to_string(),
                MetadataSource::FolderInferred,
                FOLDER_INFERRED_CONFIDENCE,
            )),
            album: Some(create_test_metadata_value(
                "Test Album".to_string(),
                MetadataSource::Embedded,
                1.0,
            )),
            album_artist: None,
            track_number: Some(create_test_metadata_value(5, MetadataSource::Embedded, 1.0)),
            disc_number: Some(create_test_metadata_value(1, MetadataSource::Embedded, 1.0)),
            year: Some(create_test_metadata_value(
                2023,
                MetadataSource::Embedded,
                1.0,
            )),
            genre: Some(create_test_metadata_value(
                "Rock".to_string(),
                MetadataSource::Embedded,
                1.0,
            )),
            duration: Some(create_test_metadata_value(
                240.5,
                MetadataSource::Embedded,
                1.0,
            )),
            format: "flac".to_string(),
            path: PathBuf::from("/test/track.flac"),
        };

        assert_eq!(metadata.title.unwrap().value, "Test Track");
        assert_eq!(
            metadata.artist.unwrap().source,
            MetadataSource::FolderInferred
        );
        assert_eq!(metadata.track_number.unwrap().value, 5);
        assert_eq!(metadata.year.unwrap().value, 2023);
        assert_eq!(metadata.duration.unwrap().value, 240.5);
        assert_eq!(metadata.format, "flac");
    }

    #[test]
    fn test_track_creation() {
        let metadata = TrackMetadata {
            title: Some(create_test_metadata_value(
                "Track".to_string(),
                MetadataSource::Embedded,
                1.0,
            )),
            artist: Some(create_test_metadata_value(
                "Artist".to_string(),
                MetadataSource::Embedded,
                1.0,
            )),
            album: Some(create_test_metadata_value(
                "Album".to_string(),
                MetadataSource::Embedded,
                1.0,
            )),
            album_artist: None,
            track_number: None,
            disc_number: None,
            year: None,
            genre: None,
            duration: None,
            format: "flac".to_string(),
            path: PathBuf::from("/test/track.flac"),
        };

        let track = Track {
            file_path: PathBuf::from("/test/track.flac"),
            checksum: None,
            metadata,
        };

        assert_eq!(track.file_path, PathBuf::from("/test/track.flac"));
        assert_eq!(track.metadata.title.unwrap().value, "Track");
        assert_eq!(track.metadata.format, "flac");
    }

    #[test]
    fn test_track_node_creation() {
        let metadata = TrackMetadata {
            title: Some(create_test_metadata_value(
                "Node Track".to_string(),
                MetadataSource::Embedded,
                1.0,
            )),
            artist: Some(create_test_metadata_value(
                "Node Artist".to_string(),
                MetadataSource::Embedded,
                1.0,
            )),
            album: Some(create_test_metadata_value(
                "Node Album".to_string(),
                MetadataSource::Embedded,
                1.0,
            )),
            album_artist: None,
            track_number: None,
            disc_number: None,
            year: None,
            genre: None,
            duration: None,
            format: "flac".to_string(),
            path: PathBuf::from("/test/node_track.flac"),
        };

        let track_node = TrackNode {
            file_path: PathBuf::from("/test/node_track.flac"),
            metadata,
        };

        assert_eq!(track_node.file_path, PathBuf::from("/test/node_track.flac"));
        assert_eq!(track_node.metadata.title.unwrap().value, "Node Track");
    }

    #[test]
    fn test_album_node_creation() {
        let track_node = TrackNode {
            file_path: PathBuf::from("/test/album/track.flac"),
            metadata: TrackMetadata {
                title: Some(create_test_metadata_value(
                    "Album Track".to_string(),
                    MetadataSource::Embedded,
                    1.0,
                )),
                artist: None,
                album: None,
                album_artist: None,
                track_number: None,
                disc_number: None,
                year: None,
                genre: None,
                duration: None,
                format: "flac".to_string(),
                path: PathBuf::from("/test/album/track.flac"),
            },
        };

        let album_node = AlbumNode {
            title: "Test Album".to_string(),
            year: Some(2023),
            tracks: vec![track_node],
            path: PathBuf::from("/test/album"),
        };

        assert_eq!(album_node.title, "Test Album");
        assert_eq!(album_node.year, Some(2023));
        assert_eq!(album_node.tracks.len(), 1);
        assert_eq!(album_node.path, PathBuf::from("/test/album"));
    }

    #[test]
    fn test_artist_node_creation() {
        let album_node = AlbumNode {
            title: "Artist Album".to_string(),
            year: None,
            tracks: vec![],
            path: PathBuf::from("/test/artist_album"),
        };

        let artist_node = ArtistNode {
            name: "Test Artist".to_string(),
            albums: vec![album_node],
        };

        assert_eq!(artist_node.name, "Test Artist");
        assert_eq!(artist_node.albums.len(), 1);
        assert_eq!(artist_node.albums[0].title, "Artist Album");
    }

    #[test]
    fn test_library_creation_and_addition() {
        let mut library = Library::new();

        assert_eq!(library.total_artists, 0);
        assert_eq!(library.total_albums, 0);
        assert_eq!(library.total_tracks, 0);
        assert_eq!(library.artists.len(), 0);

        let artist_node = ArtistNode {
            name: "Library Artist".to_string(),
            albums: vec![
                AlbumNode {
                    title: "Album 1".to_string(),
                    year: Some(2020),
                    tracks: vec![
                        TrackNode {
                            file_path: PathBuf::from("/album1/track1.flac"),
                            metadata: TrackMetadata {
                                title: None,
                                artist: None,
                                album: None,
                                album_artist: None,
                                track_number: None,
                                disc_number: None,
                                year: None,
                                genre: None,
                                duration: None,
                                format: "flac".to_string(),
                                path: PathBuf::from("/album1/track1.flac"),
                            },
                        },
                        TrackNode {
                            file_path: PathBuf::from("/album1/track2.flac"),
                            metadata: TrackMetadata {
                                title: None,
                                artist: None,
                                album: None,
                                album_artist: None,
                                track_number: None,
                                disc_number: None,
                                year: None,
                                genre: None,
                                duration: None,
                                format: "flac".to_string(),
                                path: PathBuf::from("/album1/track2.flac"),
                            },
                        },
                    ],
                    path: PathBuf::from("/album1"),
                },
                AlbumNode {
                    title: "Album 2".to_string(),
                    year: Some(2021),
                    tracks: vec![TrackNode {
                        file_path: PathBuf::from("/album2/track1.flac"),
                        metadata: TrackMetadata {
                            title: None,
                            artist: None,
                            album: None,
                            album_artist: None,
                            track_number: None,
                            disc_number: None,
                            year: None,
                            genre: None,
                            duration: None,
                            format: "flac".to_string(),
                            path: PathBuf::from("/album2/track1.flac"),
                        },
                    }],
                    path: PathBuf::from("/album2"),
                },
            ],
        };

        library.add_artist(artist_node);

        assert_eq!(library.total_artists, 1);
        assert_eq!(library.total_albums, 2);
        assert_eq!(library.total_tracks, 3);
        assert_eq!(library.artists.len(), 1);
    }

    #[test]
    fn test_serialization_deserialization() {
        let metadata = TrackMetadata {
            title: Some(create_test_metadata_value(
                "Serial Track".to_string(),
                MetadataSource::Embedded,
                1.0,
            )),
            artist: Some(create_test_metadata_value(
                "Serial Artist".to_string(),
                MetadataSource::FolderInferred,
                FOLDER_INFERRED_CONFIDENCE,
            )),
            album: Some(create_test_metadata_value(
                "Serial Album".to_string(),
                MetadataSource::Embedded,
                1.0,
            )),
            album_artist: None,
            track_number: Some(create_test_metadata_value(7, MetadataSource::Embedded, 1.0)),
            disc_number: Some(create_test_metadata_value(1, MetadataSource::Embedded, 1.0)),
            year: Some(create_test_metadata_value(
                2022,
                MetadataSource::Embedded,
                1.0,
            )),
            genre: Some(create_test_metadata_value(
                "Jazz".to_string(),
                MetadataSource::Embedded,
                1.0,
            )),
            duration: Some(create_test_metadata_value(
                195.3,
                MetadataSource::Embedded,
                1.0,
            )),
            format: "flac".to_string(),
            path: PathBuf::from("/serial/track.flac"),
        };

        let track = Track {
            file_path: PathBuf::from("/serial/track.flac"),
            checksum: None,
            metadata,
        };

        // Test Track serialization
        let json = serde_json::to_string_pretty(&track).unwrap();
        let deserialized: Track = serde_json::from_str(&json).unwrap();

        assert_eq!(track.file_path, deserialized.file_path);
        assert_eq!(
            track.metadata.title.unwrap().value,
            deserialized.metadata.title.unwrap().value
        );
        assert_eq!(
            track.metadata.artist.unwrap().source,
            deserialized.metadata.artist.unwrap().source
        );
        assert_eq!(
            track.metadata.track_number.unwrap().value,
            deserialized.metadata.track_number.unwrap().value
        );
        assert_eq!(
            track.metadata.duration.unwrap().value,
            deserialized.metadata.duration.unwrap().value
        );

        // Test Library serialization
        let mut library = Library::new();
        let artist_node = ArtistNode {
            name: "Serial Artist".to_string(),
            albums: vec![],
        };
        library.add_artist(artist_node);

        let library_json = serde_json::to_string_pretty(&library).unwrap();
        let deserialized_library: Library = serde_json::from_str(&library_json).unwrap();

        assert_eq!(library.total_artists, deserialized_library.total_artists);
        assert_eq!(library.artists.len(), deserialized_library.artists.len());
        assert_eq!(
            library.artists[0].name,
            deserialized_library.artists[0].name
        );
    }

    #[test]
    fn test_metadata_source_variants() {
        let sources = vec![
            MetadataSource::Embedded,
            MetadataSource::FolderInferred,
            MetadataSource::UserEdited,
        ];

        for source in sources {
            let mv = create_test_metadata_value("test".to_string(), source.clone(), 0.9);
            assert_eq!(mv.source, source);
        }
    }

    #[test]
    fn test_equality_implementations() {
        let mv1 = create_test_metadata_value("test".to_string(), MetadataSource::Embedded, 1.0);
        let mv2 = create_test_metadata_value("test".to_string(), MetadataSource::Embedded, 1.0);
        let mv3 =
            create_test_metadata_value("different".to_string(), MetadataSource::Embedded, 1.0);

        assert_eq!(mv1, mv2);
        assert_ne!(mv1, mv3);

        let metadata1 = TrackMetadata {
            title: Some(mv1.clone()),
            artist: None,
            album: None,
            album_artist: None,
            track_number: None,
            disc_number: None,
            year: None,
            genre: None,
            duration: None,
            format: "flac".to_string(),
            path: PathBuf::from("/test.flac"),
        };

        let metadata2 = TrackMetadata {
            title: Some(mv2),
            artist: None,
            album: None,
            album_artist: None,
            track_number: None,
            disc_number: None,
            year: None,
            genre: None,
            duration: None,
            format: "flac".to_string(),
            path: PathBuf::from("/test.flac"),
        };

        let metadata3 = TrackMetadata {
            title: Some(mv3),
            artist: None,
            album: None,
            album_artist: None,
            track_number: None,
            disc_number: None,
            year: None,
            genre: None,
            duration: None,
            format: "flac".to_string(),
            path: PathBuf::from("/test.flac"),
        };

        assert_eq!(metadata1, metadata2);
        assert_ne!(metadata1, metadata3);
    }
}
