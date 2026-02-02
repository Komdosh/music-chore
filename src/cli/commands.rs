//! CLI command definitions and handlers.

use crate::{
    build_library_hierarchy, normalize_track_titles, read_metadata, scan_dir, Library,
    OperationResult, TrackNode,
};
use clap::{Parser, Subcommand};
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

#[derive(Parser)]
#[command(name = "musicctl")]
#[command(about = "Deterministic, AI‑friendly music metadata compiler.")]
#[command(
    long_about = "A CLI tool for organizing and normalizing local music libraries using existing file metadata and directory structure only.\n\nSupported audio formats: .flac"
)]
#[command(disable_version_flag = true)]
pub struct Cli {
    /// Show version information
    #[arg(short = 'v', long = "version")]
    pub version: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Recursively scan a directory for music files.
    ///
    /// Currently supports: .flac files
    Scan {
        /// Base directory to scan.
        path: PathBuf,
        /// Output JSON instead of a simple tree.
        #[arg(long)]
        json: bool,
    },
    /// Show a human‑friendly tree view.
    Tree {
        path: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Read metadata from a single file.
    Read {
        /// Path to the file.
        file: PathBuf,
    },
    /// Write metadata to a file.
    Write {
        file: PathBuf,
        #[arg(long, num_args = 1..)]
        set: Vec<String>,
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        dry_run: bool,
    },
    /// Normalize track titles to title case.
    Normalize {
        path: PathBuf,
        #[arg(long)]
        dry_run: bool,
    },
    /// Emit library metadata in structured JSON format.
    Emit {
        path: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Validate metadata completeness and consistency.
    Validate {
        /// Base directory to validate.
        path: PathBuf,
        /// Output JSON instead of human-readable format.
        #[arg(long)]
        json: bool,
    },
}

/// Handle the parsed CLI command
pub fn handle_command(command: Commands) -> Result<(), i32> {
    match command {
        Commands::Scan { path, json } => {
            handle_scan(path, json);
            Ok(())
        }
        Commands::Tree { path, json } => {
            handle_tree(path, json);
            Ok(())
        }
        Commands::Read { file } => {
            handle_read(file);
            Ok(())
        }
        Commands::Write {
            file,
            set,
            apply,
            dry_run,
        } => handle_write(file, set, apply, dry_run),
        Commands::Normalize { path, dry_run } => {
            handle_normalize(path, dry_run);
            Ok(())
        }
        Commands::Emit { path, json } => {
            handle_emit(path, json);
            Ok(())
        }
        Commands::Validate { path, json } => {
            println!("{}", handle_validate(path, json));
            Ok(())
        }
    }
}

/// Handle scan command
pub fn handle_scan(path: PathBuf, json: bool) {
    let tracks = scan_dir(&path);
    if tracks.is_empty() {
        println!("No music files found in directory: {}", path.display());
        return;
    }

    if json {
        match to_string_pretty(&tracks) {
            Ok(s) => println!("{}", s),
            Err(e) => eprintln!("Error serializing to JSON: {}", e),
        }
    } else {
        for track in tracks {
            println!("{}", track.file_path.display());
        }
    }
}

/// Handle tree command
pub fn handle_tree(path: PathBuf, json: bool) {
    let tracks = scan_dir(&path);
    let library = build_library_hierarchy(tracks);

    if json {
        match to_string_pretty(&library) {
            Ok(s) => println!("{}", s),
            Err(e) => eprintln!("Error serializing to JSON: {}", e),
        }
    } else {
        print_tree(&library);
    }
}

/// Handle read command
pub fn handle_read(file: PathBuf) {
    match read_metadata(&file) {
        Ok(track) => match to_string_pretty(&track) {
            Ok(s) => println!("{}", s),
            Err(e) => eprintln!("Error serializing track: {}", e),
        },
        Err(e) => eprintln!("Error reading metadata: {}", e),
    }
}

/// Handle write command
pub fn handle_write(
    file: PathBuf,
    set: Vec<String>,
    apply: bool,
    dry_run: bool,
) -> Result<(), i32> {
    if apply && dry_run {
        eprintln!("Error: Cannot use both --apply and --dry-run flags simultaneously");
        return Err(1);
    }

    if !apply && !dry_run {
        eprintln!("Error: Must specify either --apply or --dry-run flag");
        return Err(1);
    }

    // Check if file exists and is supported
    if !file.exists() {
        eprintln!("Error: File does not exist: {}", file.display());
        return Err(1);
    }

    if !crate::infrastructure::is_format_supported(&file) {
        eprintln!("Error: Unsupported file format: {}", file.display());
        return Err(1);
    }

    // Read current metadata
    let mut track = match crate::read_metadata(&file) {
        Ok(track) => track,
        Err(e) => {
            eprintln!("Error reading metadata: {}", e);
            return Err(1);
        }
    };

    // Parse and apply metadata updates
    for metadata_item in set {
        if let Some((key, value)) = metadata_item.split_once('=') {
            match apply_metadata_update(&mut track.metadata, key.trim(), value.trim()) {
                Ok(()) => {
                    if dry_run {
                        println!("DRY RUN: Would set {} = {}", key.trim(), value.trim());
                    }
                }
                Err(e) => {
                    eprintln!("Error parsing metadata '{}': {}", metadata_item, e);
                    return Err(1);
                }
            }
        } else {
            eprintln!(
                "Error: Invalid metadata format '{}'. Expected 'key=value'",
                metadata_item
            );
            return Err(1);
        }
    }

    if dry_run {
        println!("DRY RUN: No changes made to file: {}", file.display());
        return Ok(());
    }

    // Apply changes to file
    match crate::infrastructure::write_metadata(&file, &track.metadata) {
        Ok(()) => {
            println!("Successfully updated metadata in: {}", file.display());
            Ok(())
        }
        Err(e) => {
            eprintln!("Error writing metadata: {}", e);
            Err(1)
        }
    }
}

/// Handle normalize command
pub fn handle_normalize(path: PathBuf, dry_run: bool) {
    match normalize_track_titles(&path) {
        Ok(results) => {
            for result in results {
                match result {
                    OperationResult::Updated {
                        track,
                        old_title,
                        new_title,
                    } => {
                        if dry_run {
                            println!(
                                "DRY RUN: Would normalize '{}' -> '{}' in {}",
                                track.file_path.display(),
                                old_title,
                                new_title
                            );
                        } else {
                            println!(
                                "NORMALIZED: '{}' -> '{}' in {}",
                                track.file_path.display(),
                                old_title,
                                new_title
                            );
                        }
                    }
                    OperationResult::NoChange { track } => {
                        if !dry_run {
                            println!(
                                "NO CHANGE: Title already title case in {}",
                                track.file_path.display()
                            );
                        }
                    }
                    OperationResult::Error { track, error } => {
                        eprintln!("ERROR: {} in {}", error, track.file_path.display());
                    }
                }
            }
        }
        Err(e) => eprintln!("Error normalizing titles: {}", e),
    }
}

/// Handle emit command
pub fn handle_emit(path: PathBuf, json: bool) {
    let tracks = scan_dir(&path);
    let library = build_library_hierarchy(tracks);

    if json {
        match to_string_pretty(&library) {
            Ok(s) => println!("{}", s),
            Err(e) => eprintln!("Error serializing to JSON: {}", e),
        }
    } else {
        // Default to structured text output for AI agents
        emit_structured_output(&library);
    }
}

/// Emit structured output optimized for AI agents
pub fn emit_structured_output(library: &Library) {
    println!("=== MUSIC LIBRARY METADATA ===");
    println!("Total Artists: {}", library.total_artists);
    println!("Total Albums: {}", library.total_albums);
    println!("Total Tracks: {}", library.total_tracks);
    println!();

    for artist in &library.artists {
        println!("ARTIST: {}", artist.name);

        for album in &artist.albums {
            let year_str = album.year.map(|y| format!(" ({})", y)).unwrap_or_default();
            println!("  ALBUM: {}{}", album.title, year_str);

            for track in &album.tracks {
                let title = track
                    .metadata
                    .title
                    .as_ref()
                    .map(|t| t.value.as_str())
                    .unwrap_or("[Unknown Title]");
                let duration = track
                    .metadata
                    .duration
                    .as_ref()
                    .map(|d| {
                        let total_seconds = d.value as u64;
                        let minutes = total_seconds / 60;
                        let seconds = total_seconds % 60;
                        format!("{}:{:02}", minutes, seconds)
                    })
                    .unwrap_or_else(|| "0:00".to_string());
                let file_path = track.file_path.to_string_lossy();

                println!(
                    "    TRACK: \"{}\" | Duration: {} | File: {}",
                    title, duration, file_path
                );
            }
        }
        println!();
    }

    println!("=== END METADATA ===");
}

/// Print library tree in human-readable format
fn print_tree(library: &Library) {
    for artist in &library.artists {
        println!("📁 {}", artist.name);

        for album in &artist.albums {
            let year_str = album.year.map(|y| format!(" ({})", y)).unwrap_or_default();
            println!("├── 📂 {}{}", album.title, year_str);

            for (i, track) in album.tracks.iter().enumerate() {
                let is_last = i == album.tracks.len() - 1;
                let prefix = if is_last {
                    "└─── 🎵"
                } else {
                    "├─── 🎵"
                };

                let track_info = format_track_info(track);
                println!(
                    "{}   {} {}",
                    prefix,
                    track
                        .file_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy(),
                    track_info
                );
            }
        }
        println!();
    }

    // Print summary
    println!("📊 Library Summary:");
    println!("   Artists: {}", library.total_artists);
    println!("   Albums: {}", library.total_albums);
    println!("   Tracks: {}", library.total_tracks);
}

/// Format track information for tree display
fn format_track_info(track: &TrackNode) -> String {
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

/// Apply a metadata update to the track metadata
fn apply_metadata_update(
    metadata: &mut crate::TrackMetadata,
    key: &str,
    value: &str,
) -> Result<(), String> {
    use crate::domain::models::MetadataValue;

    match key.to_lowercase().as_str() {
        "title" => {
            metadata.title = Some(MetadataValue::user_set(value.to_string()));
        }
        "artist" => {
            metadata.artist = Some(MetadataValue::user_set(value.to_string()));
        }
        "album" => {
            metadata.album = Some(MetadataValue::user_set(value.to_string()));
        }
        "albumartist" | "album_artist" => {
            metadata.album_artist = Some(MetadataValue::user_set(value.to_string()));
        }
        "tracknumber" | "track_number" => {
            let num = value
                .parse::<u32>()
                .map_err(|_| format!("Invalid track number: {}", value))?;
            metadata.track_number = Some(MetadataValue::user_set(num));
        }
        "discnumber" | "disc_number" => {
            let num = value
                .parse::<u32>()
                .map_err(|_| format!("Invalid disc number: {}", value))?;
            metadata.disc_number = Some(MetadataValue::user_set(num));
        }
        "year" => {
            let year = value
                .parse::<u32>()
                .map_err(|_| format!("Invalid year: {}", value))?;
            metadata.year = Some(MetadataValue::user_set(year));
        }
        "genre" => {
            metadata.genre = Some(MetadataValue::user_set(value.to_string()));
        }
        _ => {
            return Err(format!("Unsupported metadata field: {}", key));
        }
    }

    Ok(())
}

/// Print validation results in human-readable format
fn build_validation_results(results: &ValidationResult) -> String {
    let mut output = String::new();

    output.push_str("=== METADATA VALIDATION RESULTS ===\n\n");

    output.push_str("📊 Summary:\n");
    output.push_str(&format!("  Total files: {}\n", results.summary.total_files));
    output.push_str(&format!("  Valid files: {}\n", results.summary.valid_files));
    output.push_str(&format!("  Files with errors: {}\n", results.summary.files_with_errors));
    output.push_str(&format!(
        "  Files with warnings: {}\n\n",
        results.summary.files_with_warnings
    ));

    if results.valid {
        output.push_str("✅ All files passed validation!\n");
    } else {
        output.push_str(&format!("❌ Validation failed with {} errors\n", results.errors.len()));
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

/// Handle validate command
pub fn handle_validate(path: PathBuf, json: bool) -> String {
    let tracks = scan_dir(&path);
    let total_scanned = tracks.len();

    if tracks.is_empty() {
        return if json {
            format!(
                "{{\"valid\": true, \"errors\": [], \"warnings\": [], \"summary\": {{\"total_files\": 0, \"valid_files\": 0, \"files_with_errors\": 0, \"files_with_warnings\": 0}}}}"
            )
        } else {
            "No music files found to validate.".to_string()
        }
    }

    // Read metadata for validation
    let tracks_with_metadata: Vec<crate::Track> = tracks
        .into_iter()
        .filter_map(|track| read_metadata(&track.file_path).ok())
        .collect();

    if tracks_with_metadata.is_empty() {
        return if json {
            format!(
                "{{\"valid\": false, \"errors\": [], \"warnings\": [], \"summary\": {{\"total_files\": {}, \"valid_files\": 0, \"files_with_errors\": {}, \"files_with_warnings\": 0}}}}",
                total_scanned, total_scanned
            )
        } else {
            "Unable to read metadata from any files for validation.".to_string()
        }
    }

    let validation_results = validate_tracks(tracks_with_metadata);

    if json {
        to_string_pretty(&validation_results).unwrap_or_else(|e| format!("Error serializing validation results: {}", e))
    } else {
        build_validation_results(&validation_results)
    }
}