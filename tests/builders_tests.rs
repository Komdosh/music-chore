//! Tests for TrackMetadataBuilder functionality

use music_chore::core::builders::TrackMetadataBuilder;
use music_chore::core::domain::models::{MetadataSource, MetadataValue, TrackMetadata};
use std::path::PathBuf;

#[test]
fn test_track_metadata_builder_new() {
    let path = PathBuf::from("/test/path.flac");
    let result = TrackMetadataBuilder::new(&path).build();

    // Verify default values in built result
    assert!(result.title.is_none());
    assert!(result.artist.is_none());
    assert!(result.album.is_none());
    assert!(result.album_artist.is_none());
    assert!(result.track_number.is_none());
    assert!(result.disc_number.is_none());
    assert!(result.year.is_none());
    assert!(result.genre.is_none());
    assert!(result.duration.is_none());
    assert_eq!(result.format, "unknown");
    assert_eq!(result.path, path);
}

#[test]
fn test_track_metadata_builder_default() {
    let result = TrackMetadataBuilder::default().build();

    // Verify default path is empty
    assert_eq!(result.path, PathBuf::new());
    assert_eq!(result.format, "unknown");
}

#[test]
fn test_track_metadata_builder_title_embedded() {
    let path = PathBuf::from("/test.flac");
    let result = TrackMetadataBuilder::new(&path)
        .title("Test Title", MetadataSource::Embedded, 1.0)
        .build();

    assert!(result.title.is_some());
    assert_eq!(result.title.as_ref().unwrap().value, "Test Title");
    assert_eq!(
        result.title.as_ref().unwrap().source,
        MetadataSource::Embedded
    );
    assert_eq!(result.title.as_ref().unwrap().confidence, 1.0);
}

#[test]
fn test_track_metadata_builder_title_inferred() {
    let path = PathBuf::from("/test.flac");
    let result = TrackMetadataBuilder::new(&path)
        .title("Inferred Title", MetadataSource::FolderInferred, 0.3)
        .build();

    assert!(result.title.is_some());
    assert_eq!(result.title.as_ref().unwrap().value, "Inferred Title");
    assert_eq!(
        result.title.as_ref().unwrap().source,
        MetadataSource::FolderInferred
    );
    assert_eq!(result.title.as_ref().unwrap().confidence, 0.3);
}

#[test]
fn test_track_metadata_builder_title_user_set() {
    let path = PathBuf::from("/test.flac");
    let result = TrackMetadataBuilder::new(&path)
        .title("User Title", MetadataSource::UserEdited, 1.0)
        .build();

    assert!(result.title.is_some());
    assert_eq!(result.title.as_ref().unwrap().value, "User Title");
    assert_eq!(
        result.title.as_ref().unwrap().source,
        MetadataSource::UserEdited
    );
    assert_eq!(result.title.as_ref().unwrap().confidence, 1.0);
}

#[test]
fn test_track_metadata_builder_artist() {
    let path = PathBuf::from("/test.flac");
    let result = TrackMetadataBuilder::new(&path)
        .artist("Test Artist", MetadataSource::Embedded, 1.0)
        .build();

    assert!(result.artist.is_some());
    assert_eq!(result.artist.as_ref().unwrap().value, "Test Artist");
    assert_eq!(
        result.artist.as_ref().unwrap().source,
        MetadataSource::Embedded
    );
}

#[test]
fn test_track_metadata_builder_album() {
    let path = PathBuf::from("/test.flac");
    let result = TrackMetadataBuilder::new(&path)
        .album("Test Album", MetadataSource::Embedded, 1.0)
        .build();

    assert!(result.album.is_some());
    assert_eq!(result.album.as_ref().unwrap().value, "Test Album");
}

#[test]
fn test_track_metadata_builder_album_artist() {
    let path = PathBuf::from("/test.flac");
    let result = TrackMetadataBuilder::new(&path)
        .album_artist("Album Artist", MetadataSource::Embedded, 1.0)
        .build();

    assert!(result.album_artist.is_some());
    assert_eq!(result.album_artist.as_ref().unwrap().value, "Album Artist");
}

#[test]
fn test_track_metadata_builder_track_number() {
    let path = PathBuf::from("/test.flac");
    let result = TrackMetadataBuilder::new(&path)
        .track_number(5, MetadataSource::Embedded, 1.0)
        .build();

    assert!(result.track_number.is_some());
    assert_eq!(result.track_number.as_ref().unwrap().value, 5);
    assert_eq!(
        result.track_number.as_ref().unwrap().source,
        MetadataSource::Embedded
    );
}

#[test]
fn test_track_metadata_builder_disc_number() {
    let path = PathBuf::from("/test.flac");
    let result = TrackMetadataBuilder::new(&path)
        .disc_number(2, MetadataSource::Embedded, 1.0)
        .build();

    assert!(result.disc_number.is_some());
    assert_eq!(result.disc_number.as_ref().unwrap().value, 2);
}

#[test]
fn test_track_metadata_builder_year_boundary() {
    let path = PathBuf::from("/test.flac");

    // Test minimum boundary
    let result1 = TrackMetadataBuilder::new(&path)
        .year(1900, MetadataSource::Embedded, 1.0)
        .build();
    assert_eq!(result1.year.as_ref().unwrap().value, 1900);

    // Test maximum boundary
    let result2 = TrackMetadataBuilder::new(&path)
        .year(2100, MetadataSource::Embedded, 1.0)
        .build();
    assert_eq!(result2.year.as_ref().unwrap().value, 2100);
}

#[test]
fn test_track_metadata_builder_genre() {
    let path = PathBuf::from("/test.flac");
    let result = TrackMetadataBuilder::new(&path)
        .genre("Rock", MetadataSource::Embedded, 1.0)
        .build();

    assert!(result.genre.is_some());
    assert_eq!(result.genre.as_ref().unwrap().value, "Rock");
}

#[test]
fn test_track_metadata_builder_duration() {
    let path = PathBuf::from("/test.flac");
    let result = TrackMetadataBuilder::new(&path)
        .duration(180.5, MetadataSource::Embedded, 1.0)
        .build();

    assert!(result.duration.is_some());
    assert_eq!(result.duration.as_ref().unwrap().value, 180.5);
}

#[test]
fn test_track_metadata_builder_format() {
    let path = PathBuf::from("/test.flac");
    let result = TrackMetadataBuilder::new(&path).format("flac").build();

    assert_eq!(result.format, "flac");
}

#[test]
fn test_track_metadata_builder_build_full_metadata() {
    let path = PathBuf::from("/test/song.flac");
    let result = TrackMetadataBuilder::new(&path)
        .title("Test Song", MetadataSource::Embedded, 1.0)
        .artist("Test Artist", MetadataSource::Embedded, 1.0)
        .album("Test Album", MetadataSource::Embedded, 1.0)
        .album_artist("Album Artist", MetadataSource::Embedded, 1.0)
        .track_number(5, MetadataSource::Embedded, 1.0)
        .disc_number(1, MetadataSource::Embedded, 1.0)
        .year(2024, MetadataSource::Embedded, 1.0)
        .genre("Rock", MetadataSource::Embedded, 1.0)
        .duration(180.5, MetadataSource::Embedded, 1.0)
        .format("flac")
        .build();

    // Verify all fields
    assert_eq!(result.title.as_ref().unwrap().value, "Test Song");
    assert_eq!(result.artist.as_ref().unwrap().value, "Test Artist");
    assert_eq!(result.album.as_ref().unwrap().value, "Test Album");
    assert_eq!(result.album_artist.as_ref().unwrap().value, "Album Artist");
    assert_eq!(result.track_number.as_ref().unwrap().value, 5);
    assert_eq!(result.disc_number.as_ref().unwrap().value, 1);
    assert_eq!(result.year.as_ref().unwrap().value, 2024);
    assert_eq!(result.genre.as_ref().unwrap().value, "Rock");
    assert_eq!(result.duration.as_ref().unwrap().value, 180.5);
    assert_eq!(result.format, "flac");
    assert_eq!(result.path, path);
}

#[test]
fn test_track_metadata_builder_chaining() {
    let path = PathBuf::from("/test/song.flac");
    let result = TrackMetadataBuilder::new(&path)
        .title("Title", MetadataSource::Embedded, 1.0)
        .artist("Artist", MetadataSource::Embedded, 1.0)
        .album("Album", MetadataSource::Embedded, 1.0)
        .format("flac")
        .build();

    assert_eq!(result.title.as_ref().unwrap().value, "Title");
    assert_eq!(result.artist.as_ref().unwrap().value, "Artist");
    assert_eq!(result.album.as_ref().unwrap().value, "Album");
}

#[test]
fn test_track_metadata_builder_unicode_values() {
    let path = PathBuf::from("/test/song.flac");
    let result = TrackMetadataBuilder::new(&path)
        .title("日本語タイトル", MetadataSource::Embedded, 1.0)
        .artist("艺术家", MetadataSource::Embedded, 1.0)
        .album("Альбом", MetadataSource::Embedded, 1.0)
        .format("flac")
        .build();

    assert_eq!(result.title.as_ref().unwrap().value, "日本語タイトル");
    assert_eq!(result.artist.as_ref().unwrap().value, "艺术家");
    assert_eq!(result.album.as_ref().unwrap().value, "Альбом");
}

#[test]
fn test_track_metadata_builder_special_characters() {
    let path = PathBuf::from("/test/song.flac");
    let result = TrackMetadataBuilder::new(&path)
        .title(
            "Song & Artist (feat. Someone)",
            MetadataSource::Embedded,
            1.0,
        )
        .album("Album: Subtitle [Remix]", MetadataSource::Embedded, 1.0)
        .format("flac")
        .build();

    assert_eq!(
        result.title.as_ref().unwrap().value,
        "Song & Artist (feat. Someone)"
    );
    assert_eq!(
        result.album.as_ref().unwrap().value,
        "Album: Subtitle [Remix]"
    );
}
