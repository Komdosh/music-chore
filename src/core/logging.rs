//! Logging utilities for the music chore application.

use log::{Level, LevelFilter, Log, Metadata, Record};
use std::sync::Mutex;

/// Initialize application logging with the specified level
pub fn init_logging(level: LevelFilter) {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(level.to_string()))
        .init();
}

/// Initialize application logging with custom format
pub fn init_logging_with_format<F>(level: LevelFilter, format_fn: F)
where
    F: Fn(&mut env_logger::fmt::Formatter, &Record) -> std::io::Result<()> + Send + Sync + 'static,
{
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(level.to_string()))
        .format(format_fn)
        .init();
}

/// Log a scan operation
pub fn log_scan_operation(path: &std::path::Path, file_count: usize) {
    log::info!("Scanned {} files from {}", file_count, path.display());
}

/// Log a metadata read operation
pub fn log_metadata_read(file_path: &std::path::Path, success: bool) {
    if success {
        log::debug!("Successfully read metadata from {}", file_path.display());
    } else {
        log::warn!("Failed to read metadata from {}", file_path.display());
    }
}

/// Log a metadata write operation
pub fn log_metadata_write(file_path: &std::path::Path, success: bool) {
    if success {
        log::info!("Successfully updated metadata for {}", file_path.display());
    } else {
        log::error!("Failed to update metadata for {}", file_path.display());
    }
}

/// Log a normalization operation
pub fn log_normalization_operation(path: &std::path::Path, changes_count: usize) {
    if changes_count > 0 {
        log::info!(
            "Normalized {} metadata fields in {}",
            changes_count,
            path.display()
        );
    } else {
        log::debug!("No metadata changes needed for {}", path.display());
    }
}

/// Log a validation operation
pub fn log_validation_operation(
    path: &std::path::Path,
    errors_count: usize,
    warnings_count: usize,
) {
    if errors_count > 0 || warnings_count > 0 {
        log::warn!(
            "Validation for {}: {} errors, {} warnings",
            path.display(),
            errors_count,
            warnings_count
        );
    } else {
        log::info!("Validation passed for {}", path.display());
    }
}

/// Log a duplicate detection operation
pub fn log_duplicate_detection(path: &std::path::Path, duplicates_found: usize) {
    if duplicates_found > 0 {
        log::warn!(
            "Found {} duplicate files in {}",
            duplicates_found,
            path.display()
        );
    } else {
        log::info!("No duplicates found in {}", path.display());
    }
}

/// Log a CUE file operation
pub fn log_cue_operation(path: &std::path::Path, operation: &str, success: bool) {
    if success {
        log::info!(
            "CUE {} operation successful for {}",
            operation,
            path.display()
        );
    } else {
        log::error!("CUE {} operation failed for {}", operation, path.display());
    }
}

/// Log an error with context
pub fn log_error_with_context(context: &str, error: &dyn std::error::Error) {
    log::error!("Error in {}: {}", context, error);
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::LevelFilter;
    use std::path::Path;

    #[test]
    fn test_logging_functions_no_panic() {
        // These tests ensure that the formatting logic within the logging functions
        // works correctly with various inputs and doesn't cause panics.

        let path = Path::new("tests/fixtures/flac/simple/track1.flac");

        log_scan_operation(path, 42);

        log_metadata_read(path, true);
        log_metadata_read(path, false);

        log_metadata_write(path, true);
        log_metadata_write(path, false);

        log_normalization_operation(path, 5);
        log_normalization_operation(path, 0);

        log_validation_operation(path, 2, 1);
        log_validation_operation(path, 0, 0);

        log_duplicate_detection(path, 3);
        log_duplicate_detection(path, 0);

        log_cue_operation(path, "generate", true);
        log_cue_operation(path, "validate", false);

        let err = std::io::Error::new(std::io::ErrorKind::Other, "custom error");
        log_error_with_context("test context", &err);
    }

    #[test]
    fn test_init_logging_minimal() {
        // Note: We can't easily test init_logging multiple times as it can only be
        // initialized once per process. This test is primarily for coverage of the
        // builder setup code. We use a level that's unlikely to conflict.
        let _ = env_logger::Builder::new()
            .filter_level(LevelFilter::Info)
            .try_init();
    }

    #[test]
    fn test_init_logging_with_format_minimal() {
        let _ = env_logger::Builder::new()
            .format(|buf, record| {
                use std::io::Write;
                writeln!(buf, "{}: {}", record.level(), record.args())
            })
            .filter_level(LevelFilter::Info)
            .try_init();
    }
}
