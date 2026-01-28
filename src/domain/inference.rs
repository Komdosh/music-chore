use std::path::Path;
use crate::domain::{Artist, Album, Track};
use crate::domain::artist::Provenance as ArtistProvenance;
use crate::domain::album::Provenance as AlbumProvenance;
use crate::domain::metadata::Metadata;

/// Infers metadata from a file path and directory structure
pub fn infer_metadata(file_path: &str) -> Metadata {
    let mut metadata = Metadata::new();
    let path = Path::new(file_path);
    let components = path.components().collect::<Vec<_>>();
    if components.len() >= 2 {
        let album_name = components[components.len() - 2].as_os_str().to_string_lossy().to_string();
        metadata.inferred.insert("album".to_string(), album_name);
    }
    if components.len() >= 3 {
        let artist_name = components[components.len() - 3].as_os_str().to_string_lossy().to_string();
        metadata.inferred.insert("artist".to_string(), artist_name);
    }
    if let Some(filename) = path.file_stem() {
        let filename_str = filename.to_string_lossy().to_string();
        metadata.inferred.insert("title".to_string(), filename_str);
    }
    metadata
}

/// Infers artist from a directory path
pub fn infer_artist_from_path(path: &str) -> Option<Artist> {
    let components: Vec<&str> = path.split('/').collect();
    if components.is_empty() {
        return None;
    }
    let artist_name = components.last().unwrap();
    Some(Artist::new(artist_name.to_string(), ArtistProvenance::Inferred))
}

/// Infers album from a directory path
pub fn infer_album_from_path(path: &str) -> Option<Album> {
    let components: Vec<&str> = path.split('/').collect();
    if components.len() < 2 {
        return None;
    }
    let album_name = components.last().unwrap();
    let artist_name = components[components.len() - 2];
    let artist = Artist::new(artist_name.to_string(), ArtistProvenance::Inferred);
    Some(Album::new(
        album_name.to_string(),
        artist,
        None,
        None,
        Vec::new(),
        AlbumProvenance::Inferred,
    ))
}
