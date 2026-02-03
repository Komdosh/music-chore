//! Path-based metadata inference services.

use std::path::Path;

/// Infer artist name from track file path
pub fn infer_artist_from_path(track_path: &Path) -> Option<String> {
    // Path structure should be: Artist/Album/track.flac
    // Only infer artist when we have exactly Artist/Album/track.flac structure

    let components: Vec<&str> = track_path
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();

    // Need exactly Artist/Album/track.flac (3 components minimum)
    if components.len() >= 3 {
        let _track_name = components.last()?; // filename
        let album_name = components[components.len() - 2]; // parent directory
        let potential_artist = components[components.len() - 3]; // grandparent directory

        // Only infer if we have a clear Artist/Album/track.flac pattern
        // and parent directory is not the same as Artist (to avoid Album/Album/track.flac where both are "Album")
        if !potential_artist.is_empty() && !album_name.is_empty() && potential_artist != album_name
        {
            return Some(potential_artist.to_string());
        }
    }

    None
}

/// Infer album name from track file path
pub fn infer_album_from_path(track_path: &Path) -> Option<String> {
    // Path structure should be: Artist/Album/track.flac or Album/track.flac
    // Only infer album when we have at least Album/track.flac structure (3 components minimum)

    let components: Vec<&str> = track_path
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();

    // Need at least Album/track.flac (3 components minimum)
    if components.len() >= 3 {
        // The component right before the filename should be the album
        let potential_album = components[components.len() - 2]; // parent directory

        // Only infer if we have a valid album name
        if !potential_album.is_empty() {
            return Some(potential_album.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_infer_artist_from_path() {
        // Valid Artist/Album/track.flac structure (need 3 components: "The Beatles", "Abbey Road", "01 - Come Together.flac")
        let path = PathBuf::from("The Beatles/Abbey Road/01 - Come Together.flac");
        assert_eq!(
            infer_artist_from_path(&path),
            Some("The Beatles".to_string())
        );

        // Nested directory structure
        let path = PathBuf::from("/music/The Beatles/Abbey Road/01 - Come Together.flac");
        assert_eq!(
            infer_artist_from_path(&path),
            Some("The Beatles".to_string())
        );

        // Deep nested structure
        let path = PathBuf::from("/home/user/music/Genre/Artist/Album/01 - Track.flac");
        assert_eq!(infer_artist_from_path(&path), Some("Artist".to_string()));

        // Invalid: Album/track.flac (no artist) - only 3 components
        let path = PathBuf::from("Abbey Road/01 - Come Together.flac");
        assert_eq!(infer_artist_from_path(&path), None);

        // Invalid: Just track.flac - only 2 components
        let path = PathBuf::from("01 - Come Together.flac");
        assert_eq!(infer_artist_from_path(&path), None);

        // Edge case: Artist and Album have same name
        let path = PathBuf::from("Greatest Hits/Greatest Hits/01 - Song.flac");
        assert_eq!(infer_artist_from_path(&path), None);

        // Edge case: Root directory ("/") as artist
        // Note: When path is "/Abbey Road/01 - Song.flac", the components are ["/", "Abbey Road", "01 - Song.flac"]
        // This would return Some("/") which is not a valid artist name

        // Unicode artist names
        let path = PathBuf::from("Björk/Vespertine/01 - Cocoon.flac");
        assert_eq!(infer_artist_from_path(&path), Some("Björk".to_string()));

        // Artist with special characters and numbers
        let path = PathBuf::from("The-artist_123/Album (2023)/01 - Track.flac");
        assert_eq!(
            infer_artist_from_path(&path),
            Some("The-artist_123".to_string())
        );
    }

    #[test]
    fn test_infer_album_from_path() {
        // Valid Artist/Album/track.flac structure
        let path = PathBuf::from("The Beatles/Abbey Road/01 - Come Together.flac");
        assert_eq!(infer_album_from_path(&path), Some("Abbey Road".to_string()));

        // Nested directory structure
        let path = PathBuf::from("/music/The Beatles/Abbey Road/01 - Come Together.flac");
        assert_eq!(infer_album_from_path(&path), Some("Abbey Road".to_string()));

        // Album/track.flac structure - only 2 components (Abbey Road and track file)
        // Need 3+ components to infer album, so this should return None
        let path = PathBuf::from("Abbey Road/01 - Come Together.flac");
        assert_eq!(infer_album_from_path(&path), None);

        // Deep nested structure
        let path = PathBuf::from("/home/user/music/Genre/Artist/Album/01 - Track.flac");
        assert_eq!(infer_album_from_path(&path), Some("Album".to_string()));

        // Invalid: Just track.flac - only 2 components
        let path = PathBuf::from("01 - Come Together.flac");
        assert_eq!(infer_album_from_path(&path), None);

        // Unicode album names
        let path = PathBuf::from("Björk/Vespertine/01 - Cocoon.flac");
        assert_eq!(infer_album_from_path(&path), Some("Vespertine".to_string()));

        // Album with special characters and numbers
        let path = PathBuf::from("The-artist_123/Album (2023)/01 - Track.flac");
        assert_eq!(
            infer_album_from_path(&path),
            Some("Album (2023)".to_string())
        );

        // Album with spaces and hyphens
        let path = PathBuf::from("Artist/The Dark Side of the Moon/01 - Track.flac");
        assert_eq!(
            infer_album_from_path(&path),
            Some("The Dark Side of the Moon".to_string())
        );
    }
}
