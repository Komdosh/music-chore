use dsf::DsfFile;
use id3::TagLike;
use std::path::Path;

use crate::core::domain::models::{
    FOLDER_INFERRED_CONFIDENCE, MetadataValue, Track, TrackMetadata,
};
use crate::core::domain::traits::{AudioFile, AudioFileError};
use crate::core::services::inference::{infer_album_from_path, infer_artist_from_path};

/// DSF format handler
pub struct DsfHandler;

impl DsfHandler {
    /// Create a new DSF handler
    pub fn new() -> Self {
        Self
    }
}

impl Default for DsfHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioFile for DsfHandler {
    fn can_handle(&self, path: &Path) -> bool {
        path.extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("dsf"))
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["dsf"]
    }

    fn read_metadata(&self, path: &Path) -> Result<Track, AudioFileError> {
        if !self.can_handle(path) {
            return Err(AudioFileError::UnsupportedFormat);
        }

        let dsf_file = DsfFile::open(path)
            .map_err(|e| AudioFileError::InvalidFile(format!("Failed to read DSF file: {}", e)))?;

        let metadata = self.extract_metadata_from_dsf_file(&dsf_file, path);

        Ok(Track::new(path.to_path_buf(), metadata))
    }

    fn write_metadata(
        &self,
        _path: &Path,
        _metadata: &TrackMetadata,
    ) -> Result<(), AudioFileError> {
        // The `dsf` crate does not support writing ID3 tags directly.
        // Therefore, writing DSF metadata is currently not supported.
        Err(AudioFileError::WriteError(
            "Writing DSF metadata is not supported.".to_string(),
        ))
    }

    fn read_basic_info(&self, path: &Path) -> Result<TrackMetadata, AudioFileError> {
        if !self.can_handle(path) {
            return Err(AudioFileError::UnsupportedFormat);
        }

        let dsf_file = DsfFile::open(path)
            .map_err(|e| AudioFileError::InvalidFile(format!("Failed to read DSF file: {}", e)))?;

        Ok(self.extract_basic_metadata_from_dsf_file(&dsf_file, path))
    }
}

impl DsfHandler {
    /// Extract metadata from dsf::DsfFile and convert to our TrackMetadata
    fn extract_metadata_from_dsf_file(&self, dsf_file: &DsfFile, path: &Path) -> TrackMetadata {
        let mut title = None;
        let mut artist = None;
        let mut album = None;
        let mut album_artist = None;
        let mut track_number = None;
        let mut disc_number = None;
        let mut year = None;
        let mut genre = None;

        if let Some(tag) = dsf_file.id3_tag() {
            title = TagLike::title(tag).map(|s| MetadataValue::embedded(s.to_string()));
            artist = TagLike::artist(tag).map(|s| MetadataValue::embedded(s.to_string()));
            album = TagLike::album(tag).map(|s| MetadataValue::embedded(s.to_string()));
            album_artist =
                TagLike::album_artist(tag).map(|s| MetadataValue::embedded(s.to_string()));
            track_number = TagLike::track(tag).map(MetadataValue::embedded);
            disc_number = TagLike::disc(tag).map(MetadataValue::embedded);
            genre = TagLike::genre(tag).map(|s| MetadataValue::embedded(s.to_string()));

            // Get year from tag.date_recorded()
            year = TagLike::date_recorded(tag)
                .and_then(|ts| u32::try_from(ts.year).ok().map(MetadataValue::embedded));
        }

        let fmt_chunk = dsf_file.fmt_chunk();
        let duration: Option<MetadataValue<f64>> = if fmt_chunk.sampling_frequency() > 0 {
            let duration_seconds =
                (fmt_chunk.sample_count() as f64) / (fmt_chunk.sampling_frequency() as f64);
            Some(MetadataValue::embedded(duration_seconds))
        } else {
            None
        };

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
            format: "dsf".to_string(),
            path: path.to_path_buf(),
        }
    }

    /// Extract basic metadata (minimal parsing for performance)
    fn extract_basic_metadata_from_dsf_file(
        &self,
        dsf_file: &DsfFile,
        path: &Path,
    ) -> TrackMetadata {
        let fmt_chunk = dsf_file.fmt_chunk();
        let duration: Option<MetadataValue<f64>> = if fmt_chunk.sampling_frequency() > 0 {
            let duration_seconds =
                (fmt_chunk.sample_count() as f64) / (fmt_chunk.sampling_frequency() as f64);
            Some(MetadataValue::embedded(duration_seconds))
        } else {
            None
        };

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
            format: "dsf".to_string(),
            path: path.to_path_buf(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    // Fixture path for a sample DSF file
    const TEST_DSF_FILE: &str = "tests/fixtures/dsf/simple/1.dsf";

    #[test]
    fn test_dsf_handler_supported_extensions() {
        let handler = DsfHandler::new();
        let extensions = handler.supported_extensions();
        assert_eq!(extensions, vec!["dsf"]);
    }

    #[test]
    fn test_dsf_handler_can_handle() {
        let handler = DsfHandler::new();

        assert!(handler.can_handle(&PathBuf::from("test.dsf")));
        assert!(handler.can_handle(&PathBuf::from("test.DSF")));
        assert!(!handler.can_handle(&PathBuf::from("test.flac")));
        assert!(!handler.can_handle(&PathBuf::from("test.mp3")));
    }

    #[test]
    fn test_dsf_handler_new_creates_instance() {
        let handler = DsfHandler::new();
        assert!(handler.can_handle(&PathBuf::from("test.dsf")));
    }

    #[test]
    fn test_dsf_handler_default_creates_instance() {
        let handler = DsfHandler::default();
        assert!(handler.can_handle(&PathBuf::from("test.dsf")));
    }

    #[test]
    fn test_dsf_handler_read_metadata_unsupported_format() {
        let handler = DsfHandler::new();
        let result = handler.read_metadata(&PathBuf::from("test.mp3"));
        assert!(matches!(result, Err(AudioFileError::UnsupportedFormat)));
    }

    #[test]
    fn test_dsf_handler_write_metadata_unsupported_format() {
        let handler = DsfHandler::new();
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
            format: "dsf".to_string(),
            path: PathBuf::from("test.dsf"),
        };
        let result = handler.write_metadata(&PathBuf::from("test.mp3"), &metadata);
        assert!(matches!(result, Err(AudioFileError::WriteError(_))));
    }

    #[test]
    fn test_dsf_handler_read_basic_info_unsupported_format() {
        let handler = DsfHandler::new();
        let result = handler.read_basic_info(&PathBuf::from("test.mp3"));
        assert!(matches!(result, Err(AudioFileError::UnsupportedFormat)));
    }

    #[test]
    fn test_dsf_handler_read_basic_info_nonexistent_file() {
        let handler = DsfHandler::new();
        let result = handler.read_basic_info(&PathBuf::from("nonexistent.dsf"));
        assert!(matches!(result, Err(AudioFileError::InvalidFile(_))));
    }

    #[test]
    fn test_dsf_handler_read_metadata_nonexistent_file() {
        let handler = DsfHandler::new();
        let result = handler.read_metadata(&PathBuf::from("nonexistent.dsf"));
        assert!(matches!(result, Err(AudioFileError::InvalidFile(_))));
    }

    // New test case for reading metadata from a valid DSF file
    #[test]
    #[ignore = "Use it manually"]
    fn test_dsf_handler_read_metadata_from_fixture() {
        let handler = DsfHandler::new();
        let path = PathBuf::from(TEST_DSF_FILE);
        let track = handler
            .read_metadata(&path)
            .expect("Failed to read metadata from fixture");

        assert_eq!(track.metadata.format, "dsf");
        assert_eq!(track.metadata.path, path);
        // Assert some known metadata from the fixture if available
        // For example, if "1.dsf" has an artist "Test Artist" and title "Test Title"
        // assert_eq!(track.metadata.artist.unwrap().value, "Test Artist");
        // assert_eq!(track.metadata.title.unwrap().value, "Test Title");
        assert!(track.metadata.duration.is_some());
        assert!(track.metadata.duration.unwrap().value > 0.0);
    }

    // Test case for write_metadata, now expecting it to return WriteError
    #[test]
    #[ignore = "Use it manually"]
    fn test_dsf_handler_write_metadata_unsupported() {
        let handler = DsfHandler::new();
        let temp_dir = TempDir::new().unwrap();
        let temp_dsf_path = temp_dir.path().join("temp.dsf");
        fs::copy(TEST_DSF_FILE, &temp_dsf_path).expect("Failed to copy test fixture");

        let metadata = TrackMetadata {
            title: Some(MetadataValue::embedded("New Title".to_string())),
            artist: None,
            album: None,
            album_artist: None,
            track_number: None,
            disc_number: None,
            year: None,
            genre: None,
            duration: None,
            format: "dsf".to_string(),
            path: temp_dsf_path.clone(),
        };

        let result = handler.write_metadata(&temp_dsf_path, &metadata);
        assert!(matches!(result, Err(AudioFileError::WriteError(_))));
        if let Err(AudioFileError::WriteError(msg)) = result {
            assert!(msg.contains("Writing DSF metadata is not supported."));
        }
    }

    // New test case for read_basic_info
    #[test]
    #[ignore = "Use it manually"]
    fn test_dsf_handler_read_basic_info_from_fixture() {
        let handler = DsfHandler::new();
        let path = PathBuf::from(TEST_DSF_FILE);
        let metadata = handler
            .read_basic_info(&path)
            .expect("Failed to read basic info from fixture");

        assert_eq!(metadata.format, "dsf");
        assert_eq!(metadata.path, path);
        assert!(metadata.duration.is_some());
        assert!(metadata.duration.unwrap().value > 0.0);
        // Basic info should not contain specific title/artist if not inferred or explicitly handled
        assert!(
            metadata.title.is_none()
                || metadata.title.unwrap().source
                    == crate::core::domain::models::MetadataSource::FolderInferred
        );
    }
}
