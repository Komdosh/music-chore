//! Domain models and traits for music library operations.

pub mod models;
pub mod schema_version;
pub mod traits;

// Re-export commonly used types
pub use crate::core::services::library::build_library_hierarchy;
pub use models::{
    AlbumNode, ArtistNode, Library, MetadataSource, MetadataValue, OperationResult, Track,
    TrackMetadata, TrackNode,
};
pub use schema_version::{SchemaVersionWrapper, with_schema_version};
pub use traits::{AudioFile, AudioFileError, AudioFileRegistry};
