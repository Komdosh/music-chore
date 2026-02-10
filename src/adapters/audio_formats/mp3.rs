//! MP3 format implementation of the AudioFile trait.

use lofty::{
    config::WriteOptions,
    file::{AudioFile as LoftyAudioFile, TaggedFile, TaggedFileExt},
    prelude::ItemKey,
    read_from_path,
    tag::{ItemValue, TagItem},
};

use crate::adapters::audio_formats::wav::item_value_text;
use crate::core::domain::models::{
    FOLDER_INFERRED_CONFIDENCE, MetadataValue, Track, TrackMetadata,
};
use crate::core::domain::traits::{AudioFile, AudioFileError};
use crate::core::services::inference::{infer_album_from_path, infer_artist_from_path};
use std::path::Path;

/// MP3 format handler
pub struct Mp3Handler;

impl Mp3Handler {
    /// Create a new MP3 handler
    pub fn new() -> Self {
        Self
    }
}

impl Default for Mp3Handler {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioFile for Mp3Handler {
    fn can_handle(&self, path: &Path) -> bool {
        path.extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("mp3"))
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["mp3"]
    }

    fn read_metadata(&self, path: &Path) -> Result<Track, AudioFileError> {
        if !self.can_handle(path) {
            return Err(AudioFileError::UnsupportedFormat);
        }

        // Use lofty to read the file with error handling for problematic ID3 tags
        let tagged_file = read_from_path(path).map_err(|e| {
            let error_msg = format!("{}", e);
            if error_msg.contains("encrypted frame") || error_msg.contains("data length indicator")
            {
                AudioFileError::InvalidFile(format!(
                    "MP3 file contains unsupported encrypted/compressed frames: {}",
                    e
                ))
            } else {
                AudioFileError::InvalidFile(format!("Failed to read MP3 file: {}", e))
            }
        })?;

        // Extract metadata from tags and file properties
        let metadata = self.extract_metadata_from_tags(&tagged_file, path);

        Ok(Track::new(path.to_path_buf(), metadata))
    }

    fn write_metadata(&self, path: &Path, metadata: &TrackMetadata) -> Result<(), AudioFileError> {
        if !self.can_handle(path) {
            return Err(AudioFileError::UnsupportedFormat);
        }

        // Use lofty to write metadata to MP3 file
        let mut tagged_file = read_from_path(path)
            .map_err(|e| AudioFileError::InvalidFile(format!("Failed to read MP3 file: {}", e)))?;

        // Get or create the primary tag (ID3v2 for MP3)
        let tag = tagged_file
            .primary_tag_mut()
            .ok_or_else(|| AudioFileError::WriteError("MP3 file has no primary tag".to_string()))?;

        // Helper function to set a tag item
        let mut set_tag = |key: ItemKey, value: &str| {
            tag.insert(TagItem::new(key, ItemValue::Text(value.to_string())));
        };

        // Write metadata fields that have values
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

        // Save the changes to disk with default write options
        let write_options = WriteOptions::default();
        tagged_file
            .save_to_path(path, write_options)
            .map_err(|e| AudioFileError::WriteError(format!("Failed to save MP3 file: {}", e)))?;

        Ok(())
    }

    fn read_basic_info(&self, path: &Path) -> Result<TrackMetadata, AudioFileError> {
        if !self.can_handle(path) {
            return Err(AudioFileError::UnsupportedFormat);
        }

        let tagged_file = read_from_path(path).map_err(|e| {
            let error_msg = format!("{}", e);
            if error_msg.contains("encrypted frame") || error_msg.contains("data length indicator")
            {
                AudioFileError::InvalidFile(format!(
                    "MP3 file contains unsupported encrypted/compressed frames: {}",
                    e
                ))
            } else {
                AudioFileError::InvalidFile(format!("Failed to read MP3 file: {}", e))
            }
        })?;

        Ok(self.extract_basic_metadata(&tagged_file, path))
    }
}

impl Mp3Handler {
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

        // Get the primary tag (usually ID3v2 for MP3)
        if let Some(tag) = tagged_file.primary_tag() {
            for tag_item in tag.items() {
                // Helper function to convert ItemValue to string
                let item_value_str = item_value_text(tag_item);

                match tag_item.key() {
                    ItemKey::TrackTitle => {
                        title = Some(MetadataValue::embedded(item_value_str));
                    }
                    ItemKey::TrackArtist => {
                        artist = Some(MetadataValue::embedded(item_value_str));
                    }
                    ItemKey::AlbumTitle => {
                        album = Some(MetadataValue::embedded(item_value_str));
                    }
                    ItemKey::AlbumArtist => {
                        album_artist = Some(MetadataValue::embedded(item_value_str));
                    }
                    ItemKey::TrackNumber => {
                        // Handle both "track/total" formats and plain numbers
                        let clean_track =
                            item_value_str.split('/').next().unwrap_or(&item_value_str);
                        if let Ok(num) = clean_track.trim().parse::<u32>() {
                            track_number = Some(MetadataValue::embedded(num));
                        }
                    }
                    ItemKey::DiscNumber => {
                        // Handle both "disc/total" formats and plain numbers
                        let clean_disc =
                            item_value_str.split('/').next().unwrap_or(&item_value_str);
                        if let Ok(num) = clean_disc.trim().parse::<u32>() {
                            disc_number = Some(MetadataValue::embedded(num));
                        }
                    }
                    ItemKey::Year => {
                        if let Ok(year_val) = item_value_str.parse::<u32>() {
                            year = Some(MetadataValue::embedded(year_val));
                        }
                    }
                    ItemKey::Genre => {
                        genre = Some(MetadataValue::embedded(item_value_str));
                    }
                    ItemKey::RecordingDate => {
                        let clean_value = item_value_str.trim();
                        if let Ok(year_val) = clean_value.parse::<u32>() {
                            year = Some(MetadataValue::embedded(year_val));
                        }
                    }
                    _ => {} // Ignore other tags for now
                }
            }
        }

        // Get duration from file properties
        let properties = tagged_file.properties();
        let duration = Some(MetadataValue::embedded(properties.duration().as_secs_f64()));

        // Apply folder inference as fallback when embedded metadata is missing
        let inferred_artist = if artist.is_none() {
            infer_artist_from_path(path)
                .map(|artist| MetadataValue::inferred(artist, FOLDER_INFERRED_CONFIDENCE))
        } else {
            artist
        };

        let inferred_album = if album.is_none() {
            infer_album_from_path(path)
                .map(|album| MetadataValue::inferred(album, FOLDER_INFERRED_CONFIDENCE))
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
            format: "mp3".to_string(),
            path: path.to_path_buf(),
        }
    }

    /// Extract basic metadata (minimal parsing for performance)
    fn extract_basic_metadata(&self, tagged_file: &TaggedFile, path: &Path) -> TrackMetadata {
        // For basic info, just get format, duration, and use folder inference
        let properties = tagged_file.properties();
        let duration = Some(MetadataValue::embedded(properties.duration().as_secs_f64()));

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
            format: "mp3".to_string(),
            path: path.to_path_buf(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::domain::models::MetadataSource;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir; // Added MetadataSource import

    #[test]
    fn test_mp3_handler_supported_extensions() {
        let handler = Mp3Handler::new();
        let extensions = handler.supported_extensions();
        assert_eq!(extensions, vec!["mp3"]);
    }

    #[test]
    fn test_mp3_handler_can_handle() {
        let handler = Mp3Handler::new();

        assert!(handler.can_handle(&PathBuf::from("test.mp3")));
        assert!(handler.can_handle(&PathBuf::from("test.MP3")));
        assert!(!handler.can_handle(&PathBuf::from("test.flac")));
        assert!(!handler.can_handle(&PathBuf::from("test.wav")));
    }

    #[test]
    fn test_mp3_handler_new_creates_instance() {
        let handler = Mp3Handler::new();
        assert!(handler.can_handle(&PathBuf::from("test.mp3")));
    }

    #[test]
    fn test_mp3_handler_default_creates_instance() {
        let handler = Mp3Handler::default();
        assert!(handler.can_handle(&PathBuf::from("test.mp3")));
    }

    #[test]
    fn test_mp3_handler_read_metadata_unsupported_format() {
        let handler = Mp3Handler::new();
        let result = handler.read_metadata(&PathBuf::from("test.flac"));
        assert!(matches!(result, Err(AudioFileError::UnsupportedFormat)));
    }

    #[test]
    fn test_mp3_handler_write_metadata_unsupported_format() {
        let handler = Mp3Handler::new();
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
            format: "mp3".to_string(),
            path: PathBuf::from("test.mp3"),
        };
        let result = handler.write_metadata(&PathBuf::from("test.flac"), &metadata);
        assert!(matches!(result, Err(AudioFileError::UnsupportedFormat)));
    }

    #[test]
    fn test_mp3_handler_read_basic_info_unsupported_format() {
        let handler = Mp3Handler::new();
        let result = handler.read_basic_info(&PathBuf::from("test.flac"));
        assert!(matches!(result, Err(AudioFileError::UnsupportedFormat)));
    }

    #[test]
    fn test_mp3_handler_read_basic_info_nonexistent_file() {
        let handler = Mp3Handler::new();
        let result = handler.read_basic_info(&PathBuf::from("nonexistent.mp3"));
        assert!(matches!(result, Err(AudioFileError::InvalidFile(_))));
    }

    #[test]
    fn test_mp3_handler_read_metadata_nonexistent_file() {
        let handler = Mp3Handler::new();
        let result = handler.read_metadata(&PathBuf::from("nonexistent.mp3"));
        assert!(matches!(result, Err(AudioFileError::InvalidFile(_))));
    }

    #[test]
    fn test_mp3_handler_write_metadata_nonexistent_file() {
        let handler = Mp3Handler::new();
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
            format: "mp3".to_string(),
            path: PathBuf::from("nonexistent.mp3"),
        };
        let result = handler.write_metadata(&PathBuf::from("nonexistent.mp3"), &metadata);
        assert!(matches!(result, Err(AudioFileError::InvalidFile(_))));
    }

    #[test]
    fn test_mp3_handler_with_real_file_should_fail_on_dummy() {
        // Test that a dummy file (not a real MP3 file) produces an error
        let handler = Mp3Handler::new();
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.mp3");

        // Create a dummy file that is not a real MP3 file
        fs::write(&test_file, b"not a real mp3 file").unwrap();

        let result = handler.read_metadata(&test_file);
        assert!(matches!(result, Err(AudioFileError::InvalidFile(_))));
    }

    // --- New tests for successful read operations ---

    #[test]
    fn test_mp3_handler_read_metadata_success() {
        let handler = Mp3Handler::new();
        let test_file_path = PathBuf::from("tests/fixtures/mp3/simple/track1.mp3");

        let result = handler.read_metadata(&test_file_path);
        assert!(result.is_ok(), "Expected OK result, but got {:?}", result);
        let track = result.unwrap();

        let metadata = track.metadata;

        let title_meta = metadata.title.as_ref().unwrap();
        assert_eq!(title_meta.value, "Test Track 1");
        assert_eq!(title_meta.source, MetadataSource::Embedded);
        assert_eq!(title_meta.confidence, 1.0);

        let artist_meta = metadata.artist.as_ref().unwrap();
        assert_eq!(artist_meta.value, "Test Artist");
        assert_eq!(artist_meta.source, MetadataSource::Embedded);
        assert_eq!(artist_meta.confidence, 1.0);

        let album_meta = metadata.album.as_ref().unwrap();
        assert_eq!(album_meta.value, "Test Album");
        assert_eq!(album_meta.source, MetadataSource::Embedded);
        assert_eq!(album_meta.confidence, 1.0);

        assert!(metadata.album_artist.is_none());

        let track_number_meta = metadata.track_number.as_ref().unwrap();
        assert_eq!(track_number_meta.value, 1);
        assert_eq!(track_number_meta.source, MetadataSource::Embedded);
        assert_eq!(track_number_meta.confidence, 1.0);

        assert!(metadata.disc_number.is_none());

        let year_meta = metadata.year.as_ref().unwrap();
        assert_eq!(year_meta.value, 2024);
        assert_eq!(year_meta.source, MetadataSource::Embedded);
        assert_eq!(year_meta.confidence, 1.0);

        let genre_meta = metadata.genre.as_ref().unwrap();
        assert_eq!(genre_meta.value, "Test Genre");
        assert_eq!(genre_meta.source, MetadataSource::Embedded);
        assert_eq!(genre_meta.confidence, 1.0);

        let duration_meta = metadata.duration.as_ref().unwrap();
        assert!(duration_meta.value > 0.0); // Duration should be positive
        assert_eq!(duration_meta.source, MetadataSource::Embedded);
        assert_eq!(duration_meta.confidence, 1.0);

        assert_eq!(metadata.format, "mp3");
    }

    #[test]
    fn test_mp3_handler_read_basic_info_success() {
        let handler = Mp3Handler::new();
        let test_file_path = PathBuf::from("tests/fixtures/mp3/simple/track1.mp3");

        let result = handler.read_basic_info(&test_file_path);
        assert!(result.is_ok(), "Expected OK result, but got {:?}", result);
        let metadata = result.unwrap();

        assert!(metadata.title.is_none());

        let artist_meta = metadata.artist.as_ref().unwrap();
        assert_eq!(artist_meta.value, "mp3");
        assert_eq!(artist_meta.source, MetadataSource::FolderInferred);
        assert_eq!(artist_meta.confidence, FOLDER_INFERRED_CONFIDENCE);

        let album_meta = metadata.album.as_ref().unwrap();
        assert_eq!(album_meta.value, "simple");
        assert_eq!(album_meta.source, MetadataSource::FolderInferred);
        assert_eq!(album_meta.confidence, FOLDER_INFERRED_CONFIDENCE);

        assert!(metadata.album_artist.is_none());
        assert!(metadata.track_number.is_none());
        assert!(metadata.disc_number.is_none());
        assert!(metadata.year.is_none());
        assert!(metadata.genre.is_none());
        assert_eq!(metadata.format, "mp3");
        assert!(metadata.duration.is_some());

        // Duration should be embedded
        let duration_meta = metadata.duration.as_ref().unwrap();
        assert_eq!(duration_meta.source, MetadataSource::Embedded);
        assert_eq!(duration_meta.confidence, 1.0);
    }
}
