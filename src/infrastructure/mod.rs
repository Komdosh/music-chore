//! Infrastructure layer - External concerns and format handlers.

pub mod formats;
pub mod scanner;

// Re-export commonly used functions
pub use formats::{get_supported_extensions, is_format_supported, read_metadata, write_metadata};
pub use scanner::{scan_dir, scan_dir_paths, scan_dir_with_metadata};
