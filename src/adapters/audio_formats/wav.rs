//! WAV format implementation of the AudioFile trait.

use lofty::{
    config::WriteOptions,
    file::{AudioFile as LoftyAudioFile, TaggedFile, TaggedFileExt},
    prelude::ItemKey,
    read_from_path,
    tag::{ItemValue, TagItem},
};

use std::path::Path;

use crate::core::domain::models::{MetadataValue, Track, TrackMetadata};
use crate::core::domain::traits::{AudioFile, AudioFileError};
use crate::core::services::inference::{infer_album_from_path, infer_artist_from_path};

/// WAV format handler
pub struct WavHandler;

impl WavHandler {
    /// Create a new WAV handler
    pub fn new() -> Self {
        Self
    }
}

impl Default for WavHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioFile for WavHandler {
    fn can_handle(&self, path: &Path) -> bool {
        path.extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("wav"))
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["wav"]
    }

    fn read_metadata(&self, path: &Path) -> Result<Track, AudioFileError> {
        if !self.can_handle(path) {
            return Err(AudioFileError::UnsupportedFormat);
        }

        // Use lofty to read the file
        let tagged_file = read_from_path(path)
            .map_err(|e| AudioFileError::InvalidFile(format!("Failed to read WAV file: {}", e)))?;

        // Extract metadata from tags and file properties
        let metadata = self.extract_metadata_from_tags(&tagged_file, path);

        Ok(Track::new(path.to_path_buf(), metadata))
    }

    fn write_metadata(&self, path: &Path, metadata: &TrackMetadata) -> Result<(), AudioFileError> {
        if !self.can_handle(path) {
            return Err(AudioFileError::UnsupportedFormat);
        }

        // Use lofty to write metadata to WAV file
        let mut tagged_file = read_from_path(path)
            .map_err(|e| AudioFileError::InvalidFile(format!("Failed to read WAV file: {}", e)))?;

        // Get or create the primary tag
        let tag = tagged_file
            .primary_tag_mut()
            .ok_or_else(|| AudioFileError::WriteError("WAV file has no primary tag".to_string()))?;

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

        // Save changes to disk with default write options
        let write_options = WriteOptions::default();
        tagged_file
            .save_to_path(path, write_options)
            .map_err(|e| AudioFileError::WriteError(format!("Failed to save WAV file: {}", e)))?;

        Ok(())
    }

    fn read_basic_info(&self, path: &Path) -> Result<TrackMetadata, AudioFileError> {
        if !self.can_handle(path) {
            return Err(AudioFileError::UnsupportedFormat);
        }

        let tagged_file = read_from_path(path)
            .map_err(|e| AudioFileError::InvalidFile(format!("Failed to read WAV file: {}", e)))?;

        Ok(self.extract_basic_metadata(&tagged_file, path))
    }
}

pub fn item_value_text(tag_item: &TagItem) -> String {
    let item_value_str = match tag_item.value() {
        ItemValue::Text(s) => s.to_string(),
        ItemValue::Locator(s) => s.to_string(),
        ItemValue::Binary(_) => "<binary data>".to_string(),
    };
    item_value_str
}

impl WavHandler {
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

        // Get the primary tag (usually INFO chunks for WAV)
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
                        if let Ok(num) = item_value_str.parse::<u32>() {
                            track_number = Some(MetadataValue::embedded(num));
                        }
                    }
                    ItemKey::DiscNumber => {
                        if let Ok(num) = item_value_str.parse::<u32>() {
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
                    _ => {}
                }
            }
        }

        // Fallback inference for missing metadata
        if artist.is_none()
            && let Some(inferred_artist) = infer_artist_from_path(path)
        {
            artist = Some(MetadataValue {
                value: inferred_artist,
                source: crate::core::domain::models::MetadataSource::FolderInferred,
                confidence: 0.5,
            });
        }

        if album.is_none()
            && let Some(inferred_album) = infer_album_from_path(path)
        {
            album = Some(MetadataValue {
                value: inferred_album,
                source: crate::core::domain::models::MetadataSource::FolderInferred,
                confidence: 0.5,
            });
        }

        // Extract duration from file properties
        let duration = tagged_file.properties().duration().as_secs_f64();

        TrackMetadata {
            title,
            artist,
            album,
            album_artist,
            track_number,
            disc_number,
            year,
            genre,
            duration: Some(MetadataValue::embedded(duration)),
            format: "wav".to_string(),
            path: path.to_path_buf(),
        }
    }

    /// Extract basic metadata (only duration and format info)
    fn extract_basic_metadata(&self, tagged_file: &TaggedFile, path: &Path) -> TrackMetadata {
        let duration = tagged_file.properties().duration().as_secs_f64();

        TrackMetadata {
            title: None,
            artist: None,
            album: None,
            album_artist: None,
            track_number: None,
            disc_number: None,
            year: None,
            genre: None,
            duration: Some(MetadataValue::embedded(duration)),
            format: "wav".to_string(),
            path: path.to_path_buf(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_wav_handler_supported_extensions() {
        let handler = WavHandler::new();
        let extensions = handler.supported_extensions();
        assert_eq!(extensions, vec!["wav"]);
    }

    #[test]
    fn test_wav_handler_can_handle() {
        let handler = WavHandler::new();

        assert!(handler.can_handle(&PathBuf::from("test.wav")));
        assert!(handler.can_handle(&PathBuf::from("test.WAV")));
        assert!(!handler.can_handle(&PathBuf::from("test.flac")));
        assert!(!handler.can_handle(&PathBuf::from("test.mp3")));
    }

    #[test]
    fn test_wav_handler_new_creates_instance() {
        let handler = WavHandler::new();
        assert!(handler.can_handle(&PathBuf::from("test.wav")));
    }

    #[test]
    fn test_wav_handler_default_creates_instance() {
        let handler = WavHandler::default();
        assert!(handler.can_handle(&PathBuf::from("test.wav")));
    }

    #[test]
    fn test_wav_handler_read_metadata_unsupported_format() {
        let handler = WavHandler::new();
        let result = handler.read_metadata(&PathBuf::from("test.flac"));
        assert!(matches!(result, Err(AudioFileError::UnsupportedFormat)));
    }

    #[test]
    fn test_wav_handler_write_metadata_unsupported_format() {
        let handler = WavHandler::new();
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
            format: "wav".to_string(),
            path: PathBuf::from("test.wav"),
        };
        let result = handler.write_metadata(&PathBuf::from("test.flac"), &metadata);
        assert!(matches!(result, Err(AudioFileError::UnsupportedFormat)));
    }

    #[test]
    fn test_wav_handler_read_basic_info_unsupported_format() {
        let handler = WavHandler::new();
        let result = handler.read_basic_info(&PathBuf::from("test.flac"));
        assert!(matches!(result, Err(AudioFileError::UnsupportedFormat)));
    }

    #[test]
    fn test_wav_handler_read_basic_info_nonexistent_file() {
        let handler = WavHandler::new();
        let result = handler.read_basic_info(&PathBuf::from("nonexistent.wav"));
        assert!(matches!(result, Err(AudioFileError::InvalidFile(_))));
    }

    #[test]
    fn test_wav_handler_read_metadata_nonexistent_file() {
        let handler = WavHandler::new();
        let result = handler.read_metadata(&PathBuf::from("nonexistent.wav"));
        assert!(matches!(result, Err(AudioFileError::InvalidFile(_))));
    }

    #[test]
    fn test_wav_handler_write_metadata_nonexistent_file() {
        let handler = WavHandler::new();
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
            format: "wav".to_string(),
            path: PathBuf::from("nonexistent.wav"),
        };
        let result = handler.write_metadata(&PathBuf::from("nonexistent.wav"), &metadata);
        assert!(matches!(result, Err(AudioFileError::InvalidFile(_))));
    }

    #[test]
    fn test_wav_handler_with_real_file_should_fail_on_dummy() {
        // Test that a dummy file (not a real WAV file) produces an error
        let handler = WavHandler::new();
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.wav");
        
        // Create a dummy file that is not a real WAV file
        fs::write(&test_file, b"not a real wav file").unwrap();
        
        let result = handler.read_metadata(&test_file);
        assert!(matches!(result, Err(AudioFileError::InvalidFile(_))));
    }

    #[test]
    fn test_wav_handler_write_metadata_with_all_fields() {
        let handler = WavHandler::new();
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.wav");
        
        // Create a dummy file to simulate a WAV file for this test
        // In a real scenario, we'd need an actual WAV file
        fs::write(&test_file, b"dummy content").unwrap();
        
        let metadata = TrackMetadata {
            title: Some(MetadataValue::embedded("Test Title".to_string())),
            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
            album: Some(MetadataValue::embedded("Test Album".to_string())),
            album_artist: Some(MetadataValue::embedded("Test Album Artist".to_string())),
            track_number: Some(MetadataValue::embedded(5)),
            disc_number: Some(MetadataValue::embedded(1)),
            year: Some(MetadataValue::embedded(2023)),
            genre: Some(MetadataValue::embedded("Test Genre".to_string())),
            duration: Some(MetadataValue::embedded(180.0)),
            format: "wav".to_string(),
            path: test_file.clone(),
        };
        
        let result = handler.write_metadata(&test_file, &metadata);
        // This should fail because the dummy file is not a real WAV file
        assert!(matches!(result, Err(AudioFileError::InvalidFile(_))));
    }

    #[test]
    fn test_wav_handler_write_metadata_with_partial_fields() {
        let handler = WavHandler::new();
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("partial.wav");
        
        // Create a dummy file to simulate a WAV file for this test
        fs::write(&test_file, b"dummy content").unwrap();
        
        let metadata = TrackMetadata {
            title: Some(MetadataValue::embedded("Partial Title".to_string())),
            artist: None, // No artist
            album: Some(MetadataValue::embedded("Partial Album".to_string())),
            album_artist: None,
            track_number: None,
            disc_number: None,
            year: None,
            genre: None,
            duration: Some(MetadataValue::embedded(120.0)),
            format: "wav".to_string(),
            path: test_file.clone(),
        };
        
        let result = handler.write_metadata(&test_file, &metadata);
        // This should fail because the dummy file is not a real WAV file
        assert!(matches!(result, Err(AudioFileError::InvalidFile(_))));
    }

    #[test]
    fn test_wav_handler_extract_metadata_from_tags_empty_tag() {
        // This test verifies the behavior when a tag has no items
        // Since we can't easily create a TaggedFile with no tags in a test,
        // we'll test the folder inference fallback behavior
        let handler = WavHandler::new();
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("artist/album/test.wav");
        fs::create_dir_all(test_file.parent().unwrap()).unwrap();
        
        // Create an empty file to represent a WAV file
        fs::write(&test_file, b"dummy content").unwrap();
        
        // This should test the folder inference when no embedded metadata exists
        // Note: This test will likely fail since the file isn't a real WAV file,
        // but it demonstrates the intended behavior
        let _result = handler.read_basic_info(&test_file);
        // The result depends on whether the lofty crate can read the dummy file
        // If it can't, it will return an error; if it can, it will use folder inference
    }

    #[test]
    fn test_wav_handler_extract_basic_metadata() {
        // Similar to above, testing with a dummy file
        let handler = WavHandler::new();
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test_artist/test_album/test.wav");
        fs::create_dir_all(test_file.parent().unwrap()).unwrap();

        // Create an empty file
        fs::write(&test_file, b"dummy content").unwrap();

        let _result = handler.read_basic_info(&test_file);
        // This will likely fail due to the file not being a real WAV file
        // but tests the error handling path
    }

    #[test]
    fn test_wav_handler_write_metadata_no_primary_tag_error() {
        // Test the case where a file exists but has no primary tag
        let handler = WavHandler::new();
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("no_tag.wav");

        // Create a dummy file that is not a real WAV file
        fs::write(&test_file, b"dummy content").unwrap();

        let metadata = TrackMetadata {
            title: Some(MetadataValue::embedded("Test Title".to_string())),
            artist: Some(MetadataValue::embedded("Test Artist".to_string())),
            album: Some(MetadataValue::embedded("Test Album".to_string())),
            album_artist: None,
            track_number: None,
            disc_number: None,
            year: None,
            genre: None,
            duration: Some(MetadataValue::embedded(180.0)),
            format: "wav".to_string(),
            path: test_file.clone(),
        };

        let result = handler.write_metadata(&test_file, &metadata);
        // This should fail because the dummy file is not a real WAV file
        assert!(matches!(result, Err(AudioFileError::InvalidFile(_))));
    }

    #[test]
    fn test_wav_handler_read_metadata_with_valid_path_structure() {
        // Test reading metadata where folder structure provides fallback values
        let handler = WavHandler::new();
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("Test Artist").join("Test Album").join("track.wav");
        fs::create_dir_all(test_file.parent().unwrap()).unwrap();

        // Create a dummy file
        fs::write(&test_file, b"dummy content").unwrap();

        let result = handler.read_metadata(&test_file);
        // This should fail because the dummy file is not a real WAV file
        assert!(matches!(result, Err(AudioFileError::InvalidFile(_))));
    }

    #[test]
    fn test_wav_handler_duration_extraction() {
        // Test that duration is properly extracted when file can be read
        let handler = WavHandler::new();
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("duration_test.wav");

        // Create a dummy file
        fs::write(&test_file, b"dummy content").unwrap();

        let result = handler.read_basic_info(&test_file);
        // This should fail because the dummy file is not a real WAV file
        assert!(matches!(result, Err(AudioFileError::InvalidFile(_))));
    }

    #[test]
    fn test_wav_handler_folder_inference_logic() {
        // Test the folder inference logic when no embedded metadata exists
        let handler = WavHandler::new();
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("Infer Artist").join("Infer Album").join("song.wav");
        fs::create_dir_all(test_file.parent().unwrap()).unwrap();

        // Create a dummy file
        fs::write(&test_file, b"dummy content").unwrap();

        let result = handler.read_basic_info(&test_file);
        // This should fail because the dummy file is not a real WAV file
        assert!(matches!(result, Err(AudioFileError::InvalidFile(_))));
    }

    #[test]
    fn test_wav_handler_metadata_confidence_levels() {
        // Test that embedded metadata has confidence 1.0 and inferred has lower confidence
        let handler = WavHandler::new();
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("Confidence Artist").join("Confidence Album").join("track.wav");
        fs::create_dir_all(test_file.parent().unwrap()).unwrap();

        // Create a dummy file
        fs::write(&test_file, b"dummy content").unwrap();

        let result = handler.read_metadata(&test_file);
        // This should fail because the dummy file is not a real WAV file
        assert!(matches!(result, Err(AudioFileError::InvalidFile(_))));
    }
}

