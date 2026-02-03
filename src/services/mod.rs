//! Business logic services.

pub mod inference;
pub mod normalization;
pub mod scanner;
pub mod library;
pub mod formats;

// Re-export commonly used functions
pub use inference::{infer_album_from_path, infer_artist_from_path};
pub use normalization::{
    normalize_track_titles, normalize_track_titles_with_options, to_title_case,
};
