//! Audio format registry and factory.
use crate::adapters::audio_formats::dsf::DsfHandler;
use crate::adapters::audio_formats::flac::FlacHandler;
use crate::adapters::audio_formats::mp3::Mp3Handler;
use crate::adapters::audio_formats::wav::WavHandler;
use crate::adapters::audio_formats::wavpack::WavPackHandler;
use crate::core::domain::models::{MetadataValue, TrackMetadata};
#[allow(unused_imports)]
use crate::core::domain::traits::{AudioFileError, AudioFileRegistry};
use std::path::Path;

pub mod dsf;
pub mod flac;
pub mod mp3;
pub mod wav;
pub mod wavpack;

/// Basic audio information extracted for CUE file processing.
#[derive(Debug, Clone)]
pub struct BasicAudioInfo {
    pub duration: Option<MetadataValue<f64>>,
    pub format: String,
}

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
pub fn read_metadata(path: &Path) -> Result<crate::core::domain::models::Track, AudioFileError> {
    let registry = create_audio_registry();
    let handler = registry.find_handler(path)?;
    let track = handler.read_metadata(path)?;

    // NOTE: We don't validate metadata schema during normal read operations
    // to avoid side effects. Validation should be done explicitly by calling
    // the validation functions when needed.

    Ok(track)
}

/// Read basic metadata (duration, format) from a file.
/// This is used primarily for CUE sheet processing where full metadata is not needed.
pub fn read_basic_info(path: &Path) -> Result<BasicAudioInfo, AudioFileError> {
    let registry = create_audio_registry();
    let handler = registry.find_handler(path)?;
    let track = handler.read_metadata(path)?; // Temporarily read full metadata

    Ok(BasicAudioInfo {
        duration: track.metadata.duration,
        format: track.metadata.format,
    })
}

/// Write metadata to a file using the appropriate format handler
pub fn write_metadata(path: &Path, metadata: &TrackMetadata) -> Result<(), AudioFileError> {
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
