//! Centralized error types for the music chore application.

use serde::Serialize;
use std::fmt;

/// Main error enum for the music chore application
#[derive(Debug, Clone, Serialize)]
pub enum MusicChoreError {
    /// File format is not supported by this handler
    UnsupportedFormat(String),
    /// File could not be read or parsed
    InvalidFile(String),
    /// I/O error occurred
    IoError(String),
    /// Metadata could not be written
    WriteError(String),
    /// Missing required field
    MissingRequiredField(String),
    /// Invalid value for a field
    InvalidValue(String, String),
    /// Format mismatch for a field
    FormatMismatch(String, String),
    /// Configuration error
    ConfigError(String),
    /// Validation error
    ValidationError(String),
    /// Scan error
    ScanError(String),
    /// Other error
    Other(String),
}

impl fmt::Display for MusicChoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MusicChoreError::UnsupportedFormat(msg) => write!(f, "Unsupported file format: {}", msg),
            MusicChoreError::InvalidFile(msg) => write!(f, "Invalid file: {}", msg),
            MusicChoreError::IoError(msg) => write!(f, "I/O error: {}", msg),
            MusicChoreError::WriteError(msg) => write!(f, "Write error: {}", msg),
            MusicChoreError::MissingRequiredField(field) => write!(f, "Missing required field: {}", field),
            MusicChoreError::InvalidValue(field, value) => write!(f, "Invalid value '{}' for field '{}'", value, field),
            MusicChoreError::FormatMismatch(field, format) => write!(f, "Format mismatch for field '{}': {}", field, format),
            MusicChoreError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            MusicChoreError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            MusicChoreError::ScanError(msg) => write!(f, "Scan error: {}", msg),
            MusicChoreError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for MusicChoreError {}

impl From<std::io::Error> for MusicChoreError {
    fn from(error: std::io::Error) -> Self {
        MusicChoreError::IoError(error.to_string())
    }
}

impl From<serde_json::Error> for MusicChoreError {
    fn from(error: serde_json::Error) -> Self {
        MusicChoreError::Other(format!("JSON error: {}", error))
    }
}