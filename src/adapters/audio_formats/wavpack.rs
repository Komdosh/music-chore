//! WavPack format implementation of the AudioFile trait.

use lofty::{
    config::WriteOptions,
    file::{AudioFile as LoftyAudioFile, TaggedFile, TaggedFileExt},
    prelude::ItemKey,
    read_from_path,
    tag::{ItemValue, TagItem},
};

use std::path::Path;
use crate::adapters::audio_formats::wav::item_value_text;

use crate::core::domain::models::{FOLDER_INFERRED_CONFIDENCE, MetadataValue, Track, TrackMetadata};
use crate::core::domain::traits::{AudioFile, AudioFileError};
use crate::core::services::inference::{infer_album_from_path, infer_artist_from_path};

/// WavPack format handler
pub struct WavPackHandler;

impl WavPackHandler {
    /// Create a new WavPack handler
    pub fn new() -> Self {
        Self
    }
}

impl Default for WavPackHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioFile for WavPackHandler {
    fn can_handle(&self, path: &Path) -> bool {
        path.extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("wv"))
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["wv"]
    }

    fn read_metadata(&self, path: &Path) -> Result<Track, AudioFileError> {
        if !self.can_handle(path) {
            return Err(AudioFileError::UnsupportedFormat);
        }

        // Use lofty to read the file
        let tagged_file = read_from_path(path)
            .map_err(|e| AudioFileError::InvalidFile(format!("Failed to read WavPack file: {}", e)))?;

        // Extract metadata from tags and file properties
        let metadata = self.extract_metadata_from_tags(&tagged_file, path);

        Ok(Track::new(path.to_path_buf(), metadata))
    }

    fn write_metadata(&self, path: &Path, metadata: &TrackMetadata) -> Result<(), AudioFileError> {
        if !self.can_handle(path) {
            return Err(AudioFileError::UnsupportedFormat);
        }

        // Use lofty to write metadata to WavPack file
        let mut tagged_file = read_from_path(path)
            .map_err(|e| AudioFileError::InvalidFile(format!("Failed to read WavPack file: {}", e)))?;

        // Get or create the primary tag
        let tag = tagged_file.primary_tag_mut().ok_or_else(|| {
            AudioFileError::WriteError("WavPack file has no primary tag".to_string())
        })?;

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
            .map_err(|e| AudioFileError::WriteError(format!("Failed to save WavPack file: {}", e)))?;

        Ok(())
    }

    fn read_basic_info(&self, path: &Path) -> Result<TrackMetadata, AudioFileError> {
        if !self.can_handle(path) {
            return Err(AudioFileError::UnsupportedFormat);
        }

        let tagged_file = read_from_path(path)
            .map_err(|e| AudioFileError::InvalidFile(format!("Failed to read WavPack file: {}", e)))?;

        Ok(self.extract_basic_metadata(&tagged_file, path))
    }
}

impl WavPackHandler {
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

        // Get the primary tag
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
            format: "wv".to_string(),
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
            format: "wv".to_string(),
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
    fn test_wavpack_handler_supported_extensions() {
        let handler = WavPackHandler::new();
        let extensions = handler.supported_extensions();
        assert_eq!(extensions, vec!["wv"]);
    }

    #[test]
    fn test_wavpack_handler_can_handle() {
        let handler = WavPackHandler::new();

        assert!(handler.can_handle(&PathBuf::from("test.wv")));
        assert!(handler.can_handle(&PathBuf::from("test.WV")));
        assert!(!handler.can_handle(&PathBuf::from("test.flac")));
        assert!(!handler.can_handle(&PathBuf::from("test.mp3")));
    }

    #[test]
    fn test_wavpack_handler_new_creates_instance() {
        let handler = WavPackHandler::new();
        assert!(handler.can_handle(&PathBuf::from("test.wv")));
    }

    #[test]
    fn test_wavpack_handler_default_creates_instance() {
        let handler = WavPackHandler::default();
        assert!(handler.can_handle(&PathBuf::from("test.wv")));
    }

    #[test]
    fn test_wavpack_handler_read_metadata_unsupported_format() {
        let handler = WavPackHandler::new();
        let result = handler.read_metadata(&PathBuf::from("test.mp3"));
        assert!(matches!(result, Err(AudioFileError::UnsupportedFormat)));
    }

    #[test]
    fn test_wavpack_handler_write_metadata_unsupported_format() {
        let handler = WavPackHandler::new();
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
            format: "wv".to_string(),
            path: PathBuf::from("test.wv"),
        };
        let result = handler.write_metadata(&PathBuf::from("test.mp3"), &metadata);
        assert!(matches!(result, Err(AudioFileError::UnsupportedFormat)));
    }

    #[test]
    fn test_wavpack_handler_read_basic_info_unsupported_format() {
        let handler = WavPackHandler::new();
        let result = handler.read_basic_info(&PathBuf::from("test.mp3"));
        assert!(matches!(result, Err(AudioFileError::UnsupportedFormat)));
    }

    #[test]
    fn test_wavpack_handler_read_basic_info_nonexistent_file() {
        let handler = WavPackHandler::new();
        let result = handler.read_basic_info(&PathBuf::from("nonexistent.wv"));
        assert!(matches!(result, Err(AudioFileError::InvalidFile(_))));
    }

    #[test]
    fn test_wavpack_handler_read_metadata_nonexistent_file() {
        let handler = WavPackHandler::new();
        let result = handler.read_metadata(&PathBuf::from("nonexistent.wv"));
        assert!(matches!(result, Err(AudioFileError::InvalidFile(_))));
    }

    #[test]
    fn test_wavpack_handler_write_metadata_nonexistent_file() {
        let handler = WavPackHandler::new();
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
            format: "wv".to_string(),
            path: PathBuf::from("nonexistent.wv"),
        };
        let result = handler.write_metadata(&PathBuf::from("nonexistent.wv"), &metadata);
        assert!(matches!(result, Err(AudioFileError::InvalidFile(_))));
    }


    #[test]
    fn test_wavpack_handler_with_real_file_should_fail_on_dummy() {
        // Test that a dummy file (not a real WavPack file) produces an error
        let handler = WavPackHandler::new();
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.wv");
        
        // Create a dummy file that is not a real WavPack file
        fs::write(&test_file, b"not a real wavpack file").unwrap();
        
        let result = handler.read_metadata(&test_file);
        assert!(matches!(result, Err(AudioFileError::InvalidFile(_))));
    }

    #[test]
    fn test_wavpack_handler_write_metadata_with_all_fields() {
        let handler = WavPackHandler::new();
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.wv");
        
        // Create a dummy file to simulate a WavPack file for this test
        // In a real scenario, we'd need an actual WavPack file
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
            format: "wv".to_string(),
            path: test_file.clone(),
        };
        
        let result = handler.write_metadata(&test_file, &metadata);
        // This should fail because the dummy file is not a real WavPack file
        assert!(matches!(result, Err(AudioFileError::InvalidFile(_))));
    }

    #[test]
    fn test_wavpack_handler_write_metadata_with_partial_fields() {
        let handler = WavPackHandler::new();
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("partial.wv");
        
        // Create a dummy file to simulate a WavPack file for this test
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
            format: "wv".to_string(),
            path: test_file.clone(),
        };
        
        let result = handler.write_metadata(&test_file, &metadata);
        // This should fail because the dummy file is not a real WavPack file
        assert!(matches!(result, Err(AudioFileError::InvalidFile(_))));
    }

    #[test]
    fn test_wavpack_handler_extract_metadata_from_tags_folder_inference_artist_album() {
        use crate::core::domain::models::MetadataSource;

        let handler = WavPackHandler::new();
        let temp_dir = TempDir::new().unwrap();

        let test_artist_name = "Inferred Artist";
        let test_album_name = "Inferred Album";
        let relative_path = PathBuf::from(test_artist_name)
                                      .join(test_album_name)
                                      .join("track.wv");
        let test_file_path = temp_dir.path().join(&relative_path);
        
        fs::create_dir_all(test_file_path.parent().unwrap()).unwrap();
        // Copy the silent.wv fixture to the test path
        let fixture_source_path = PathBuf::from("tests/fixtures/wavpack/silent/silent.wv");
        fs::copy(&fixture_source_path, &test_file_path).expect("Failed to copy fixture file");

        let result = handler.read_metadata(&test_file_path);
        assert!(result.is_ok(), "Expected OK result, but got {:?}", result);
        let metadata = result.unwrap().metadata;

        // When infer_artist_from_path is called with test_file_path, it processes
        // components relative to the conceptual music library root.
        // The expected artist should be "Inferred Artist" and album "Inferred Album".
        println!("Test Path: {:?}", test_file_path);
        println!("Inferred Artist: {:?}", metadata.artist);
        println!("Inferred Album: {:?}", metadata.album);

        let artist_meta = metadata.artist.as_ref().unwrap();
        assert_eq!(artist_meta.value, test_artist_name);
        assert_eq!(artist_meta.confidence, FOLDER_INFERRED_CONFIDENCE);
        assert_eq!(artist_meta.source, MetadataSource::FolderInferred);

        let album_meta = metadata.album.as_ref().unwrap();
        assert_eq!(album_meta.value, test_album_name);
        assert_eq!(album_meta.confidence, FOLDER_INFERRED_CONFIDENCE);
        assert_eq!(album_meta.source, MetadataSource::FolderInferred);
        assert!(metadata.title.is_none());
    }

    #[test]
    fn test_wavpack_handler_extract_metadata_from_tags_folder_inference_artist_only() {
        use crate::core::domain::models::MetadataSource;

        let handler = WavPackHandler::new();
        let temp_dir = TempDir::new().unwrap();
        let temp_dir_name = temp_dir.path().file_name().unwrap().to_str().unwrap().to_string(); // Get the actual temp dir name

        let test_album_name = "Inferred Album"; // This folder will act as the album name
        let relative_test_path = PathBuf::from(test_album_name).join("track.wv");
        let test_file_path = temp_dir.path().join(&relative_test_path);
        
        fs::create_dir_all(test_file_path.parent().unwrap()).unwrap();
        // Copy the silent.wv fixture to the test path
        let fixture_source_path = PathBuf::from("tests/fixtures/wavpack/silent/silent.wv");
        fs::copy(&fixture_source_path, &test_file_path).expect("Failed to copy fixture file");

        let result = handler.read_metadata(&test_file_path);
        assert!(result.is_ok(), "Expected OK result, but got {:?}", result);
        let metadata = result.unwrap().metadata;

        // In this structure (TempDir/Inferred Album/track.wv):
        // Artist will be inferred from TempDir's name (components[len-3])
        let artist_meta = metadata.artist.as_ref().unwrap();
        assert_eq!(artist_meta.value, temp_dir_name);
        assert_eq!(artist_meta.confidence, FOLDER_INFERRED_CONFIDENCE);
        assert_eq!(artist_meta.source, MetadataSource::FolderInferred);
        
        // Album will be inferred from "Inferred Album" (parent.file_name())
        let album_meta = metadata.album.as_ref().unwrap();
        assert_eq!(album_meta.value, test_album_name);
        assert_eq!(album_meta.confidence, FOLDER_INFERRED_CONFIDENCE);
        assert_eq!(album_meta.source, MetadataSource::FolderInferred);
        assert!(metadata.title.is_none());
    }

    #[test]
    fn test_wavpack_handler_extract_metadata_from_tags_no_inference_flat_structure() {
        use crate::core::domain::models::MetadataSource;

        let handler = WavPackHandler::new();
        let temp_dir = TempDir::new().unwrap();
        let temp_dir_name = temp_dir.path().file_name().unwrap().to_str().unwrap().to_string(); // Get the actual temp dir name
        let temp_dir_grandparent_name = temp_dir.path().parent().unwrap().file_name().unwrap().to_str().unwrap().to_string(); // Get the parent of the temp dir

        let test_file_name = "track.wv";
        let test_file_path = temp_dir.path().join(test_file_name);
        
        // Copy the silent.wv fixture to the test path
        let fixture_source_path = PathBuf::from("tests/fixtures/wavpack/silent/silent.wv");
        fs::copy(&fixture_source_path, &test_file_path).expect("Failed to copy fixture file");

        let result = handler.read_metadata(&test_file_path);
        assert!(result.is_ok(), "Expected OK result, but got {:?}", result);
        let metadata = result.unwrap().metadata;

        // In a flat structure (TempDir/track.wv):
        // Artist will be inferred from the grandparent of the TempDir (components[len-3])
        let artist_meta = metadata.artist.as_ref().unwrap();
        assert_eq!(artist_meta.value, temp_dir_grandparent_name);
        assert_eq!(artist_meta.confidence, FOLDER_INFERRED_CONFIDENCE);
        assert_eq!(artist_meta.source, MetadataSource::FolderInferred);
        
        // Album will be inferred from the TempDir name itself (parent.file_name())
        let album_meta = metadata.album.as_ref().unwrap();
        assert_eq!(album_meta.value, temp_dir_name);
        assert_eq!(album_meta.confidence, FOLDER_INFERRED_CONFIDENCE);
        assert_eq!(album_meta.source, MetadataSource::FolderInferred);
        assert!(metadata.title.is_none());
    }
}