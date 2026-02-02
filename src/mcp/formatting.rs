//! MCP-specific output formatting functions
//!
//! These functions format CLI results for MCP consumption while maintaining
//! the same output format and behavior as the CLI tools.

use crate::{
    cli::commands::ValidationResult,
    Library,
};

/// Format CLI validation results for MCP output (compatible with existing CLI format)
pub fn format_validation_results(results: &ValidationResult) -> String {
    let mut output = String::new();

    output.push_str("=== MUSIC LIBRARY VALIDATION ===\n");
    output.push_str(&format!("📊 Summary:\n"));
    output.push_str(&format!("  Total files: {}\n", results.summary.total_files));
    output.push_str(&format!("  Valid files: {}\n", results.summary.valid_files));
    output.push_str(&format!(
        "  Files with errors: {}\n",
        results.summary.files_with_errors
    ));
    output.push_str(&format!(
        "  Files with warnings: {}\n\n",
        results.summary.files_with_warnings
    ));

    if results.valid {
        output.push_str("✅ All files passed validation!\n");
    } else {
        output.push_str(&format!(
            "❌ Validation failed with {} errors\n\n",
            results.errors.len()
        ));
    }

    if !results.errors.is_empty() {
        output.push_str("🔴 ERRORS:\n");
        for error in &results.errors {
            output.push_str(&format!("  File: {}\n", error.file_path));
            output.push_str(&format!("  Field: {}\n", error.field));
            output.push_str(&format!("  Issue: {}\n\n", error.message));
        }
    }

    if !results.warnings.is_empty() {
        output.push_str("🟡 WARNINGS:\n");
        for warning in &results.warnings {
            output.push_str(&format!("  File: {}\n", warning.file_path));
            output.push_str(&format!("  Field: {}\n", warning.field));
            output.push_str(&format!("  Issue: {}\n\n", warning.message));
        }
    }

    output.push_str("=== END VALIDATION ===\n");
    output
}

/// Format library tree for MCP output (using CLI logic)
pub fn format_tree_output(library: &Library) -> String {
    let mut output = String::new();

    for artist in &library.artists {
        output.push_str(&format!("📁 {}\n", artist.name));

        for album in &artist.albums {
            let year_str = album.year.map(|y| format!(" ({})", y)).unwrap_or_default();
            output.push_str(&format!("├── 📂 {}{}\n", album.title, year_str));

            for (i, track) in album.tracks.iter().enumerate() {
                let is_last = i == album.tracks.len() - 1;
                let prefix = if is_last {
                    "└─── 🎵"
                } else {
                    "├─── 🎵"
                };

                let track_info = format_track_info(track);
                output.push_str(&format!(
                    "{}   {} {}\n",
                    prefix,
                    track
                        .file_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy(),
                    track_info
                ));
            }
        }
        output.push('\n');
    }

    // Print summary
    output.push_str("📊 Library Summary:\n");
    output.push_str(&format!("   Artists: {}\n", library.total_artists));
    output.push_str(&format!("   Albums: {}\n", library.total_albums));
    output.push_str(&format!("   Tracks: {}\n", library.total_tracks));

    output
}

/// Format track information for tree display
fn format_track_info(track: &crate::TrackNode) -> String {
    let mut info = Vec::new();

    if let Some(duration) = &track.metadata.duration {
        let minutes = (duration.value / 60.0) as u32;
        let seconds = (duration.value % 60.0) as u32;
        info.push(format!("{}:{:02}", minutes, seconds));
    }

    if let Some(track_number) = &track.metadata.track_number {
        info.push(format!("#{}", track_number.value));
    }

    if let Some(format_str) = track.metadata.format.strip_prefix(".") {
        info.push(format_str.to_uppercase());
    } else {
        info.push(track.metadata.format.to_uppercase());
    }

    let source = match track
        .metadata
        .title
        .as_ref()
        .map(|t| &t.source)
        .unwrap_or(&crate::MetadataSource::FolderInferred)
    {
        crate::MetadataSource::Embedded => "🎯",
        crate::MetadataSource::FolderInferred => "🤖",
        crate::MetadataSource::UserEdited => "👤",
    };

    format!("[{}] {}", source, info.join(" | "))
}
