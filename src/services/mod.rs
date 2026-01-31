//! Business logic services.

pub mod inference;
pub mod normalization;

// Re-export commonly used functions
pub use inference::{infer_album_from_path, infer_artist_from_path};
pub use normalization::{normalize_track_titles, to_title_case};
