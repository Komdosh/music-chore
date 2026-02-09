//! Tests for type-safe newtype wrappers

use music_chore::core::types::{
    AlbumName, ArtistName, Confidence, DiscNumber, Duration, FilePath, MetadataOperationMode,
    TrackNumber, TrackTitle, Year,
};
use std::path::PathBuf;

#[test]
fn test_track_title_from_string() {
    let title = TrackTitle("Test Title".to_string());
    assert_eq!(title.0, "Test Title");
}

#[test]
fn test_track_title_from_str() {
    let title = TrackTitle::from("Test Title");
    assert_eq!(title.0, "Test Title");
}

#[test]
fn test_track_title_as_ref() {
    let title = TrackTitle::from("Test Title".to_string());
    assert_eq!(title.as_ref(), "Test Title");
}

#[test]
fn test_artist_name_from_string() {
    let artist = ArtistName("Artist Name".to_string());
    assert_eq!(artist.0, "Artist Name");
}

#[test]
fn test_artist_name_from_str() {
    let artist = ArtistName::from("Artist Name");
    assert_eq!(artist.0, "Artist Name");
}

#[test]
fn test_artist_name_as_ref() {
    let artist = ArtistName::from("Artist Name".to_string());
    assert_eq!(artist.as_ref(), "Artist Name");
}

#[test]
fn test_album_name_from_string() {
    let album = AlbumName("Album Name".to_string());
    assert_eq!(album.0, "Album Name");
}

#[test]
fn test_album_name_from_str() {
    let album = AlbumName::from("Album Name");
    assert_eq!(album.0, "Album Name");
}

#[test]
fn test_album_name_as_ref() {
    let album = AlbumName::from("Album Name".to_string());
    assert_eq!(album.as_ref(), "Album Name");
}

#[test]
fn test_file_path_from_pathbuf() {
    let path = PathBuf::from("/test/path.flac");
    let file_path = FilePath::from(path.clone());
    assert_eq!(file_path.0, path);
}

#[test]
fn test_file_path_from_pathbuf_ref() {
    let path = PathBuf::from("/test/path.flac");
    let file_path = FilePath::from(&path);
    assert_eq!(file_path.0, path);
}

#[test]
fn test_file_path_as_ref() {
    let path = PathBuf::from("/test/path.flac");
    let file_path = FilePath::from(path.clone());
    assert_eq!(file_path.as_ref(), &path);
}

#[test]
fn test_track_number_from_u32() {
    let number = TrackNumber::from(5u32);
    assert_eq!(number.0, 5);
}

#[test]
fn test_track_number_from_u32_ref() {
    let number = TrackNumber::from(&7u32);
    assert_eq!(number.0, 7);
}

#[test]
fn test_track_number_as_ref() {
    let number = TrackNumber::from(10u32);
    assert_eq!(number.as_ref(), &10);
}

#[test]
fn test_track_number_ord() {
    assert!(TrackNumber(1) < TrackNumber(2));
    assert!(TrackNumber(2) > TrackNumber(1));
    assert!(TrackNumber(5) == TrackNumber(5));
}

#[test]
fn test_track_number_boundary_values() {
    let min = TrackNumber(0u32);
    assert_eq!(min.0, 0);

    let max = TrackNumber(u32::MAX);
    assert_eq!(max.0, u32::MAX);
}

#[test]
fn test_disc_number_from_u32() {
    let number = DiscNumber::from(2u32);
    assert_eq!(number.0, 2);
}

#[test]
fn test_disc_number_from_u32_ref() {
    let number = DiscNumber::from(&3u32);
    assert_eq!(number.0, 3);
}

#[test]
fn test_disc_number_as_ref() {
    let number = DiscNumber::from(1u32);
    assert_eq!(number.as_ref(), &1);
}

#[test]
fn test_disc_number_ord() {
    assert!(DiscNumber(1) < DiscNumber(2));
    assert!(DiscNumber(2) > DiscNumber(1));
    assert!(DiscNumber(1) == DiscNumber(1));
}

#[test]
fn test_year_from_u32() {
    let year = Year::from(2024u32);
    assert_eq!(year.0, 2024);
}

#[test]
fn test_year_from_u32_ref() {
    let year = Year::from(&2023u32);
    assert_eq!(year.0, 2023);
}

#[test]
fn test_year_as_ref() {
    let year = Year::from(2020u32);
    assert_eq!(year.as_ref(), &2020);
}

#[test]
fn test_year_ord() {
    assert!(Year(2020) < Year(2024));
    assert!(Year(2024) > Year(2020));
    assert!(Year(1900) == Year(1900));
}

#[test]
fn test_year_boundary_values() {
    let min = Year(1900u32);
    assert_eq!(min.0, 1900);

    let max = Year(2100u32);
    assert_eq!(max.0, 2100);
}

#[test]
fn test_duration_from_f64() {
    let duration = Duration::from(180.5);
    assert_eq!(duration.0, 180.5);
}

#[test]
fn test_duration_from_f64_ref() {
    let duration = Duration::from(&200.0);
    assert_eq!(duration.0, 200.0);
}

#[test]
fn test_duration_as_ref() {
    let duration = Duration::from(150.0);
    assert_eq!(duration.as_ref(), &150.0);
}

#[test]
fn test_duration_edge_values() {
    let zero = Duration::from(0.0);
    assert_eq!(zero.0, 0.0);

    let very_long = Duration::from(9999.999);
    assert_eq!(very_long.0, 9999.999);
}

#[test]
fn test_confidence_from_f32() {
    let confidence = Confidence::from(0.75);
    assert_eq!(confidence.0, 0.75);
}

#[test]
fn test_confidence_from_f32_ref() {
    let confidence = Confidence::from(&0.5);
    assert_eq!(confidence.0, 0.5);
}

#[test]
fn test_confidence_as_ref() {
    let confidence = Confidence::from(0.9);
    assert_eq!(confidence.as_ref(), &0.9);
}

#[test]
fn test_confidence_boundary_values() {
    let min = Confidence::from(0.0);
    assert_eq!(min.0, 0.0);

    let max = Confidence::from(1.0);
    assert_eq!(max.0, 1.0);
}

#[test]
fn test_confidence_partial_ord() {
    assert!(Confidence(0.5) < Confidence(0.75));
    assert!(Confidence(0.75) > Confidence(0.5));
    assert!(Confidence(0.5) == Confidence(0.5));
}

#[test]
fn test_metadata_operation_mode_apply() {
    let mode = MetadataOperationMode::Apply;
    assert!(matches!(mode, MetadataOperationMode::Apply));
}

#[test]
fn test_metadata_operation_mode_dry_run() {
    let mode = MetadataOperationMode::DryRun;
    assert!(matches!(mode, MetadataOperationMode::DryRun));
}

#[test]
fn test_metadata_operation_mode_validate() {
    let mode = MetadataOperationMode::Validate;
    assert!(matches!(mode, MetadataOperationMode::Validate));
}

#[test]
fn test_metadata_operation_mode_equality() {
    let apply = MetadataOperationMode::Apply;
    let dry_run = MetadataOperationMode::DryRun;

    assert_ne!(apply, dry_run);
    assert_eq!(apply, MetadataOperationMode::Apply);
    assert_eq!(dry_run, MetadataOperationMode::DryRun);
}
