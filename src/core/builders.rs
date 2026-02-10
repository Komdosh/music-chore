//! Builder patterns for complex objects in the music chore application.

use crate::core::domain::models::{MetadataSource, MetadataValue, TrackMetadata};
use std::path::PathBuf;

/// Builder for TrackMetadata to facilitate easy construction of metadata objects
#[derive(Debug, Clone)]
pub struct TrackMetadataBuilder {
    title: Option<MetadataValue<String>>,
    artist: Option<MetadataValue<String>>,
    album: Option<MetadataValue<String>>,
    album_artist: Option<MetadataValue<String>>,
    track_number: Option<MetadataValue<u32>>,
    disc_number: Option<MetadataValue<u32>>,
    year: Option<MetadataValue<u32>>,
    genre: Option<MetadataValue<String>>,
    duration: Option<MetadataValue<f64>>,
    format: String,
    path: PathBuf,
}

impl TrackMetadataBuilder {
    /// Create a new builder with default values
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            title: None,
            artist: None,
            album: None,
            album_artist: None,
            track_number: None,
            disc_number: None,
            year: None,
            genre: None,
            duration: None,
            format: "unknown".to_string(),
            path: path.into(),
        }
    }

    /// Set the title metadata
    pub fn title<V: Into<String>>(
        mut self,
        value: V,
        source: MetadataSource,
        confidence: f32,
    ) -> Self {
        self.title = Some(MetadataValue {
            value: value.into(),
            source,
            confidence,
        });
        self
    }

    /// Set the artist metadata
    pub fn artist<V: Into<String>>(
        mut self,
        value: V,
        source: MetadataSource,
        confidence: f32,
    ) -> Self {
        self.artist = Some(MetadataValue {
            value: value.into(),
            source,
            confidence,
        });
        self
    }

    /// Set the album metadata
    pub fn album<V: Into<String>>(
        mut self,
        value: V,
        source: MetadataSource,
        confidence: f32,
    ) -> Self {
        self.album = Some(MetadataValue {
            value: value.into(),
            source,
            confidence,
        });
        self
    }

    /// Set the album artist metadata
    pub fn album_artist<V: Into<String>>(
        mut self,
        value: V,
        source: MetadataSource,
        confidence: f32,
    ) -> Self {
        self.album_artist = Some(MetadataValue {
            value: value.into(),
            source,
            confidence,
        });
        self
    }

    /// Set the track number metadata
    pub fn track_number(mut self, value: u32, source: MetadataSource, confidence: f32) -> Self {
        self.track_number = Some(MetadataValue {
            value,
            source,
            confidence,
        });
        self
    }

    /// Set the disc number metadata
    pub fn disc_number(mut self, value: u32, source: MetadataSource, confidence: f32) -> Self {
        self.disc_number = Some(MetadataValue {
            value,
            source,
            confidence,
        });
        self
    }

    /// Set the year metadata
    pub fn year(mut self, value: u32, source: MetadataSource, confidence: f32) -> Self {
        self.year = Some(MetadataValue {
            value,
            source,
            confidence,
        });
        self
    }

    /// Set the genre metadata
    pub fn genre<V: Into<String>>(
        mut self,
        value: V,
        source: MetadataSource,
        confidence: f32,
    ) -> Self {
        self.genre = Some(MetadataValue {
            value: value.into(),
            source,
            confidence,
        });
        self
    }

    /// Set the duration metadata
    pub fn duration(mut self, value: f64, source: MetadataSource, confidence: f32) -> Self {
        self.duration = Some(MetadataValue {
            value,
            source,
            confidence,
        });
        self
    }

    /// Set the format
    pub fn format<V: Into<String>>(mut self, value: V) -> Self {
        self.format = value.into();
        self
    }

    /// Build the TrackMetadata instance
    pub fn build(self) -> TrackMetadata {
        TrackMetadata {
            title: self.title,
            artist: self.artist,
            album: self.album,
            album_artist: self.album_artist,
            track_number: self.track_number,
            disc_number: self.disc_number,
            year: self.year,
            genre: self.genre,
            duration: self.duration,
            format: self.format,
            path: self.path,
        }
    }
}

impl Default for TrackMetadataBuilder {
    fn default() -> Self {
        Self::new(PathBuf::new())
    }
}
