//! Domain layer - Pure business logic for music metadata handling.
//!
//! This layer contains format-agnostic logic for inference, normalization,
//! and validation of music metadata.

pub mod artist;
pub mod album;
pub mod track;
pub mod metadata;
pub mod inference;
pub mod normalization;
pub mod validation;

// Re-export key types
pub use artist::Artist;
pub use album::Album;
pub use track::Track;
pub use metadata::Metadata;
pub use inference::infer_metadata;
pub use normalization::normalize_metadata;
pub use validation::validate_metadata;