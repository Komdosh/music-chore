//! Format-agnostic trait for audio file operations.

use crate::domain::models::{Track, TrackMetadata};
use std::path::Path;

/// Errors that can occur during audio file operations
#[derive(Debug, Clone)]
pub enum AudioFileError {
    /// File format is not supported by this handler
    UnsupportedFormat,
    /// File could not be read or parsed
    InvalidFile(String),
    /// I/O error occurred
    IoError(String),
    /// Metadata could not be written
    WriteError(String),
}

impl std::fmt::Display for AudioFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioFileError::UnsupportedFormat => write!(f, "Unsupported file format"),
            AudioFileError::InvalidFile(msg) => write!(f, "Invalid file: {}", msg),
            AudioFileError::IoError(msg) => write!(f, "I/O error: {}", msg),
            AudioFileError::WriteError(msg) => write!(f, "Write error: {}", msg),
        }
    }
}

impl std::error::Error for AudioFileError {}

/// Format-agnostic trait for audio file operations
pub trait AudioFile: Send + Sync {
    /// Check if this handler can process the given file
    fn can_handle(&self, path: &Path) -> bool;

    /// Get the file extensions this handler supports
    fn supported_extensions(&self) -> Vec<&'static str>;

    /// Read metadata from an audio file
    fn read_metadata(&self, path: &Path) -> Result<Track, AudioFileError>;

    /// Write metadata to an audio file
    fn write_metadata(&self, path: &Path, metadata: &TrackMetadata) -> Result<(), AudioFileError>;

    /// Get basic track information without full metadata parsing
    fn read_basic_info(&self, path: &Path) -> Result<TrackMetadata, AudioFileError>;
}

/// Registry for audio file handlers
pub struct AudioFileRegistry {
    handlers: Vec<Box<dyn AudioFile>>,
}

impl AudioFileRegistry {
    /// Create a new registry with default handlers
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Register a new audio file handler
    pub fn register(&mut self, handler: Box<dyn AudioFile>) {
        self.handlers.push(handler);
    }

    /// Find a handler that can process the given file
    pub fn find_handler(&self, path: &Path) -> Result<&dyn AudioFile, AudioFileError> {
        for handler in &self.handlers {
            if handler.can_handle(path) {
                return Ok(handler.as_ref());
            }
        }
        Err(AudioFileError::UnsupportedFormat)
    }

    /// Get all supported file extensions
    pub fn supported_extensions(&self) -> Vec<String> {
        let mut extensions = Vec::new();
        for handler in &self.handlers {
            extensions.extend(
                handler
                    .supported_extensions()
                    .into_iter()
                    .map(|s| s.to_lowercase()),
            );
        }
        extensions.sort();
        extensions.dedup();
        extensions
    }
}

impl Default for AudioFileRegistry {
    fn default() -> Self {
        Self::new()
    }
}
