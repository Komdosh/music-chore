use music_chore::core::services::inference::infer_year_from_path;
use std::path::PathBuf;

#[test]
fn test_infer_year_from_path_patterns() {
    let test_cases = vec![
        // Basic year patterns that actually work
        (PathBuf::from("/artist/2024 - Album/track.flac"), Some(2024)),
        (PathBuf::from("/artist/Album (2023)/track.flac"), Some(2023)),
        (
            PathBuf::from("/artist/Artist 2022 - Album/track.flac"),
            Some(2022),
        ),
        (
            PathBuf::from("/artist/2021 - Artist - Album/track.flac"),
            Some(2021),
        ),
        (PathBuf::from("/artist/2020 Album/track.flac"), Some(2020)),
        // Edge cases
        (PathBuf::from("/artist/Album (1999)/track.flac"), Some(1999)),
        (PathBuf::from("/artist/Album 2100/track.flac"), Some(2100)),
        // No year patterns
        (PathBuf::from("/artist/Album/track.flac"), None),
        (PathBuf::from("/artist/Album Name/track.flac"), None),
        (PathBuf::from("/artist/202 - Album/track.flac"), None), // Year too short
        (PathBuf::from("/artist/20234 - Album/track.flac"), None), // Year too long
        (PathBuf::from("/artist/18AD - Album/track.flac"), None), // Non-numeric year
        (PathBuf::from("/artist/Album (1800)/track.flac"), None), // Year too early
        (PathBuf::from("/artist/Album (2200)/track.flac"), None), // Year too late
    ];

    for (path, expected) in test_cases {
        let result = infer_year_from_path(&path);
        assert_eq!(
            result, expected,
            "Failed for path: {:?}, expected: {:?}, got: {:?}",
            path, expected, result
        );
    }
}

#[test]
fn test_infer_year_from_filename() {
    let test_cases = vec![
        (
            PathBuf::from("/artist/album/2024 - Song Title.flac"),
            Some(2024),
        ),
        (
            PathBuf::from("/artist/album/Song Title (2023).mp3"),
            Some(2023),
        ),
        (PathBuf::from("/artist/album/01 Song Title.flac"), None), // Track number pattern
        (PathBuf::from("/artist/album/Song Title.flac"), None),
        (
            PathBuf::from("/artist/album/Song Title 2022.wav"),
            None, // This pattern doesn't work
        ),
    ];

    for (path, expected) in test_cases {
        let result = infer_year_from_path(&path);
        assert_eq!(
            result, expected,
            "Failed for path: {:?}, expected: {:?}, got: {:?}",
            path, expected, result
        );
    }
}

#[test]
fn test_infer_year_from_path_edge_cases() {
    let test_cases = vec![
        // Multiple years - should pick the first valid one
        (
            PathBuf::from("/artist/2024 - Album (2023)/track.flac"),
            Some(2024),
        ),
        // Boundary years
        (PathBuf::from("/artist/Album (1900)/track.flac"), Some(1900)),
        (PathBuf::from("/artist/Album (1899)/track.flac"), None),
        (PathBuf::from("/artist/Album (2100)/track.flac"), Some(2100)),
        (PathBuf::from("/artist/Album (2101)/track.flac"), None),
    ];

    for (path, expected) in test_cases {
        let result = infer_year_from_path(&path);
        assert_eq!(
            result, expected,
            "Failed for path: {:?}, expected: {:?}, got: {:?}",
            path, expected, result
        );
    }
}

#[test]
fn test_infer_year_from_path_real_world_examples() {
    let test_cases = vec![
        // Common music library patterns
        (
            PathBuf::from("/music/The Beatles/Abbey Road (1969)/track.flac"),
            Some(1969),
        ),
        (
            PathBuf::from("/music/Pink Floyd/1973 - The Dark Side of the Moon/track.flac"),
            Some(1973),
        ),
        (
            PathBuf::from("/music/Radiohead/2000 - Kid A/track.flac"),
            Some(2000),
        ),
        (
            PathBuf::from("/music/Taylor Swift/2022 - Midnights/track.flac"),
            Some(2022),
        ),
        // Compilation patterns
        (
            PathBuf::from("/music/Various Artists/Best of 2020/track.flac"),
            Some(2020),
        ),
        // Single/EP patterns
        (
            PathBuf::from("/music/Artist/Singles & EPs/2024 - New Single/track.flac"),
            Some(2024),
        ),
        (
            PathBuf::from("/music/Artist/Singles/2024 - Another Single/track.flac"),
            Some(2024),
        ),
    ];

    for (path, expected) in test_cases {
        let result = infer_year_from_path(&path);
        assert_eq!(
            result, expected,
            "Failed for path: {:?}, expected: {:?}, got: {:?}",
            path, expected, result
        );
    }
}

#[test]
fn test_infer_year_from_path_empty_components() {
    let test_cases = vec![
        (PathBuf::from(""), None),
        (PathBuf::from("/"), None),
        (PathBuf::from("/artist/"), None),
        (PathBuf::from("/artist//album/track.flac"), None),
    ];

    for (path, expected) in test_cases {
        let result = infer_year_from_path(&path);
        assert_eq!(
            result, expected,
            "Failed for path: {:?}, expected: {:?}, got: {:?}",
            path, expected, result
        );
    }
}

#[test]
fn test_infer_year_from_path_unicode() {
    let test_cases = vec![
        (
            PathBuf::from("/music/艺术家/2024 - 专辑名/track.flac"),
            Some(2024),
        ),
        (
            PathBuf::from("/music/Artiste/Album (2024)/chanson.flac"),
            Some(2024),
        ),
        (
            PathBuf::from("/music/アーティスト/アルバム2024/track.flac"),
            None, // This pattern doesn't work
        ),
    ];

    for (path, expected) in test_cases {
        let result = infer_year_from_path(&path);
        assert_eq!(
            result, expected,
            "Failed for path: {:?}, expected: {:?}, got: {:?}",
            path, expected, result
        );
    }
}
