use music_chore::{build_library_hierarchy, MetadataSource, MetadataValue, Track, TrackMetadata};
use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_track(
        path: &str,
        artist: Option<&str>,
        album: Option<&str>,
        title: Option<&str>,
    ) -> Track {
        Track {
            file_path: PathBuf::from(path),
            metadata: TrackMetadata {
                title: title.map(|t| MetadataValue {
                    value: t.to_string(),
                    source: MetadataSource::Embedded,
                    confidence: 1.0,
                }),
                artist: artist.map(|a| MetadataValue {
                    value: a.to_string(),
                    source: MetadataSource::Embedded,
                    confidence: 1.0,
                }),
                album: album.map(|a| MetadataValue {
                    value: a.to_string(),
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
                path: PathBuf::from(path),
            },
        }
    }

    #[test]
    fn test_build_library_hierarchy_single_artist_single_album() {
        let tracks = vec![
            create_test_track(
                "Artist1/Album1/track1.flac",
                Some("Artist1"),
                Some("Album1"),
                Some("Track1"),
            ),
            create_test_track(
                "Artist1/Album1/track2.flac",
                Some("Artist1"),
                Some("Album1"),
                Some("Track2"),
            ),
        ];

        let library = build_library_hierarchy(tracks);

        assert_eq!(library.total_artists, 1);
        assert_eq!(library.total_albums, 1);
        assert_eq!(library.total_tracks, 2);
        assert_eq!(library.artists.len(), 1);

        let artist = &library.artists[0];
        assert_eq!(artist.name, "Artist1");
        assert_eq!(artist.albums.len(), 1);

        let album = &artist.albums[0];
        assert_eq!(album.title, "Album1");
        assert_eq!(album.tracks.len(), 2);
    }

    #[test]
    fn test_build_library_hierarchy_multiple_artists() {
        let tracks = vec![
            create_test_track(
                "ArtistA/Album1/track1.flac",
                Some("ArtistA"),
                Some("Album1"),
                Some("Track1"),
            ),
            create_test_track(
                "ArtistB/Album1/track1.flac",
                Some("ArtistB"),
                Some("Album1"),
                Some("Track1"),
            ),
            create_test_track(
                "ArtistA/Album2/track1.flac",
                Some("ArtistA"),
                Some("Album2"),
                Some("Track1"),
            ),
        ];

        let library = build_library_hierarchy(tracks);

        assert_eq!(library.total_artists, 2);
        assert_eq!(library.total_albums, 3);
        assert_eq!(library.total_tracks, 3);
        assert_eq!(library.artists.len(), 2);

        // Check ArtistA has 2 albums
        let artist_a = library
            .artists
            .iter()
            .find(|a| a.name == "ArtistA")
            .unwrap();
        assert_eq!(artist_a.albums.len(), 2);

        // Check ArtistB has 1 album
        let artist_b = library
            .artists
            .iter()
            .find(|a| a.name == "ArtistB")
            .unwrap();
        assert_eq!(artist_b.albums.len(), 1);
    }

    #[test]
    fn test_build_library_hierarchy_missing_metadata() {
        let tracks = vec![Track {
            file_path: PathBuf::from("UnknownArtist/UnknownAlbum/track1.flac"),
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
                path: PathBuf::from("UnknownArtist/UnknownAlbum/track1.flac"),
            },
        }];

        let library = build_library_hierarchy(tracks);

        assert_eq!(library.total_artists, 1);
        assert_eq!(library.total_albums, 1);
        assert_eq!(library.total_tracks, 1);

        let artist = &library.artists[0];
        assert_eq!(artist.name, "Unknown Artist");

        let album = &artist.albums[0];
        assert_eq!(album.title, "Unknown Album");
    }

    #[test]
    fn test_build_library_hierarchy_empty_input() {
        let tracks: Vec<Track> = vec![];
        let library = build_library_hierarchy(tracks);

        assert_eq!(library.total_artists, 0);
        assert_eq!(library.total_albums, 0);
        assert_eq!(library.total_tracks, 0);
        assert_eq!(library.artists.len(), 0);
    }

    #[test]
    fn test_build_library_hierarchy_preserves_metadata() {
        let tracks = vec![create_test_track(
            "Artist/Album/track1.flac",
            Some("Artist"),
            Some("Album"),
            Some("Track 1"),
        )];

        let library = build_library_hierarchy(tracks);
        let track_node = &library.artists[0].albums[0].tracks[0];

        // Check that metadata is preserved
        assert_eq!(track_node.metadata.title.as_ref().unwrap().value, "Track 1");
        assert_eq!(track_node.metadata.artist.as_ref().unwrap().value, "Artist");
        assert_eq!(track_node.metadata.album.as_ref().unwrap().value, "Album");
        assert_eq!(track_node.metadata.format, "flac");
    }

    #[test]
    fn test_build_library_hierarchy_with_years() {
        let mut track = create_test_track(
            "Artist/Album/track1.flac",
            Some("Artist"),
            Some("Album (2023)"),
            Some("Track 1"),
        );

        // Add year to metadata
        track.metadata.year = Some(MetadataValue {
            value: 2023,
            source: MetadataSource::Embedded,
            confidence: 1.0,
        });

        let library = build_library_hierarchy(vec![track]);
        let album = &library.artists[0].albums[0];

        assert_eq!(album.year, Some(2023));
    }
}
