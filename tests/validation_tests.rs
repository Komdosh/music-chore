//! Unit tests for validation functionality
//! Tests the validate_library_structure and format_validation_results functions

use music_chore::{
    mcp_server::{
        format_validation_results, validate_library_structure, ValidationIssue, ValidationResult,
    },
    AlbumNode, ArtistNode, Library, MetadataValue, TrackMetadata, TrackNode,
};
use std::path::PathBuf;

#[test]
fn test_validate_empty_library() {
    let library = Library {
        total_artists: 0,
        total_albums: 0,
        total_tracks: 0,
        artists: vec![],
    };

    let result = validate_library_structure(&library);

    assert_eq!(result.total_issues, 0);
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.warnings.len(), 0);
    assert_eq!(result.info.len(), 1); // Only the summary info
    assert!(result.info[0]
        .message
        .contains("0 artists, 0 albums, 0 tracks"));
}

#[test]
fn test_validate_perfect_library() {
    let library = Library {
        total_artists: 1,
        total_albums: 1,
        total_tracks: 2,
        artists: vec![ArtistNode {
            name: "Test Artist".to_string(),
            albums: vec![AlbumNode {
                title: "Test Album".to_string(),
                year: Some(2023),
                tracks: vec![
                    TrackNode {
                        file_path: PathBuf::from("/test/track1.flac"),
                        metadata: create_basic_metadata("Track 1", 1),
                    },
                    TrackNode {
                        file_path: PathBuf::from("/test/track2.flac"),
                        metadata: create_basic_metadata("Track 2", 2),
                    },
                ],
                path: PathBuf::from("/test/album"),
            }],
        }],
    };

    let result = validate_library_structure(&library);

    assert_eq!(result.total_issues, 0);
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.warnings.len(), 0);
    assert_eq!(result.info.len(), 1); // Only the summary info
}

#[test]
fn test_validate_missing_metadata() {
    let library = Library {
        total_artists: 1,
        total_albums: 1,
        total_tracks: 1,
        artists: vec![ArtistNode {
            name: "Test Artist".to_string(),
            albums: vec![AlbumNode {
                title: "Test Album".to_string(),
                year: None, // Missing year
                tracks: vec![TrackNode {
                    file_path: PathBuf::from("/test/track1.flac"),
                    metadata: TrackMetadata {
                        title: None,  // Missing title
                        artist: None, // Missing artist
                        album: None,  // Missing album
                        album_artist: None,
                        year: None,
                        track_number: None, // Missing track number
                        disc_number: None,
                        genre: None,
                        duration: None,
                        format: "flac".to_string(),
                        path: PathBuf::from("/test/track1.flac"),
                    },
                }],
                path: PathBuf::from("/test/album"),
            }],
        }],
    };

    let result = validate_library_structure(&library);

    assert_eq!(result.total_issues, 5); // 1 error + 4 warnings
    assert_eq!(result.errors.len(), 1); // Missing title
    assert_eq!(result.warnings.len(), 4); // Missing year, artist, album, track number
    assert_eq!(result.info.len(), 1);

    // Check specific errors
    assert_eq!(result.errors[0].category, "track");
    assert_eq!(result.errors[0].message, "Track missing title");

    // Check specific warnings
    let warning_categories: Vec<_> = result.warnings.iter().map(|w| &w.category).collect();
    assert!(warning_categories.contains(&&"album".to_string())); // Missing year
    assert!(warning_categories.contains(&&"track".to_string())); // Missing artist, album, track number
}

#[test]
fn test_validate_duplicate_track_numbers() {
    let library = Library {
        total_artists: 1,
        total_albums: 1,
        total_tracks: 2,
        artists: vec![ArtistNode {
            name: "Test Artist".to_string(),
            albums: vec![AlbumNode {
                title: "Test Album".to_string(),
                year: Some(2023),
                tracks: vec![
                    TrackNode {
                        file_path: PathBuf::from("/test/track1.flac"),
                        metadata: create_basic_metadata("Track 1", 1),
                    },
                    TrackNode {
                        file_path: PathBuf::from("/test/track2.flac"),
                        metadata: TrackMetadata {
                            title: Some(MetadataValue::embedded("Track 2".to_string())),
                            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
                            album: Some(MetadataValue::embedded("Test Album".to_string())),
                            album_artist: Some(MetadataValue::embedded("Test Artist".to_string())),
                            year: Some(MetadataValue::embedded(2023)),
                            track_number: Some(MetadataValue::embedded(1)), // Duplicate!
                            disc_number: None,
                            genre: Some(MetadataValue::embedded("Rock".to_string())),
                            duration: Some(MetadataValue::embedded(240.0)),
                            format: "flac".to_string(),
                            path: PathBuf::from("/test/track2.flac"),
                        },
                    },
                ],
                path: PathBuf::from("/test/album"),
            }],
        }],
    };

    let result = validate_library_structure(&library);

    assert_eq!(result.total_issues, 1);
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.warnings.len(), 1);
    assert_eq!(result.warnings[0].category, "track");
    assert_eq!(result.warnings[0].message, "Duplicate track number");
}

#[test]
fn test_validate_unusual_durations() {
    let library = Library {
        total_artists: 1,
        total_albums: 1,
        total_tracks: 3,
        artists: vec![ArtistNode {
            name: "Test Artist".to_string(),
            albums: vec![AlbumNode {
                title: "Test Album".to_string(),
                year: Some(2023),
                tracks: vec![
                    TrackNode {
                        file_path: PathBuf::from("/test/short.flac"),
                        metadata: TrackMetadata {
                            title: Some(MetadataValue::embedded("Short Track".to_string())),
                            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
                            album: Some(MetadataValue::embedded("Test Album".to_string())),
                            album_artist: Some(MetadataValue::embedded("Test Artist".to_string())),
                            year: Some(MetadataValue::embedded(2023)),
                            track_number: Some(MetadataValue::embedded(1)),
                            disc_number: None,
                            genre: Some(MetadataValue::embedded("Rock".to_string())),
                            duration: Some(MetadataValue::embedded(5.0)), // Too short
                            format: "flac".to_string(),
                            path: PathBuf::from("/test/short.flac"),
                        },
                    },
                    TrackNode {
                        file_path: PathBuf::from("/test/normal.flac"),
                        metadata: create_basic_metadata("Normal Track", 2),
                    },
                    TrackNode {
                        file_path: PathBuf::from("/test/long.flac"),
                        metadata: TrackMetadata {
                            title: Some(MetadataValue::embedded("Long Track".to_string())),
                            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
                            album: Some(MetadataValue::embedded("Test Album".to_string())),
                            album_artist: Some(MetadataValue::embedded("Test Artist".to_string())),
                            year: Some(MetadataValue::embedded(2023)),
                            track_number: Some(MetadataValue::embedded(3)),
                            disc_number: None,
                            genre: Some(MetadataValue::embedded("Rock".to_string())),
                            duration: Some(MetadataValue::embedded(4000.0)), // Too long
                            format: "flac".to_string(),
                            path: PathBuf::from("/test/long.flac"),
                        },
                    },
                ],
                path: PathBuf::from("/test/album"),
            }],
        }],
    };

    let result = validate_library_structure(&library);

    assert_eq!(result.total_issues, 2); // 2 warnings for unusual durations
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.warnings.len(), 2);

    let warning_messages: Vec<_> = result.warnings.iter().map(|w| &w.message).collect();
    assert!(warning_messages.contains(&&"Very short track (less than 10 seconds)".to_string()));
    assert!(warning_messages.contains(&&"Very long track (more than 1 hour)".to_string()));
}

#[test]
fn test_validate_empty_entities() {
    let library = Library {
        total_artists: 2,
        total_albums: 2,
        total_tracks: 0,
        artists: vec![
            ArtistNode {
                name: "".to_string(), // Empty artist name
                albums: vec![AlbumNode {
                    title: "Album with no tracks".to_string(),
                    year: Some(2023),
                    tracks: vec![], // Empty album
                    path: PathBuf::from("/test/empty_album"),
                }],
            },
            ArtistNode {
                name: "Artist with no albums".to_string(),
                albums: vec![], // No albums
            },
        ],
    };

    let result = validate_library_structure(&library);

    assert_eq!(result.total_issues, 3); // 2 errors + 1 warning
    assert_eq!(result.errors.len(), 2); // Empty artist name, empty album
    assert_eq!(result.warnings.len(), 1); // Artist with no albums

    let error_categories: Vec<_> = result.errors.iter().map(|e| &e.category).collect();
    assert!(error_categories.contains(&&"artist".to_string()));
    assert!(error_categories.contains(&&"album".to_string()));

    assert_eq!(result.warnings[0].category, "artist");
    assert_eq!(result.warnings[0].message, "Artist has no albums");
}

#[test]
fn test_format_validation_results_clean() {
    let result = ValidationResult {
        total_issues: 0,
        warnings: vec![],
        errors: vec![],
        info: vec![ValidationIssue {
            severity: "info".to_string(),
            category: "summary".to_string(),
            message: "Library validation complete: 1 artists, 1 albums, 2 tracks".to_string(),
            details: None,
            file_path: None,
        }],
    };

    let formatted = format_validation_results(&result);

    assert!(formatted.contains("=== MUSIC LIBRARY VALIDATION ==="));
    assert!(formatted.contains("Total Issues Found: 0"));
    assert!(formatted.contains("ℹ️  INFO (1):"));
    assert!(formatted.contains("Library validation complete: 1 artists, 1 albums, 2 tracks"));
    assert!(formatted.contains("=== END VALIDATION ==="));
    assert!(!formatted.contains("🔴 ERRORS"));
    assert!(!formatted.contains("🟡 WARNINGS"));
}

#[test]
fn test_format_validation_results_with_issues() {
    let result = ValidationResult {
        total_issues: 3,
        warnings: vec![ValidationIssue {
            severity: "warning".to_string(),
            category: "track".to_string(),
            message: "Track missing track number".to_string(),
            details: Some(
                "Artist: Test Artist, Album: Test Album, File: /test/track.flac".to_string(),
            ),
            file_path: Some("/test/track.flac".to_string()),
        }],
        errors: vec![
            ValidationIssue {
                severity: "error".to_string(),
                category: "track".to_string(),
                message: "Track missing title".to_string(),
                details: Some("Artist: Test Artist, Album: Test Album".to_string()),
                file_path: Some("/test/track.flac".to_string()),
            },
            ValidationIssue {
                severity: "error".to_string(),
                category: "album".to_string(),
                message: "Album title is empty".to_string(),
                details: Some("Artist: Test Artist".to_string()),
                file_path: None,
            },
        ],
        info: vec![ValidationIssue {
            severity: "info".to_string(),
            category: "summary".to_string(),
            message: "Library validation complete: 1 artists, 1 albums, 1 tracks".to_string(),
            details: None,
            file_path: None,
        }],
    };

    let formatted = format_validation_results(&result);

    assert!(formatted.contains("=== MUSIC LIBRARY VALIDATION ==="));
    assert!(formatted.contains("Total Issues Found: 3"));
    assert!(formatted.contains("🔴 ERRORS (2):"));
    assert!(formatted.contains("🟡 WARNINGS (1):"));
    assert!(formatted.contains("ℹ️  INFO (1):"));
    assert!(formatted.contains("[TRACK] Track missing title"));
    assert!(formatted.contains("[ALBUM] Album title is empty"));
    assert!(formatted.contains("[TRACK] Track missing track number"));
    assert!(formatted.contains("File: /test/track.flac"));
    assert!(formatted.contains("=== END VALIDATION ==="));
}

#[test]
fn test_validate_duplicate_albums() {
    let library = Library {
        total_artists: 1,
        total_albums: 2,
        total_tracks: 2,
        artists: vec![ArtistNode {
            name: "Test Artist".to_string(),
            albums: vec![
                AlbumNode {
                    title: "Same Album".to_string(),
                    year: Some(2023),
                    tracks: vec![TrackNode {
                        file_path: PathBuf::from("/test/album1_track1.flac"),
                        metadata: create_basic_metadata("Track 1", 1),
                    }],
                    path: PathBuf::from("/test/album1"),
                },
                AlbumNode {
                    title: "Same Album".to_string(), // Duplicate title (case insensitive check)
                    year: Some(2024),
                    tracks: vec![TrackNode {
                        file_path: PathBuf::from("/test/album2_track1.flac"),
                        metadata: create_basic_metadata("Track 1", 1),
                    }],
                    path: PathBuf::from("/test/album2"),
                },
            ],
        }],
    };

    let result = validate_library_structure(&library);

    assert_eq!(result.total_issues, 1);
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.warnings.len(), 1);
    assert_eq!(result.warnings[0].category, "album");
    assert_eq!(result.warnings[0].message, "Duplicate album title found");
}

// Helper function to create basic metadata for testing
fn create_basic_metadata(title: &str, track_number: u32) -> TrackMetadata {
    TrackMetadata {
        title: Some(MetadataValue::embedded(title.to_string())),
        artist: Some(MetadataValue::embedded("Test Artist".to_string())),
        album: Some(MetadataValue::embedded("Test Album".to_string())),
        album_artist: Some(MetadataValue::embedded("Test Artist".to_string())),
        year: Some(MetadataValue::embedded(2023)),
        track_number: Some(MetadataValue::embedded(track_number)),
        disc_number: None,
        genre: Some(MetadataValue::embedded("Rock".to_string())),
        duration: Some(MetadataValue::embedded(180.0)),
        format: "flac".to_string(),
        path: PathBuf::from("/test"),
    }
}
