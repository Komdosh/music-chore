use crate::services::formats::read_metadata;
use crate::services::scanner::scan_dir;
use serde_json::to_string_pretty;
use std::path::PathBuf;

#[derive(Debug, serde::Serialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub summary: ValidationSummary,
}

#[derive(Debug, serde::Serialize)]
pub struct ValidationError {
    pub file_path: String,
    pub field: String,
    pub message: String,
}

#[derive(Debug, serde::Serialize)]
pub struct ValidationWarning {
    pub file_path: String,
    pub field: String,
    pub message: String,
}

#[derive(Debug, serde::Serialize)]
pub struct ValidationSummary {
    pub total_files: usize,
    pub valid_files: usize,
    pub files_with_errors: usize,
    pub files_with_warnings: usize,
}
pub fn validate_path(path: &PathBuf, json: bool) -> Result<String, String> {
    let tracks = scan_dir(path);
    let total_scanned = tracks.len();

    if tracks.is_empty() {
        return Err(if json {
            "{\"valid\": true, \"errors\": [], \"warnings\": [], \"summary\": {\"total_files\": 0, \"valid_files\": 0, \"files_with_errors\": 0, \"files_with_warnings\": 0}}".to_string()
        } else {
            "No music files found to validate.".to_string()
        });
    }

    // Read metadata for validation
    let tracks_with_metadata: Vec<crate::Track> = tracks
        .into_iter()
        .filter_map(|track| read_metadata(&track.file_path).ok())
        .collect();

    if tracks_with_metadata.is_empty() {
        return Err(if json {
            format!(
                "{{\"valid\": false, \"errors\": [], \"warnings\": [], \"summary\": {{\"total_files\": {}, \"valid_files\": 0, \"files_with_errors\": {}, \"files_with_warnings\": 0}}}}",
                total_scanned, total_scanned
            )
        } else {
            "Unable to read metadata from any files for validation.".to_string()
        });
    }

    let validation_results = validate_tracks(tracks_with_metadata);

    let result = if json {
        to_string_pretty(&validation_results)
            .unwrap_or_else(|e| format!("Error serializing validation results: {}", e))
    } else {
        build_validation_results(&validation_results)
    };
    Ok(result)
}

/// Print validation results in human-readable format
fn build_validation_results(results: &ValidationResult) -> String {
    let mut output = String::new();

    output.push_str("=== METADATA VALIDATION RESULTS ===\n\n");

    output.push_str("📊 Summary:\n");
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
            "❌ Validation failed with {} errors\n",
            results.errors.len()
        ));
    }

    if !results.errors.is_empty() {
        output.push_str("\n🔴 ERRORS:\n");
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
            output.push_str(&format!("  Issue: {}\n", warning.message));
        }
    }

    output.push_str("=== END VALIDATION ===\n");

    output
}

/// Validate tracks for missing required fields and consistency issues
pub fn validate_tracks(tracks: Vec<crate::Track>) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut files_with_errors = std::collections::HashSet::new();
    let mut files_with_warnings = std::collections::HashSet::new();

    for track in &tracks {
        let file_path = track.file_path.to_string_lossy().to_string();
        let mut has_error = false;
        let mut has_warning = false;

        // Required fields: title, artist, album
        if track.metadata.title.is_none() {
            errors.push(ValidationError {
                file_path: file_path.clone(),
                field: "title".to_string(),
                message: "Missing required field: title".to_string(),
            });
            has_error = true;
        }

        if track.metadata.artist.is_none() {
            errors.push(ValidationError {
                file_path: file_path.clone(),
                field: "artist".to_string(),
                message: "Missing required field: artist".to_string(),
            });
            has_error = true;
        }

        if track.metadata.album.is_none() {
            errors.push(ValidationError {
                file_path: file_path.clone(),
                field: "album".to_string(),
                message: "Missing required field: album".to_string(),
            });
            has_error = true;
        }

        // Check for empty or whitespace-only fields
        if let Some(ref title) = track.metadata.title {
            if title.value.trim().is_empty() {
                errors.push(ValidationError {
                    file_path: file_path.clone(),
                    field: "title".to_string(),
                    message: "Title field is empty".to_string(),
                });
                has_error = true;
            }
        }

        if let Some(ref artist) = track.metadata.artist {
            if artist.value.trim().is_empty() {
                errors.push(ValidationError {
                    file_path: file_path.clone(),
                    field: "artist".to_string(),
                    message: "Artist field is empty".to_string(),
                });
                has_error = true;
            }
        }

        if let Some(ref album) = track.metadata.album {
            if album.value.trim().is_empty() {
                errors.push(ValidationError {
                    file_path: file_path.clone(),
                    field: "album".to_string(),
                    message: "Album field is empty".to_string(),
                });
                has_error = true;
            }
        }

        // Warnings for recommended fields
        if track.metadata.track_number.is_none() {
            warnings.push(ValidationWarning {
                file_path: file_path.clone(),
                field: "track_number".to_string(),
                message: "Missing recommended field: track_number".to_string(),
            });
            has_warning = true;
        }

        if track.metadata.year.is_none() {
            warnings.push(ValidationWarning {
                file_path: file_path.clone(),
                field: "year".to_string(),
                message: "Missing recommended field: year".to_string(),
            });
            has_warning = true;
        }

        // Check for reasonable year ranges
        if let Some(ref year) = track.metadata.year {
            if year.value < 1900 || year.value > 2100 {
                warnings.push(ValidationWarning {
                    file_path: file_path.clone(),
                    field: "year".to_string(),
                    message: format!("Year {} seems unusual (expected 1900-2100)", year.value),
                });
                has_warning = true;
            }
        }

        // Check for reasonable track numbers
        if let Some(ref track_number) = track.metadata.track_number {
            if track_number.value == 0 || track_number.value > 99 {
                warnings.push(ValidationWarning {
                    file_path: file_path.clone(),
                    field: "track_number".to_string(),
                    message: format!(
                        "Track number {} seems unusual (expected 1-99)",
                        track_number.value
                    ),
                });
                has_warning = true;
            }
        }

        // Check for very long titles
        if let Some(ref title) = track.metadata.title {
            if title.value.len() > 200 {
                warnings.push(ValidationWarning {
                    file_path: file_path.clone(),
                    field: "title".to_string(),
                    message: format!("Title is very long ({} characters)", title.value.len()),
                });
                has_warning = true;
            }
        }

        if has_error {
            files_with_errors.insert(file_path.clone());
        }
        if has_warning {
            files_with_warnings.insert(file_path);
        }
    }

    let total_files = tracks.len();
    let valid_files = total_files - files_with_errors.len();
    let summary = ValidationSummary {
        total_files,
        valid_files,
        files_with_errors: files_with_errors.len(),
        files_with_warnings: files_with_warnings.len(),
    };

    ValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
        summary,
    }
}
