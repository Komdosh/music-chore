//! M4A format implementation of the AudioFile trait.

use lofty::{
    config::WriteOptions,
    file::{AudioFile as LoftyAudioFile, TaggedFile, TaggedFileExt},
    prelude::ItemKey,
    read_from_path,
    tag::{ItemValue, TagItem},
};
use std::path::Path;

use crate::adapters::audio_formats::wav::item_value_text;
use crate::core::domain::models::{
    FOLDER_INFERRED_CONFIDENCE, MetadataValue, Track, TrackMetadata,
};
use crate::core::domain::traits::{AudioFile, AudioFileError};
use crate::core::services::inference::{infer_album_from_path, infer_artist_from_path};

/// M4A format handler
pub struct M4aHandler;

impl M4aHandler {
    /// Create a new M4A handler
    pub fn new() -> Self {
        Self
    }
}

impl Default for M4aHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioFile for M4aHandler {
    fn can_handle(&self, path: &Path) -> bool {
        path.extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("m4a"))
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["m4a"]
    }

    fn read_metadata(&self, path: &Path) -> Result<Track, AudioFileError> {
        if !self.can_handle(path) {
            return Err(AudioFileError::UnsupportedFormat);
        }

        let tagged_file = read_from_path(path)
            .map_err(|e| AudioFileError::InvalidFile(format!("Failed to read M4A file: {}", e)))?;

        let metadata = self.extract_metadata_from_tags(&tagged_file, path);
        Ok(Track::new(path.to_path_buf(), metadata))
    }

    fn write_metadata(&self, path: &Path, metadata: &TrackMetadata) -> Result<(), AudioFileError> {
        if !self.can_handle(path) {
            return Err(AudioFileError::UnsupportedFormat);
        }

        let mut tagged_file = read_from_path(path)
            .map_err(|e| AudioFileError::InvalidFile(format!("Failed to read M4A file: {}", e)))?;

        // M4A (MP4) uses MP4 tags as the primary tag representation
        let tag = tagged_file
            .primary_tag_mut()
            .ok_or_else(|| AudioFileError::WriteError("M4A file has no primary tag".to_string()))?;

        let mut set_tag = |key: ItemKey, value: &str| {
            tag.insert(TagItem::new(key, ItemValue::Text(value.to_string())));
        };

        if let Some(ref title) = metadata.title {
            set_tag(ItemKey::TrackTitle, &title.value);
        }
        if let Some(ref artist) = metadata.artist {
            set_tag(ItemKey::TrackArtist, &artist.value);
        }
        if let Some(ref album) = metadata.album {
            set_tag(ItemKey::AlbumTitle, &album.value);
        }
        if let Some(ref album_artist) = metadata.album_artist {
            set_tag(ItemKey::AlbumArtist, &album_artist.value);
        }
        if let Some(ref track_number) = metadata.track_number {
            set_tag(ItemKey::TrackNumber, &track_number.value.to_string());
        }
        if let Some(ref disc_number) = metadata.disc_number {
            set_tag(ItemKey::DiscNumber, &disc_number.value.to_string());
        }
        if let Some(ref year) = metadata.year {
            set_tag(ItemKey::Year, &year.value.to_string());
        }
        if let Some(ref genre) = metadata.genre {
            set_tag(ItemKey::Genre, &genre.value);
        }

        tagged_file
            .save_to_path(path, WriteOptions::default())
            .map_err(|e| AudioFileError::WriteError(format!("Failed to save M4A file: {}", e)))?;

        Ok(())
    }

    fn read_basic_info(&self, path: &Path) -> Result<TrackMetadata, AudioFileError> {
        if !self.can_handle(path) {
            return Err(AudioFileError::UnsupportedFormat);
        }

        let tagged_file = read_from_path(path)
            .map_err(|e| AudioFileError::InvalidFile(format!("Failed to read M4A file: {}", e)))?;

        Ok(self.extract_basic_metadata(&tagged_file, path))
    }
}

impl M4aHandler {
    /// Extract metadata from lofty TaggedFile and convert to our TrackMetadata
    fn extract_metadata_from_tags(&self, tagged_file: &TaggedFile, path: &Path) -> TrackMetadata {
        let mut title = None;
        let mut artist = None;
        let mut album = None;
        let mut album_artist = None;
        let mut track_number = None;
        let mut disc_number = None;
        let mut year = None;
        let mut genre = None;

        if let Some(tag) = tagged_file.primary_tag() {
            for tag_item in tag.items() {
                let item_value_str = item_value_text(tag_item);
                match tag_item.key() {
                    ItemKey::TrackTitle => title = Some(MetadataValue::embedded(item_value_str)),
                    ItemKey::TrackArtist => artist = Some(MetadataValue::embedded(item_value_str)),
                    ItemKey::AlbumTitle => album = Some(MetadataValue::embedded(item_value_str)),
                    ItemKey::AlbumArtist => {
                        album_artist = Some(MetadataValue::embedded(item_value_str))
                    }
                    ItemKey::TrackNumber => {
                        if let Ok(num) = item_value_str.parse::<u32>() {
                            track_number = Some(MetadataValue::embedded(num));
                        }
                    }
                    ItemKey::DiscNumber => {
                        if let Ok(num) = item_value_str.parse::<u32>() {
                            disc_number = Some(MetadataValue::embedded(num));
                        }
                    }
                    ItemKey::Year | ItemKey::RecordingDate => {
                        let clean_value = item_value_str.trim();
                        if let Ok(year_val) = clean_value.parse::<u32>() {
                            year = Some(MetadataValue::embedded(year_val));
                        }
                    }
                    ItemKey::Genre => genre = Some(MetadataValue::embedded(item_value_str)),
                    _ => {}
                }
            }
        }

        let duration = Some(MetadataValue::embedded(
            tagged_file.properties().duration().as_secs_f64(),
        ));

        let inferred_artist = if artist.is_none() {
            infer_artist_from_path(path)
                .map(|a| MetadataValue::inferred(a, FOLDER_INFERRED_CONFIDENCE))
        } else {
            artist
        };

        let inferred_album = if album.is_none() {
            infer_album_from_path(path)
                .map(|a| MetadataValue::inferred(a, FOLDER_INFERRED_CONFIDENCE))
        } else {
            album
        };

        TrackMetadata {
            title,
            artist: inferred_artist,
            album: inferred_album,
            album_artist,
            track_number,
            disc_number,
            year,
            genre,
            duration,
            format: "m4a".to_string(),
            path: path.to_path_buf(),
        }
    }

    /// Extract basic metadata (minimal parsing for performance)
    fn extract_basic_metadata(&self, tagged_file: &TaggedFile, path: &Path) -> TrackMetadata {
        let duration = Some(MetadataValue::embedded(
            tagged_file.properties().duration().as_secs_f64(),
        ));

        let inferred_artist = infer_artist_from_path(path)
            .map(|artist| MetadataValue::inferred(artist, FOLDER_INFERRED_CONFIDENCE));
        let inferred_album = infer_album_from_path(path)
            .map(|album| MetadataValue::inferred(album, FOLDER_INFERRED_CONFIDENCE));

        TrackMetadata {
            title: None,
            artist: inferred_artist,
            album: inferred_album,
            album_artist: None,
            track_number: None,
            disc_number: None,
            year: None,
            genre: None,
            duration,
            format: "m4a".to_string(),
            path: path.to_path_buf(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lofty::{
        file::FileType,
        properties::FileProperties,
        tag::{Tag, TagType},
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::Duration;
    use tempfile::TempDir;

    fn make_tagged_file(duration_secs: f64, items: Vec<TagItem>) -> TaggedFile {
        let mut tag = Tag::new(TagType::Mp4Ilst);
        for item in items {
            tag.insert(item);
        }

        let properties = FileProperties::new(
            Duration::from_secs_f64(duration_secs),
            None,
            None,
            None,
            None,
            None,
            None,
        );

        TaggedFile::new(FileType::Mp4, properties, vec![tag])
    }

    fn make_tagged_file_without_tags(duration_secs: f64) -> TaggedFile {
        let properties = FileProperties::new(
            Duration::from_secs_f64(duration_secs),
            None,
            None,
            None,
            None,
            None,
            None,
        );

        TaggedFile::new(FileType::Mp4, properties, vec![])
    }

    #[test]
    fn test_m4a_handler_supported_extensions() {
        let handler = M4aHandler::new();
        let extensions = handler.supported_extensions();
        assert_eq!(extensions, vec!["m4a"]);
    }

    #[test]
    fn test_m4a_handler_new_creates_instance() {
        let handler = M4aHandler::new();
        assert!(handler.can_handle(&PathBuf::from("test.m4a")));
    }

    #[test]
    fn test_m4a_handler_default_creates_instance() {
        let handler = M4aHandler::default();
        assert!(handler.can_handle(&PathBuf::from("test.m4a")));
    }

    #[test]
    fn test_m4a_handler_can_handle() {
        let handler = M4aHandler::new();

        assert!(handler.can_handle(&PathBuf::from("test.m4a")));
        assert!(handler.can_handle(&PathBuf::from("test.M4A")));
        assert!(!handler.can_handle(&PathBuf::from("test.flac")));
        assert!(!handler.can_handle(&PathBuf::from("test.mp3")));
    }

    #[test]
    fn test_m4a_handler_read_metadata_unsupported_format() {
        let handler = M4aHandler::new();
        let result = handler.read_metadata(&PathBuf::from("test.flac"));
        assert!(matches!(result, Err(AudioFileError::UnsupportedFormat)));
    }

    #[test]
    fn test_m4a_handler_read_basic_info_unsupported_format() {
        let handler = M4aHandler::new();
        let result = handler.read_basic_info(&PathBuf::from("test.flac"));
        assert!(matches!(result, Err(AudioFileError::UnsupportedFormat)));
    }

    #[test]
    fn test_m4a_handler_write_metadata_unsupported_format() {
        let handler = M4aHandler::new();
        let metadata = TrackMetadata {
            title: None,
            artist: None,
            album: None,
            album_artist: None,
            track_number: None,
            disc_number: None,
            year: None,
            genre: None,
            duration: None,
            format: "m4a".to_string(),
            path: PathBuf::from("test.m4a"),
        };
        let result = handler.write_metadata(&PathBuf::from("test.flac"), &metadata);
        assert!(matches!(result, Err(AudioFileError::UnsupportedFormat)));
    }

    #[test]
    fn test_m4a_handler_read_metadata_nonexistent_file() {
        let handler = M4aHandler::new();
        let result = handler.read_metadata(&PathBuf::from("nonexistent.m4a"));
        assert!(matches!(result, Err(AudioFileError::InvalidFile(_))));
    }

    #[test]
    fn test_m4a_handler_read_basic_info_nonexistent_file() {
        let handler = M4aHandler::new();
        let result = handler.read_basic_info(&PathBuf::from("nonexistent.m4a"));
        assert!(matches!(result, Err(AudioFileError::InvalidFile(_))));
    }

    #[test]
    fn test_m4a_handler_write_metadata_invalid_file_returns_invalid_file_error() {
        let handler = M4aHandler::new();
        let temp_dir = TempDir::new().expect("temp dir should be created");
        let m4a_path = temp_dir.path().join("bad.m4a");
        fs::write(&m4a_path, "not a real m4a file").expect("test file should be written");

        let metadata = TrackMetadata {
            title: Some(MetadataValue::embedded("Title".to_string())),
            artist: None,
            album: None,
            album_artist: None,
            track_number: None,
            disc_number: None,
            year: None,
            genre: None,
            duration: None,
            format: "m4a".to_string(),
            path: m4a_path.clone(),
        };

        let result = handler.write_metadata(&m4a_path, &metadata);
        assert!(matches!(result, Err(AudioFileError::InvalidFile(_))));
    }

    #[test]
    fn test_extract_metadata_from_tags_maps_all_supported_fields() {
        let handler = M4aHandler::new();
        let path = PathBuf::from("Music/Embedded Artist/Embedded Album/track.m4a");
        let tagged_file = make_tagged_file(
            245.5,
            vec![
                TagItem::new(
                    ItemKey::TrackTitle,
                    ItemValue::Text("Track Name".to_string()),
                ),
                TagItem::new(
                    ItemKey::TrackArtist,
                    ItemValue::Text("Artist Name".to_string()),
                ),
                TagItem::new(
                    ItemKey::AlbumTitle,
                    ItemValue::Text("Album Name".to_string()),
                ),
                TagItem::new(
                    ItemKey::AlbumArtist,
                    ItemValue::Text("Album Artist".to_string()),
                ),
                TagItem::new(ItemKey::TrackNumber, ItemValue::Text("7".to_string())),
                TagItem::new(ItemKey::DiscNumber, ItemValue::Text("2".to_string())),
                TagItem::new(ItemKey::RecordingDate, ItemValue::Text("1999".to_string())),
                TagItem::new(ItemKey::Genre, ItemValue::Text("Trip-Hop".to_string())),
            ],
        );

        let metadata = handler.extract_metadata_from_tags(&tagged_file, &path);

        assert_eq!(
            metadata.title.as_ref().map(|v| v.value.as_str()),
            Some("Track Name")
        );
        assert_eq!(
            metadata.artist.as_ref().map(|v| v.value.as_str()),
            Some("Artist Name")
        );
        assert_eq!(
            metadata.album.as_ref().map(|v| v.value.as_str()),
            Some("Album Name")
        );
        assert_eq!(
            metadata.album_artist.as_ref().map(|v| v.value.as_str()),
            Some("Album Artist")
        );
        assert_eq!(metadata.track_number.as_ref().map(|v| v.value), Some(7));
        assert_eq!(metadata.disc_number.as_ref().map(|v| v.value), Some(2));
        assert_eq!(metadata.year.as_ref().map(|v| v.value), Some(1999));
        assert_eq!(
            metadata.genre.as_ref().map(|v| v.value.as_str()),
            Some("Trip-Hop")
        );
        assert_eq!(metadata.duration.as_ref().map(|v| v.value), Some(245.5));
        assert_eq!(metadata.format, "m4a");
        assert_eq!(metadata.path, path);
    }

    #[test]
    fn test_extract_metadata_from_tags_uses_recording_date_and_folder_fallbacks() {
        let handler = M4aHandler::new();
        let path = PathBuf::from("Library/Fallback Artist/Fallback Album/track.m4a");
        let tagged_file = make_tagged_file(
            60.25,
            vec![
                TagItem::new(
                    ItemKey::TrackTitle,
                    ItemValue::Text("Only Title".to_string()),
                ),
                TagItem::new(ItemKey::TrackNumber, ItemValue::Text("NaN".to_string())),
                TagItem::new(ItemKey::DiscNumber, ItemValue::Text("disc".to_string())),
                TagItem::new(ItemKey::Year, ItemValue::Text("bad-year".to_string())),
                TagItem::new(
                    ItemKey::RecordingDate,
                    ItemValue::Text("  2004  ".to_string()),
                ),
            ],
        );

        let metadata = handler.extract_metadata_from_tags(&tagged_file, &path);

        assert_eq!(
            metadata.title.as_ref().map(|v| v.value.as_str()),
            Some("Only Title")
        );
        assert_eq!(
            metadata.artist.as_ref().map(|v| v.value.as_str()),
            Some("Fallback Artist")
        );
        assert_eq!(
            metadata.album.as_ref().map(|v| v.value.as_str()),
            Some("Fallback Album")
        );
        assert_eq!(metadata.track_number, None);
        assert_eq!(metadata.disc_number, None);
        assert_eq!(metadata.year.as_ref().map(|v| v.value), Some(2004));
        assert_eq!(metadata.duration.as_ref().map(|v| v.value), Some(60.25));
    }

    #[test]
    fn test_extract_metadata_from_tags_without_primary_tag_still_sets_duration_and_inference() {
        let handler = M4aHandler::new();
        let path = PathBuf::from("Collection/Artist Folder/Album Folder/track.m4a");
        let tagged_file = make_tagged_file_without_tags(12.0);

        let metadata = handler.extract_metadata_from_tags(&tagged_file, &path);

        assert_eq!(metadata.title, None);
        assert_eq!(
            metadata.artist.as_ref().map(|v| v.value.as_str()),
            Some("Artist Folder")
        );
        assert_eq!(
            metadata.album.as_ref().map(|v| v.value.as_str()),
            Some("Album Folder")
        );
        assert_eq!(metadata.year, None);
        assert_eq!(metadata.duration.as_ref().map(|v| v.value), Some(12.0));
        assert_eq!(metadata.format, "m4a");
    }

    #[test]
    fn test_extract_basic_metadata_sets_duration_and_inferred_path_fields() {
        let handler = M4aHandler::new();
        let path = PathBuf::from("Audio/Basic Artist/Basic Album/file.m4a");
        let tagged_file = make_tagged_file(99.0, vec![]);

        let metadata = handler.extract_basic_metadata(&tagged_file, &path);

        assert_eq!(metadata.title, None);
        assert_eq!(
            metadata.artist.as_ref().map(|v| v.value.as_str()),
            Some("Basic Artist")
        );
        assert_eq!(
            metadata.album.as_ref().map(|v| v.value.as_str()),
            Some("Basic Album")
        );
        assert_eq!(metadata.album_artist, None);
        assert_eq!(metadata.track_number, None);
        assert_eq!(metadata.disc_number, None);
        assert_eq!(metadata.year, None);
        assert_eq!(metadata.genre, None);
        assert_eq!(metadata.duration.as_ref().map(|v| v.value), Some(99.0));
        assert_eq!(metadata.format, "m4a");
        assert_eq!(metadata.path, path);
    }

    #[test]
    fn test_extract_basic_metadata_without_folder_context_has_no_inference() {
        let handler = M4aHandler::new();
        let path = PathBuf::from("track.m4a");
        let tagged_file = make_tagged_file(1.5, vec![]);

        let metadata = handler.extract_basic_metadata(&tagged_file, &path);

        assert_eq!(metadata.artist, None);
        assert_eq!(metadata.album, None);
        assert_eq!(metadata.duration.as_ref().map(|v| v.value), Some(1.5));
        assert_eq!(metadata.format, "m4a");
        assert_eq!(metadata.path, path);
    }
}
