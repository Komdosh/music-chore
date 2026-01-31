//! Library hierarchy building from track collections.

use crate::domain::models::{AlbumNode, ArtistNode, Library, Track, TrackNode};
use std::collections::HashMap;
use std::path::PathBuf;

/// Build library hierarchy from flat track list
pub fn build_library_hierarchy(tracks: Vec<Track>) -> Library {
    let mut artists_map: HashMap<String, Vec<Track>> = HashMap::new();

    // Group tracks by artist
    for track in tracks {
        let artist_name = track
            .metadata
            .artist
            .as_ref()
            .map(|a| a.value.clone())
            .unwrap_or_else(|| "Unknown Artist".to_string());

        artists_map.entry(artist_name).or_default().push(track);
    }

    let mut library = Library::new();

    // Build artist -> album -> track hierarchy
    for (artist_name, artist_tracks) in artists_map {
        let mut albums_map: HashMap<String, Vec<Track>> = HashMap::new();

        // Group tracks by album
        for track in artist_tracks {
            let album_name = track
                .metadata
                .album
                .as_ref()
                .map(|a| a.value.clone())
                .unwrap_or_else(|| "Unknown Album".to_string());

            albums_map.entry(album_name).or_default().push(track);
        }

        let mut albums = Vec::new();
        for (album_name, album_tracks) in albums_map {
            // Extract year from first track (assuming all tracks in album have same year)
            let year = album_tracks
                .first()
                .and_then(|t| t.metadata.year.as_ref())
                .map(|y| y.value);

            // Capture album path before moving tracks
            let album_path = album_tracks
                .first()
                .map(|t| t.file_path.parent().unwrap().to_path_buf())
                .unwrap_or_else(|| PathBuf::from(""));

            let mut track_nodes = Vec::new();
            for track in album_tracks {
                track_nodes.push(TrackNode {
                    file_path: track.file_path,
                    metadata: track.metadata,
                });
            }

            albums.push(AlbumNode {
                title: album_name,
                year,
                tracks: track_nodes,
                path: album_path,
            });
        }

        library.add_artist(ArtistNode {
            name: artist_name,
            albums,
        });
    }

    library
}
