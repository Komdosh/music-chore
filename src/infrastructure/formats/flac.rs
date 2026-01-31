//! FLAC format implementation of the AudioFile trait.

use lofty::{
    file::{AudioFile as LoftyAudioFile, TaggedFile, TaggedFileExt},
    prelude::ItemKey,
    read_from_path,
    tag::ItemValue,
};

use std::path::Path;

use crate::domain::models::{MetadataValue, Track, TrackMetadata};
use crate::domain::traits::{AudioFile, AudioFileError};
use crate::services::inference::{infer_album_from_path, infer_artist_from_path};

/// FLAC format handler
pub struct FlacHandler;

impl FlacHandler {
    /// Create a new FLAC handler
    pub fn new() -> Self {
        Self
    }
}

impl Default for FlacHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioFile for FlacHandler {
    fn can_handle(&self, path: &Path) -> bool {
        path.extension()
            .map_or(false, |ext| ext.eq_ignore_ascii_case("flac"))
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["flac"]
    }

    fn read_metadata(&self, path: &Path) -> Result<Track, AudioFileError> {
        if !self.can_handle(path) {
            return Err(AudioFileError::UnsupportedFormat);
        }

        // Use lofty to read the file
        let tagged_file = read_from_path(path)
            .map_err(|e| AudioFileError::InvalidFile(format!("Failed to read FLAC file: {}", e)))?;

        // Extract metadata from tags and file properties
        let metadata = self.extract_metadata_from_tags(&tagged_file, path);

        Ok(Track {
            file_path: path.to_path_buf(),
            metadata,
        })
    }

    fn write_metadata(&self, path: &Path, _metadata: &TrackMetadata) -> Result<(), AudioFileError> {
        if !self.can_handle(path) {
            return Err(AudioFileError::UnsupportedFormat);
        }

        // TODO: Implement FLAC metadata writing
        // For now, return an error to indicate it's not implemented
        Err(AudioFileError::WriteError(
            "FLAC metadata writing not yet implemented".to_string(),
        ))
    }

    fn read_basic_info(&self, path: &Path) -> Result<TrackMetadata, AudioFileError> {
        if !self.can_handle(path) {
            return Err(AudioFileError::UnsupportedFormat);
        }

        let tagged_file = read_from_path(path)
            .map_err(|e| AudioFileError::InvalidFile(format!("Failed to read FLAC file: {}", e)))?;

        Ok(self.extract_basic_metadata(&tagged_file, path))
    }
}

impl FlacHandler {
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

        // Get the primary tag (usually Vorbis Comments for FLAC)
        if let Some(tag) = tagged_file.primary_tag() {
            for tag_item in tag.items() {
                // Helper function to convert ItemValue to string
                let item_value_str = match tag_item.value() {
                    ItemValue::Text(s) => s.to_string(),
                    ItemValue::Locator(s) => s.to_string(),
                    ItemValue::Binary(_) => format!("<binary data>"),
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
            infer_artist_from_path(path).map(|artist| MetadataValue::inferred(artist, 0.8))
        } else {
            artist
        };

        let inferred_album = if album.is_none() {
            infer_album_from_path(path).map(|album| MetadataValue::inferred(album, 0.8))
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
            format: "flac".to_string(),
            path: path.to_path_buf(),
        }
    }

    /// Extract basic metadata (minimal parsing for performance)
    fn extract_basic_metadata(&self, tagged_file: &TaggedFile, path: &Path) -> TrackMetadata {
        // For basic info, just get format, duration, and use folder inference
        let properties = tagged_file.properties();
        let duration = Some(MetadataValue::embedded(properties.duration().as_secs_f64()));

        let inferred_artist =
            infer_artist_from_path(path).map(|artist| MetadataValue::inferred(artist, 0.8));
        let inferred_album =
            infer_album_from_path(path).map(|album| MetadataValue::inferred(album, 0.8));

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
            format: "flac".to_string(),
            path: path.to_path_buf(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_flac_handler_supported_extensions() {
        let handler = FlacHandler::new();
        let extensions = handler.supported_extensions();
        assert_eq!(extensions, vec!["flac"]);
    }

    #[test]
    fn test_flac_handler_can_handle() {
        let handler = FlacHandler::new();

        assert!(handler.can_handle(&PathBuf::from("test.flac")));
        assert!(handler.can_handle(&PathBuf::from("test.FLAC")));
        assert!(!handler.can_handle(&PathBuf::from("test.mp3")));
        assert!(!handler.can_handle(&PathBuf::from("test.wav")));
    }
}
