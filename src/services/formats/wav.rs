//! WAV format implementation of the AudioFile trait.

use lofty::{
    config::WriteOptions,
    file::{AudioFile as LoftyAudioFile, TaggedFile, TaggedFileExt},
    prelude::ItemKey,
    read_from_path,
    tag::{ItemValue, TagItem},
};

use std::path::Path;

use crate::domain::models::{MetadataValue, Track, TrackMetadata};
use crate::domain::traits::{AudioFile, AudioFileError};
use crate::services::inference::{infer_album_from_path, infer_artist_from_path};

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
                let item_value_str = match tag_item.value() {
                    ItemValue::Text(s) => s.to_string(),
                    ItemValue::Locator(s) => s.to_string(),
                    ItemValue::Binary(_) => "<binary data>".to_string(),
                };

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
            && let Some(inferred_artist) = infer_artist_from_path(path) {
                artist = Some(MetadataValue {
                    value: inferred_artist,
                    source: crate::domain::models::MetadataSource::FolderInferred,
                    confidence: 0.5,
                });
            }

        if album.is_none()
            && let Some(inferred_album) = infer_album_from_path(path) {
                album = Some(MetadataValue {
                    value: inferred_album,
                    source: crate::domain::models::MetadataSource::FolderInferred,
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
