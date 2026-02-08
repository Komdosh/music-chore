//! Music Chore - Modular, format-agnostic music metadata tool.

// Public module declarations
pub mod core {
    pub mod domain;
    pub mod services;
    pub mod errors;
    pub mod config;  // This is the old config module
    pub mod configuration;  // This is the new comprehensive configuration system
    pub mod logging;  // This is the new logging module
    pub mod types;
    pub mod builders;
}
pub mod adapters;
pub mod presentation;
pub mod mcp;

// Re-export commonly used types and functions for backwards compatibility
pub use core::domain::{
    AlbumNode, ArtistNode, Library, MetadataSource, MetadataValue, OperationResult, Track,
    TrackMetadata, TrackNode, build_library_hierarchy,
};

pub use core::services::{infer_album_from_path, infer_artist_from_path, normalization::to_title_case};
pub use core::errors::MusicChoreError;
