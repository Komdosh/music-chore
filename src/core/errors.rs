//! Centralized error types for the music chore application.

use serde::Serialize;
use std::fmt;

/// Main error enum for the music chore application
#[derive(Debug, Clone, Serialize, PartialEq, schemars::JsonSchema)]
pub enum MusicChoreError {
    /// I/O error occurred
    IoError(String),
    /// File format is not supported
    FormatNotSupported(String),
    /// File not found
    FileNotFound(String),
    /// Metadata parsing error
    MetadataParseError(String),
    /// Invalid metadata field
    InvalidMetadataField { field: String, value: String },
    /// Directory access error
    DirectoryAccessError(String),
    /// Permission denied
    PermissionDenied(String),
    /// Invalid path
    InvalidPath(String),
    /// Unsupported audio format
    UnsupportedAudioFormat(String),
    /// Invalid configuration
    InvalidConfiguration(String),
    /// Validation error
    ValidationError(String),
    /// Processing error
    ProcessingError(String),
    /// Conversion error
    ConversionError(String),
    /// Checksum error
    ChecksumError(String),
    /// CUE file error
    CueFileError(String),
    /// Other error
    Other(String),
}

impl fmt::Display for MusicChoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MusicChoreError::IoError(msg) => write!(f, "I/O error: {}", msg),
            MusicChoreError::FormatNotSupported(msg) => write!(f, "Format not supported: {}", msg),
            MusicChoreError::FileNotFound(msg) => write!(f, "File not found: {}", msg),
            MusicChoreError::MetadataParseError(msg) => {
                write!(f, "Metadata parsing error: {}", msg)
            }
            MusicChoreError::InvalidMetadataField { field, value } => {
                write!(f, "Invalid value '{}' for field '{}'", value, field)
            }
            MusicChoreError::DirectoryAccessError(msg) => {
                write!(f, "Directory access error: {}", msg)
            }
            MusicChoreError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            MusicChoreError::InvalidPath(msg) => write!(f, "Invalid path: {}", msg),
            MusicChoreError::UnsupportedAudioFormat(msg) => {
                write!(f, "Unsupported audio format: {}", msg)
            }
            MusicChoreError::InvalidConfiguration(msg) => {
                write!(f, "Invalid configuration: {}", msg)
            }
            MusicChoreError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            MusicChoreError::ProcessingError(msg) => write!(f, "Processing error: {}", msg),
            MusicChoreError::ConversionError(msg) => write!(f, "Conversion error: {}", msg),
            MusicChoreError::ChecksumError(msg) => write!(f, "Checksum error: {}", msg),
            MusicChoreError::CueFileError(msg) => write!(f, "CUE file error: {}", msg),
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

impl From<std::str::Utf8Error> for MusicChoreError {
    fn from(error: std::str::Utf8Error) -> Self {
        MusicChoreError::Other(format!("UTF-8 error: {}", error))
    }
}

impl From<std::num::ParseIntError> for MusicChoreError {
    fn from(error: std::num::ParseIntError) -> Self {
        MusicChoreError::Other(format!("Integer parsing error: {}", error))
    }
}

impl From<std::num::ParseFloatError> for MusicChoreError {
    fn from(error: std::num::ParseFloatError) -> Self {
        MusicChoreError::Other(format!("Float parsing error: {}", error))
    }
}
