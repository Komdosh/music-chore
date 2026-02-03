//! Audio format registry and factory.

use crate::domain::traits::{AudioFileError, AudioFileRegistry};
use crate::services::formats::flac::FlacHandler;
use std::path::Path;

pub mod flac;

/// Create a new audio file registry with all supported format handlers
pub fn create_audio_registry() -> AudioFileRegistry {
    let mut registry = AudioFileRegistry::new();

    // Register FLAC handler
    registry.register(Box::new(FlacHandler::new()));

    // Future formats will be registered here:
    // registry.register(Box::new(Mp3Handler::new()));
    // registry.register(Box::new(WavHandler::new()));
    // registry.register(Box::new(DsfHandler::new()));

    registry
}

/// Read metadata from a file using the appropriate format handler
pub fn read_metadata(path: &Path) -> Result<crate::domain::models::Track, AudioFileError> {
    let registry = create_audio_registry();
    let handler = registry.find_handler(path)?;
    handler.read_metadata(path)
}

/// Write metadata to a file using the appropriate format handler
pub fn write_metadata(
    path: &Path,
    metadata: &crate::domain::models::TrackMetadata,
) -> Result<(), AudioFileError> {
    let registry = create_audio_registry();
    let handler = registry.find_handler(path)?;
    handler.write_metadata(path, metadata)
}

/// Check if a file format is supported
pub fn is_format_supported(path: &Path) -> bool {
    let registry = create_audio_registry();
    registry.find_handler(path).is_ok()
}

/// Get all supported file extensions
pub fn get_supported_extensions() -> Vec<String> {
    let registry = create_audio_registry();
    registry.supported_extensions()
}
