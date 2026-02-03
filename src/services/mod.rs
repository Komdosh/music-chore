//! Business logic services.

pub mod apply_metadata;
pub mod cue;
pub mod duplicates;
pub mod format_tree;
pub mod formats;
pub mod inference;
pub mod library;
pub mod normalization;
pub mod scanner;
pub mod validation;

// Re-export commonly used functions
pub use inference::{infer_album_from_path, infer_artist_from_path};
