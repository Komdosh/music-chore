use crate::adapters::audio_formats::{get_supported_extensions, read_metadata};
use crate::core::domain::with_schema_version;
use crate::core::services::apply_metadata::write_metadata_by_path;
use crate::core::services::cue::{format_cue_validation_result, generate_cue_for_path, parse_cue_file, validate_cue_consistency, CueGenerationError};
use crate::core::services::duplicates::find_duplicates;
use crate::core::services::format_tree::{emit_by_path, format_tree_output};
use crate::core::services::library::build_library_hierarchy;
use crate::core::services::normalization::{normalize, normalize_genres_in_library};
use crate::core::services::scanner::scan_dir;
use crate::presentation::cli::commands::validate_path;
use crate::presentation::cli::Commands;
use serde_json::to_string_pretty;
use std::path::{Path, PathBuf};

/// Handle the parsed CLI command
pub fn handle_command(command: Commands) -> Result<(), i32> {
    match command {
        Commands::Scan {
            path,
            max_depth,
            follow_symlinks,
            exclude,
            json,
            verbose,
        } => {
            match handle_scan(path, max_depth, follow_symlinks, exclude, json, verbose) {
                Ok(()) => Ok(()),
                Err(code) => Err(code),
            }
        }
        Commands::Tree { path, json } => {
            match handle_tree(path, json) {
                Ok(()) => Ok(()),
                Err(code) => Err(code),
            }
        }
        Commands::Read { file } => {
            match handle_read(file) {
                Ok(()) => Ok(()),
                Err(code) => Err(code),
            }
        }
        Commands::Write {
            file,
            set,
            apply,
            dry_run,
        } => {
            match handle_write(file, set, apply, dry_run) {
                Ok(()) => Ok(()),
                Err(code) => Err(code),
            }
        }
        Commands::Normalize {
            path,
            genres,
            title: _,
            dry_run,
        } => {
            if genres {
                match handle_normalize_genres(path, dry_run) {
                    Ok(()) => Ok(()),
                    Err(code) => Err(code),
                }
            } else {
                match handle_normalize(path, dry_run) {
                    Ok(()) => Ok(()),
                    Err(code) => Err(code),
                }
            }
        }
        Commands::Emit { path, json } => {
            match handle_emit(path, json) {
                Ok(()) => Ok(()),
                Err(code) => Err(code),
            }
        }
        Commands::Cue {
            path,
            output,
            dry_run,
            force,
            audio_dir,
            json,
            generate,
            parse,
            validate,
        } => {
            match handle_cue(CueParams {
                path, output, dry_run, force, audio_dir, json, generate, parse, validate,
            }) {
                Ok(()) => Ok(()),
                Err(code) => Err(code),
            }
        }
        Commands::Validate { path, json } => {
            match handle_validate(path, json) {
                Ok(()) => Ok(()),
                Err(code) => Err(code),
            }
        }
        Commands::Duplicates { path, json } => {
            match handle_duplicates(path, json) {
                Ok(()) => Ok(()),
                Err(code) => Err(code),
            }
        }
    }
}

pub fn handle_scan(
    path: PathBuf,
    max_depth: Option<usize>,
    follow_symlinks: bool,
    exclude: Vec<String>,
    json: bool,
    verbose: bool,
) -> Result<(), i32> {
    if !path.exists() {
        eprintln!("Error: Path does not exist: {}", path.display());
        return Err(1);
    }

    let tracks = crate::core::services::scanner::scan_dir_with_options_verbose(&path, max_depth, follow_symlinks, exclude, verbose);

    if tracks.is_empty() {
        if path.is_file() {
            eprintln!("Error: Path is not a directory: {}", path.display());
            return Err(1);
        } else {
            eprintln!("No music files found in directory: {}", path.display());
        }
        return Ok(());
    }

    if verbose {
        eprintln!("Scanned {} music files from {}", tracks.len(), path.display());
    }

    if json {
        match to_string_pretty(&tracks) {
            Ok(s) => println!("{}", s),
            Err(e) => {
                eprintln!("Error serializing to JSON: {}", e);
                return Err(1);
            }
        }
    } else {
        for track in tracks {
            println!("{}", track.file_path.display());
        }
    }

    Ok(())
}

pub fn handle_tree(path: PathBuf, json: bool) -> Result<(), i32> {
    if !path.exists() {
        eprintln!("Error: Path does not exist: {}", path.display());
        return Err(1);
    }

    if json {
        let tracks = scan_dir(&path);
        let library = build_library_hierarchy(tracks);
        let wrapper = with_schema_version(&library);
        match to_string_pretty(&wrapper) {
            Ok(s) => println!("{}", s),
            Err(e) => {
                eprintln!("Error serializing to JSON: {}", e);
                return Err(1);
            }
        }
    } else {
        println!("{}", format_tree_output(&path));
    }

    Ok(())
}

pub fn handle_read(file: PathBuf) -> Result<(), i32> {
    if !file.exists() {
        eprintln!("Error: File does not exist: {}", file.display());
        return Err(1);
    }

    match read_metadata(&file) {
        Ok(track) => {
            let wrapper = with_schema_version(&track);
            match to_string_pretty(&wrapper) {
                Ok(s) => println!("{}", s),
                Err(e) => {
                    eprintln!("Error serializing track: {}", e);
                    return Err(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading metadata: {}", e);
            return Err(1);
        }
    }

    Ok(())
}

pub fn handle_write(file: PathBuf, set: Vec<String>, apply: bool, dry_run: bool) -> Result<(), i32> {
    if !file.exists() && apply {
        eprintln!("Error: File does not exist: {}", file.display());
        return Err(1);
    }

    match write_metadata_by_path(&file, set, apply, dry_run) {
        Ok(result) => println!("{}", result),
        Err(e) => {
            eprintln!("{}", e);
            return Err(1);
        }
    }

    Ok(())
}

pub fn handle_normalize(path: PathBuf, dry_run: bool) -> Result<(), i32> {
    if !path.exists() {
        eprintln!("Error: Path does not exist: {}", path.display());
        return Err(1);
    }

    match normalize(path, dry_run) {
        Ok(result) => {
            println!("{}", result);
            Ok(())
        },
        Err(e) => {
            eprintln!("{}", e);
            Err(1)
        }
    }
}

pub fn handle_normalize_genres(path: PathBuf, dry_run: bool) -> Result<(), i32> {
    if !path.exists() {
        eprintln!("Error: Path does not exist: {}", path.display());
        return Err(1);
    }

    match normalize_genres_in_library(&path, dry_run) {
        Ok(result) => {
            println!("{}", result);
            Ok(())
        },
        Err(e) => {
            eprintln!("{}", e);
            Err(1)
        }
    }
}

pub fn handle_emit(path: PathBuf, json: bool) -> Result<(), i32> {
    if !path.exists() {
        eprintln!("Error: Path does not exist: {}", path.display());
        return Err(1);
    }

    match emit_by_path(&path, json) {
        Ok(result) => println!("{}", result),
        Err(err) => {
            eprintln!("{}", err);
            return Err(1);
        }
    }

    Ok(())
}

pub fn handle_duplicates(path: PathBuf, json: bool) -> Result<(), i32> {
    if !path.exists() {
        eprintln!("Error: Path does not exist: {}", path.display());
        return Err(1);
    }

    match find_duplicates(&path, json) {
        Ok(value) => {
            println!("{}", value);
            Ok(())
        },
        Err(value) => {
            eprintln!("{}", value);
            Err(1)
        }
    }
}

fn handle_validate(path: PathBuf, json: bool) -> Result<(), i32> {
    if !path.exists() {
        eprintln!("Error: Path does not exist: {}", path.display());
        return Err(1);
    }

    match validate_path(&path, json) {
        Ok(value) => {
            println!("{}", value);
            Ok(())
        },
        Err(value) => {
            eprintln!("{}", value);
            Err(1)
        }
    }
}

struct CueParams {
    path: PathBuf,
    output: Option<PathBuf>,
    dry_run: bool,
    force: bool,
    audio_dir: Option<PathBuf>,
    json: bool,
    generate: bool,
    parse: bool,
    validate: bool,
}

fn handle_cue(params: CueParams) -> Result<(), i32> {
    let operation_count = params.generate as u8 + params.parse as u8 + params.validate as u8;

    if operation_count == 0 {
        eprintln!("Error: Must specify --generate, --parse, or --validate");
        return Err(1);
    }

    if operation_count > 1 {
        eprintln!("Error: Can only specify one of --generate, --parse, or --validate");
        return Err(1);
    }

    if params.generate {
        handle_cue_generate(params.path, params.output, params.dry_run, params.force)?;
    } else if params.parse {
        handle_cue_parse(params.path, params.json)?;
    } else if params.validate {
        handle_cue_validate(params.path, params.audio_dir, params.json)?;
    }

    Ok(())
}

fn handle_cue_generate(
    path: PathBuf,
    output: Option<PathBuf>,
    dry_run: bool,
    force: bool,
) -> Result<(), i32> {
    match generate_cue_for_path(&path, output) {
        Ok(result) => {
            if !dry_run && result.output_path.exists() && !force {
                eprintln!(
                    "Error: Cue file already exists at '{}'. Use --force to overwrite.",
                    result.output_path.display()
                );
                return Err(1);
            }

            if dry_run {
                println!("{}", result.cue_content);
                println!("---");
                println!("Would write to: {}", result.output_path.display());
            } else {
                match std::fs::write(&result.output_path, &result.cue_content) {
                    Ok(_) => println!("Cue file written to: {}", result.output_path.display()),
                    Err(e) => {
                        eprintln!("Error writing cue file: {}", e);
                        return Err(1);
                    }
                }
            }
            Ok(())
        }
        Err(CueGenerationError::NoMusicFiles) => {
            eprintln!(
                "No music files found in directory (checked only immediate files, not subdirectories)"
            );
            Err(1)
        }
        Err(CueGenerationError::NoReadableFiles) => {
            eprintln!("No readable music files found in directory");
            Err(1)
        }
        Err(CueGenerationError::FileReadError(msg)) => {
            eprintln!("{}", msg);
            Err(1)
        }
    }
}

fn handle_cue_parse(path: PathBuf, json: bool) -> Result<(), i32> {
    if !path.exists() {
        eprintln!("Error: File does not exist: {}", path.display());
        return Err(1);
    }

    match parse_cue_file(&path) {
        Ok(cue_file) => {
            if json {
                let wrapper = with_schema_version(&cue_file);
                match to_string_pretty(&wrapper) {
                    Ok(s) => println!("{}", s),
                    Err(e) => {
                        eprintln!("Error serializing cue file: {}", e);
                        return Err(1);
                    }
                }
            } else {
                println!("Cue File: {}", path.display());
                if let Some(performer) = &cue_file.performer {
                    println!("  Performer: {}", performer);
                }
                if let Some(title) = &cue_file.title {
                    println!("  Title: {}", title);
                }
                if !cue_file.files.is_empty() {
                    println!("  Files:");
                    for file in &cue_file.files {
                        println!("    - {}", file);
                    }
                }
                println!("  Tracks: {}", cue_file.tracks.len());
                for track in &cue_file.tracks {
                    let file_info = track
                        .file
                        .as_ref()
                        .map(|f| format!(" [{}]", f))
                        .unwrap_or_default();
                    println!(
                        "    Track {:02}: {}{}",
                        track.number,
                        track.title.as_deref().unwrap_or("(no title)"),
                        file_info
                    );
                }
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("Error parsing cue file: {}", e);
            Err(1)
        }
    }
}

fn handle_cue_validate(path: PathBuf, audio_dir: Option<PathBuf>, json: bool) -> Result<(), i32> {
    if !path.exists() {
        eprintln!("Error: File does not exist: {}", path.display());
        return Err(1);
    }

    let audio_directory = audio_dir.unwrap_or_else(|| {
        path.parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    });

    let supported_extensions = get_supported_extensions();

    let audio_files: Vec<PathBuf> = match std::fs::read_dir(&audio_directory) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            .filter(|e| {
                // Only include audio files, not the CUE file itself
                let extension = e.path().extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.to_lowercase());

                extension.as_deref().iter().any(|ext| supported_extensions.contains(&ext.to_string()))
            })
            .map(|e| e.path())
            .collect(),
        Err(e) => {
            eprintln!("Error reading audio directory: {}", e);
            return Err(1);
        }
    };

    let audio_files_refs: Vec<&Path> = audio_files.iter().map(|p| p.as_path()).collect();
    let result = validate_cue_consistency(&path, &audio_files_refs);

    if json {
        let wrapper = with_schema_version(&result);
        match to_string_pretty(&wrapper) {
            Ok(s) => println!("{}", s),
            Err(e) => {
                eprintln!("Error serializing result: {}", e);
                return Err(1);
            }
        }
    } else {
        println!("{}", format_cue_validation_result(&result));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use std::path::PathBuf;

    #[test]
    fn test_handle_scan_with_existing_path() {
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().join("test_dir");
        fs::create_dir(&test_path).unwrap();
        
        // Create a dummy audio file to ensure scan finds something
        let audio_file = test_path.join("test.flac");
        fs::write(&audio_file, b"dummy flac content").unwrap();
        
        let result = handle_scan(
            test_path,
            None,
            false,
            vec![],
            false,
            false
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_scan_with_nonexistent_path() {
        let nonexistent_path = PathBuf::from("/nonexistent/path/test");
        let result = handle_scan(
            nonexistent_path,
            None,
            false,
            vec![],
            false,
            false
        );
        assert_eq!(result, Err(1));
    }

    #[test]
    fn test_handle_tree_with_existing_path() {
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().join("test_dir");
        fs::create_dir(&test_path).unwrap();
        
        let result = handle_tree(test_path, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_tree_with_nonexistent_path() {
        let nonexistent_path = PathBuf::from("/nonexistent/path/test");
        let result = handle_tree(nonexistent_path, false);
        assert_eq!(result, Err(1));
    }

    #[test]
    fn test_handle_read_with_nonexistent_file() {
        let nonexistent_file = PathBuf::from("/nonexistent/path/test.flac");
        let result = handle_read(nonexistent_file);
        assert_eq!(result, Err(1));
    }

    #[test]
    fn test_handle_normalize_with_nonexistent_path() {
        let nonexistent_path = PathBuf::from("/nonexistent/path/test");
        let result = handle_normalize(nonexistent_path, true);
        assert_eq!(result, Err(1));
    }

    #[test]
    fn test_handle_normalize_genres_with_nonexistent_path() {
        let nonexistent_path = PathBuf::from("/nonexistent/path/test");
        let result = handle_normalize_genres(nonexistent_path, true);
        assert_eq!(result, Err(1));
    }

    #[test]
    fn test_handle_emit_with_nonexistent_path() {
        let nonexistent_path = PathBuf::from("/nonexistent/path/test");
        let result = handle_emit(nonexistent_path, false);
        assert_eq!(result, Err(1));
    }

    #[test]
    fn test_handle_duplicates_with_nonexistent_path() {
        let nonexistent_path = PathBuf::from("/nonexistent/path/test");
        let result = handle_duplicates(nonexistent_path, false);
        assert_eq!(result, Err(1));
    }

    #[test]
    fn test_handle_validate_with_nonexistent_path() {
        let nonexistent_path = PathBuf::from("/nonexistent/path/test");
        let result = handle_validate(nonexistent_path, false);
        assert_eq!(result, Err(1));
    }

    #[test]
    fn test_handle_cue_generate_with_nonexistent_path() {
        let nonexistent_path = PathBuf::from("/nonexistent/path/test");
        let result = handle_cue_generate(nonexistent_path, None, false, false);
        assert_eq!(result, Err(1));
    }

    #[test]
    fn test_handle_cue_parse_with_nonexistent_file() {
        let nonexistent_file = PathBuf::from("/nonexistent/path/test.cue");
        let result = handle_cue_parse(nonexistent_file, false);
        assert_eq!(result, Err(1));
    }

    #[test]
    fn test_handle_cue_validate_with_nonexistent_file() {
        let nonexistent_file = PathBuf::from("/nonexistent/path/test.cue");
        let result = handle_cue_validate(nonexistent_file, None, false);
        assert_eq!(result, Err(1));
    }

    #[test]
    fn test_handle_cue_operation_selection_logic() {
        // Test that cue command enforces exactly one operation
        let params_no_op = CueParams {
            path: PathBuf::from("test.cue"),
            output: None,
            dry_run: false,
            force: false,
            audio_dir: None,
            json: false,
            generate: false,
            parse: false,
            validate: false,
        };
        
        let result_no_op = handle_cue(params_no_op);
        assert_eq!(result_no_op, Err(1)); // Should fail when no operation is specified
        
        let params_multi_op = CueParams {
            path: PathBuf::from("test.cue"),
            output: None,
            dry_run: false,
            force: false,
            audio_dir: None,
            json: false,
            generate: true,
            parse: true,
            validate: false,
        };
        
        let result_multi_op = handle_cue(params_multi_op);
        assert_eq!(result_multi_op, Err(1)); // Should fail when multiple operations are specified
    }

    #[test]
    fn test_handle_write_with_nonexistent_file_dry_run() {
        // Test that handle_write fails when file doesn't exist, even with dry_run
        let nonexistent_file = PathBuf::from("/nonexistent/path/test.flac");
        let result = handle_write(nonexistent_file, vec![], false, true); // apply=false, dry_run=true
        // This should fail since the file doesn't exist
        assert_eq!(result, Err(1));
    }

    #[test]
    fn test_handle_write_with_nonexistent_file_apply_true() {
        // Test that handle_write fails when file doesn't exist and apply is true
        let nonexistent_file = PathBuf::from("/nonexistent/path/test.flac");
        let result = handle_write(nonexistent_file, vec![], true, false); // apply=true, dry_run=false
        assert_eq!(result, Err(1)); // Should fail when trying to apply to non-existent file
    }

    // --- New tests for handle_cue_validate ---

    #[test]
    fn test_handle_cue_validate_success_same_directory() {
        let temp_dir = TempDir::new().unwrap();
        let cue_file_path = temp_dir.path().join("test.cue");
        let audio_file_path = temp_dir.path().join("track01.flac");

        // Create a dummy CUE file
        let cue_content = format!(
            r#"TITLE "Test Album"
PERFORMER "Test Artist"
FILE "track01.flac" WAVE
  TRACK 01 AUDIO
    TITLE "Test Track 01"
    PERFORMER "Test Artist"
    INDEX 01 00:00:00"#
        );
        fs::write(&cue_file_path, cue_content).unwrap();

        // Create a dummy audio file
        fs::write(&audio_file_path, b"dummy flac content").unwrap();

        let result = handle_cue_validate(cue_file_path, None, false);
        assert!(result.is_ok());
        // Expected: Prints success message indicating validation passed
    }

    #[test]
    fn test_handle_cue_validate_success_with_audio_dir() {
        let cue_temp_dir = TempDir::new().unwrap();
        let audio_temp_dir = TempDir::new().unwrap();

        let cue_file_path = cue_temp_dir.path().join("test.cue");
        let audio_file_path = audio_temp_dir.path().join("track01.flac");

        // Create a dummy CUE file referencing the audio file in a separate directory
        let cue_content = format!(
            r#"TITLE "Test Album"
PERFORMER "Test Artist"
FILE "track01.flac" WAVE
  TRACK 01 AUDIO
    TITLE "Test Track 01"
    PERFORMER "Test Artist"
    INDEX 01 00:00:00"#
        );
        fs::write(&cue_file_path, cue_content).unwrap();

        // Create a dummy audio file in the specified audio directory
        fs::write(&audio_file_path, b"dummy flac content").unwrap();

        let result = handle_cue_validate(cue_file_path, Some(audio_temp_dir.path().to_path_buf()), false);
        assert!(result.is_ok());
        // Expected: Prints success message indicating validation passed
    }

    #[test]
    fn test_handle_cue_validate_reports_missing_audio_files() {
        let temp_dir = TempDir::new().unwrap();
        let cue_file_path = temp_dir.path().join("test.cue");

        // Create a dummy CUE file referencing a non-existent audio file
        let cue_content = format!(
            r#"TITLE "Test Album"
PERFORMER "Test Artist"
FILE "non_existent_track.flac" WAVE
  TRACK 01 AUDIO
    TITLE "Non-Existent Track"
    PERFORMER "Test Artist"
    INDEX 01 00:00:00"#
        );
        fs::write(&cue_file_path, cue_content).unwrap();

        // DO NOT create the audio file

        let result = handle_cue_validate(cue_file_path, None, false);
        assert!(result.is_ok()); // Validation should still return Ok, but report the missing file
        // Expected: Prints validation report including "File 'non_existent_track.flac' mentioned in CUE file not found"
    }

    // --- New tests for handle_cue_parse ---

    #[test]
    fn test_handle_cue_parse_success_text_output() {
        let temp_dir = TempDir::new().unwrap();
        let cue_file_path = temp_dir.path().join("test.cue");

        let cue_content = r#"TITLE "Example Album"
PERFORMER "Example Artist"
FILE "audio.flac" WAVE
  TRACK 01 AUDIO
    TITLE "Track One"
    PERFORMER "Example Artist"
    INDEX 01 00:00:00
  TRACK 02 AUDIO
    TITLE "Track Two"
    PERFORMER "Another Artist"
    INDEX 01 03:00:00"#;
        fs::write(&cue_file_path, cue_content).unwrap();

        let result = handle_cue_parse(cue_file_path, false);
        assert!(result.is_ok());
        // Expected output: Formatted text representation of the CUE file.
    }

    #[test]
    fn test_handle_cue_parse_success_json_output() {
        let temp_dir = TempDir::new().unwrap();
        let cue_file_path = temp_dir.path().join("test.cue");

        let cue_content = r#"TITLE "Example Album"
PERFORMER "Example Artist"
FILE "audio.flac" WAVE
  TRACK 01 AUDIO
    TITLE "Track One"
    PERFORMER "Example Artist"
    INDEX 01 00:00:00"#;
        fs::write(&cue_file_path, cue_content).unwrap();

        let result = handle_cue_parse(cue_file_path, true);
        assert!(result.is_ok());
        // Expected output: JSON representation of the CUE file.
    }

    #[test]
    fn test_handle_cue_parse_invalid_cue_syntax() {
        let temp_dir = TempDir::new().unwrap();
        let cue_file_path = temp_dir.path().join("invalid.cue");

        // Malformed CUE content
        let cue_content = r#"TITLE "Example Album"
PERFORMER "Example Artist"
FILE "audio.flac" WAVE
  TRACK 01 AUDIO
    TITLE "Track One"
    PERFORMER Example Artist" // Missing quote
    INDEX 01 00:00:00"#;
        fs::write(&cue_file_path, cue_content).unwrap();

        let result = handle_cue_parse(cue_file_path, false);
        assert_eq!(result, Err(1));
        // Expected: Prints an error message about parsing the cue file.
    }
}
