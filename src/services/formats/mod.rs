//! Audio format registry and factory.

use crate::domain::traits::{AudioFileError, AudioFileRegistry};
use crate::services::formats::dsf::DsfHandler;
use crate::services::formats::flac::FlacHandler;
use crate::services::formats::mp3::Mp3Handler;
use crate::services::formats::wav::WavHandler;
use crate::services::formats::wavpack::WavPackHandler;
use std::path::Path;

pub mod dsf;
pub mod flac;
pub mod mp3;
pub mod wav;
pub mod wavpack;

/// Create a new audio file registry with all supported format handlers
pub fn create_audio_registry() -> AudioFileRegistry {
    let mut registry = AudioFileRegistry::new();

    // Register FLAC handler
    registry.register(Box::new(FlacHandler::new()));

    // Register MP3 handler
    registry.register(Box::new(Mp3Handler::new()));

    // Register WAV handler
    registry.register(Box::new(WavHandler::new()));

    // Register DSF handler
    registry.register(Box::new(DsfHandler::new()));

    // Register WavPack handler
    registry.register(Box::new(WavPackHandler::new()));

    registry
}

/// Read metadata from a file using the appropriate format handler
pub fn read_metadata(path: &Path) -> Result<crate::domain::models::Track, AudioFileError> {
    let registry = create_audio_registry();
    let handler = registry.find_handler(path)?;
    let track = handler.read_metadata(path)?;

    // Optionally validate metadata schema after reading
    if let Err(validation_error) = crate::services::validation::metadata_validation::validate_track_metadata(&track) {
        eprintln!("Warning: Metadata validation failed for {}: {}", path.display(), validation_error);
    }

    Ok(track)
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
