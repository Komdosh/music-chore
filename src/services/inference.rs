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

        // Invalid: Album/track.flac (no artist) - only 3 components
        let path = PathBuf::from("Abbey Road/01 - Come Together.flac");
        assert_eq!(infer_artist_from_path(&path), None);

        // Invalid: Just track.flac - only 2 components
        let path = PathBuf::from("01 - Come Together.flac");
        assert_eq!(infer_artist_from_path(&path), None);

        // Edge case: Artist and Album have same name
        let path = PathBuf::from("Greatest Hits/Greatest Hits/01 - Song.flac");
        assert_eq!(infer_artist_from_path(&path), None);
    }
}
