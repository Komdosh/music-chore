//! Inference logic for extracting metadata from file paths and directory structures.

use std::path::Path;
use crate::domain::{Artist, Album, Track};
use crate::domain::track::Provenance;
use crate::domain::metadata::Metadata;

/// Infers metadata from a file path and directory structure
pub fn infer_metadata(file_path: &str) -> Metadata {
    let mut metadata = Metadata::new();

    // Parse the file path to extract information
    let path = Path::new(file_path);
    let mut components = path.components().collect::<Vec<_>>();

    // Extract album name from the parent directory
    if components.len() >= 2 {
        let album_name = components[components.len() - 2].as_os_str().to_string_lossy().to_string();
        metadata.inferred.insert("album".to_string(), album_name);
    }

    // Extract artist name from the grandparent directory
    if components.len() >= 3 {
        let artist_name = components[components.len() - 3].as_os_str().to_string_lossy().to_string();
        metadata.inferred.insert("artist".to_string(), artist_name);
    }

    // Extract track title from filename (without extension)
    if let Some(filename) = path.file_stem() {
        let filename_str = filename.to_string_lossy().to_string();
        metadata.inferred.insert("title".to_string(), filename_str);
    }

    // Set confidence scores for inferred fields
    metadata.set_confidence("album".to_string(), 0.8);
    metadata.set_confidence("artist".to_string(), 0.8);
    metadata.set_confidence("title".to_string(), 0.9);

    metadata
}

/// Infers artist from a directory path
pub fn infer_artist_from_path(path: &str) -> Option<Artist> {
    let path_components: Vec<&str> = path.split('/').collect();

    if path_components.len() >= 1 {
        let artist_name = path_components[path_components.len() - 1];
        Some(Artist::new(
            artist_name.to_string(),
            Provenance::Inferred,
        ))
    } else {
        None
    }
}

/// Infers album from a directory path
pub fn infer_album_from_path(path: &str) -> Option<Album> {
    let path_components: Vec<&str> = path.split('/').collect();

    if path_components.len() >= 2 {
        let album_name = path_components[path_components.len() - 1];
        let artist_name = path_components[path_components.len() - 2];

        // Create a basic artist for the album
        let artist = Artist::new(artist_name.to_string(), Provenance::Inferred);

        Some(Album::new(
            album_name.to_string(),
            artist,
            None, // year
            None, // genre
            Vec::new(), // tracks
            Provenance::Inferred,
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_metadata() {
        let metadata = infer_metadata("/Users/user/Music/Artist/Album/Song.flac");
        assert!(metadata.inferred.contains_key("album"));
        assert!(metadata.inferred.contains_key("artist"));
        assert!(metadata.inferred.contains_key("title"));
    }

    #[test]
    fn test_infer_artist_from_path() {
        let artist = infer_artist_from_path("/Users/user/Music/Artist");
        assert!(artist.is_some());
        assert_eq!(artist.unwrap().name, "Artist");
    }

    #[test]
    fn test_infer_album_from_path() {
        let album = infer_album_from_path("/Users/user/Music/Artist/Album");
        assert!(album.is_some());
        assert_eq!(album.as_ref().unwrap().title, "Album");
        assert_eq!(album.as_ref().unwrap().artist.name, "Artist");
    }
}