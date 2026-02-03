//! MP3 format implementation of the AudioFile trait.

use lofty::{
    config::WriteOptions,
    file::{AudioFile as LoftyAudioFile, TaggedFile, TaggedFileExt},
    prelude::ItemKey,
    read_from_path,
    tag::{ItemValue, TagItem},
};

use std::path::Path;

use crate::domain::models::{MetadataValue, Track, TrackMetadata, FOLDER_INFERRED_CONFIDENCE};
use crate::domain::traits::{AudioFile, AudioFileError};
use crate::services::inference::{infer_album_from_path, infer_artist_from_path};

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

        // Use lofty to read the file
        let tagged_file = read_from_path(path)
            .map_err(|e| AudioFileError::InvalidFile(format!("Failed to read MP3 file: {}", e)))?;

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

        let tagged_file = read_from_path(path)
            .map_err(|e| AudioFileError::InvalidFile(format!("Failed to read MP3 file: {}", e)))?;

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
            infer_artist_from_path(path).map(|artist| MetadataValue::inferred(artist, FOLDER_INFERRED_CONFIDENCE))
        } else {
            artist
        };

        let inferred_album = if album.is_none() {
            infer_album_from_path(path).map(|album| MetadataValue::inferred(album, FOLDER_INFERRED_CONFIDENCE))
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

        let inferred_artist =
            infer_artist_from_path(path).map(|artist| MetadataValue::inferred(artist, FOLDER_INFERRED_CONFIDENCE));
        let inferred_album =
            infer_album_from_path(path).map(|album| MetadataValue::inferred(album, FOLDER_INFERRED_CONFIDENCE));

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
    use std::path::PathBuf;

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
}
