//! Music Chore - Modular, format-agnostic music metadata tool.

// Public module declarations
pub mod cli;
pub mod domain;
pub mod mcp;
pub mod services;

// Re-export commonly used types and functions for backwards compatibility
pub use domain::{
    AlbumNode, ArtistNode, Library, MetadataSource, MetadataValue, OperationResult, Track,
    TrackMetadata, TrackNode, build_library_hierarchy,
};

pub use services::{infer_album_from_path, infer_artist_from_path, normalization::to_title_case};
