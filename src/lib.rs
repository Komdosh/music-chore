//! Music Chore - Modular, format-agnostic music metadata tool.

// Public module declarations
pub mod cli;
pub mod domain;
pub mod mcp;
pub mod services;

// Re-export commonly used types and functions for backwards compatibility
pub use domain::{
    build_library_hierarchy, AlbumNode, ArtistNode, Library, MetadataSource, MetadataValue,
    OperationResult, Track, TrackMetadata, TrackNode,
};

pub use services::{
    infer_album_from_path, infer_artist_from_path, normalization::to_title_case,
};
