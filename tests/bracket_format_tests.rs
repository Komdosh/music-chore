//! Tests for bracket folder patterns

use music_chore::core::services::inference::{infer_album_from_path, infer_artist_from_path};
use std::path::PathBuf;

#[test]
fn test_bracket_format_in_folder_name() {
    // Folder name with bracket format indicator: "Artist [FLAC]"
    let path = PathBuf::from("/music/Some guy [FLAC]/track.flac");

    let album = infer_album_from_path(&path);
    let artist = infer_artist_from_path(&path);

    println!("Album: {:?}", album);
    println!("Artist: {:?}", artist);

    // The [FLAC] should be stripped from the album name
    assert_eq!(
        album,
        Some("Some guy".to_string()),
        "Album should be 'Some guy' without [FLAC] suffix"
    );

    // Artist should be inferred from parent folder or filename
    assert!(artist.is_some());
}

#[test]
fn test_bracket_format_with_separator() {
    // Folder name with separator and bracket: "Artist - Album [FLAC]"
    let path = PathBuf::from("/music/Artist Name - Album Title [FLAC]/track.flac");

    let album = infer_album_from_path(&path);
    let artist = infer_artist_from_path(&path);

    println!("Album: {:?}", album);
    println!("Artist: {:?}", artist);

    assert_eq!(album, Some("Album Title".to_string()));
    assert_eq!(artist, Some("Artist Name".to_string()));
}

#[test]
fn test_multiple_bracket_patterns() {
    // Test various bracket patterns
    let test_cases = vec![
        ("Artist [FLAC]", "Artist"),
        ("Artist [MP3]", "Artist"),
        ("Artist [WAV]", "Artist"),
        ("Album Title [FLAC]", "Album Title"),
    ];

    for (input, expected) in test_cases {
        let path = PathBuf::from(format!("/music/{}/track.flac", input));
        let album = infer_album_from_path(&path);
        assert_eq!(
            album,
            Some(expected.to_string()),
            "Failed for input: {}",
            input
        );
    }
}
