use crate::build_library_hierarchy;
use crate::cli::commands::validate_path;
use crate::cli::Commands;
use crate::domain::with_schema_version;
use crate::services::apply_metadata::write_metadata_by_path;
use crate::services::cue::{
    generate_cue_for_path, parse_cue_file, validate_cue_consistency, CueGenerationError,
};
use crate::services::duplicates::find_duplicates;
use crate::services::format_tree::{emit_by_path, format_tree_output};
use crate::services::formats::read_metadata;
use crate::services::normalization::{normalize, normalize_genres_in_library};
use crate::services::scanner::scan_dir;
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
            handle_scan(path, max_depth, follow_symlinks, exclude, json, verbose);
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
        } => {
            handle_write(file, set, apply, dry_run);
            Ok(())
        }
        Commands::Normalize {
            path,
            genres,
            title: _,
            dry_run,
        } => {
            if genres {
                handle_normalize_genres(path, dry_run);
            } else {
                handle_normalize(path, dry_run);
            }
            Ok(())
        }
        Commands::Emit { path, json } => {
            handle_emit(path, json);
            Ok(())
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
        } => handle_cue(
            path, output, dry_run, force, audio_dir, json, generate, parse, validate,
        ),
        Commands::Validate { path, json } => {
            handle_validate(path, json);
            Ok(())
        }
        Commands::Duplicates { path, json } => {
            handle_duplicates(path, json);
            Ok(())
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
) {
    let tracks = crate::services::scanner::scan_dir_with_options_verbose(&path, max_depth, follow_symlinks, exclude, verbose);

    if tracks.is_empty() {
        eprintln!("No music files found in directory: {}", path.display());
        return;
    }

    if verbose {
        eprintln!("Scanned {} music files from {}", tracks.len(), path.display());
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

pub fn handle_tree(path: PathBuf, json: bool) {
    if json {
        let tracks = scan_dir(&path);
        let library = build_library_hierarchy(tracks);
        let wrapper = with_schema_version(&library);
        match to_string_pretty(&wrapper) {
            Ok(s) => println!("{}", s),
            Err(e) => eprintln!("Error serializing to JSON: {}", e),
        }
    } else {
        println!("{}", format_tree_output(&path));
    }
}

pub fn handle_read(file: PathBuf) {
    match read_metadata(&file) {
        Ok(track) => {
            let wrapper = with_schema_version(&track);
            match to_string_pretty(&wrapper) {
                Ok(s) => println!("{}", s),
                Err(e) => eprintln!("Error serializing track: {}", e),
            }
        }
        Err(e) => eprintln!("Error reading metadata: {}", e),
    }
}

pub fn handle_write(file: PathBuf, set: Vec<String>, apply: bool, dry_run: bool) {
    match write_metadata_by_path(&file, set, apply, dry_run) {
        Ok(result) => println!("{}", result),
        Err(e) => eprintln!("{}", e),
    }
}

pub fn handle_normalize(path: PathBuf, dry_run: bool) {
    match normalize(path, dry_run) {
        Ok(result) => println!("{}", result),
        Err(e) => eprintln!("{}", e),
    }
}

pub fn handle_normalize_genres(path: PathBuf, dry_run: bool) {
    match normalize_genres_in_library(&path, dry_run) {
        Ok(result) => println!("{}", result),
        Err(e) => eprintln!("{}", e),
    }
}

pub fn handle_emit(path: PathBuf, json: bool) {
    match emit_by_path(&path, json) {
        Ok(result) => println!("{}", result),
        Err(err) => eprintln!("{}", err),
    }
}

pub fn handle_duplicates(path: PathBuf, json: bool) {
    match find_duplicates(&path, json) {
        Ok(value) => println!("{}", value),
        Err(value) => eprintln!("{}", value),
    }
}

fn handle_validate(path: PathBuf, json: bool) {
    match validate_path(&path, json) {
        Ok(value) => println!("{}", value),
        Err(value) => eprintln!("{}", value),
    }
}

fn handle_cue(
    path: PathBuf,
    output: Option<PathBuf>,
    dry_run: bool,
    force: bool,
    audio_dir: Option<PathBuf>,
    json: bool,
    generate: bool,
    parse: bool,
    validate: bool,
) -> Result<(), i32> {
    let operation_count = generate as u8 + parse as u8 + validate as u8;

    if operation_count == 0 {
        eprintln!("Error: Must specify --generate, --parse, or --validate");
        return Err(1);
    }

    if operation_count > 1 {
        eprintln!("Error: Can only specify one of --generate, --parse, or --validate");
        return Err(1);
    }

    if generate {
        handle_cue_generate(path, output, dry_run, force)?;
    } else if parse {
        handle_cue_parse(path, json);
    } else if validate {
        handle_cue_validate(path, audio_dir, json);
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

fn handle_cue_parse(path: PathBuf, json: bool) {
    match parse_cue_file(&path) {
        Ok(cue_file) => {
            if json {
                let wrapper = with_schema_version(&cue_file);
                match to_string_pretty(&wrapper) {
                    Ok(s) => println!("{}", s),
                    Err(e) => eprintln!("Error serializing cue file: {}", e),
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
        }
        Err(e) => eprintln!("Error parsing cue file: {}", e),
    }
}

fn handle_cue_validate(path: PathBuf, audio_dir: Option<PathBuf>, json: bool) {
    let audio_directory = audio_dir.unwrap_or_else(|| {
        path.parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    });

    let audio_files: Vec<PathBuf> = match std::fs::read_dir(&audio_directory) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            .map(|e| e.path())
            .collect(),
        Err(e) => {
            eprintln!("Error reading audio directory: {}", e);
            return;
        }
    };

    let audio_files_refs: Vec<&Path> = audio_files.iter().map(|p| p.as_path()).collect();
    let result = validate_cue_consistency(&path, &audio_files_refs);

    if json {
        let wrapper = with_schema_version(&result);
        match to_string_pretty(&wrapper) {
            Ok(s) => println!("{}", s),
            Err(e) => eprintln!("Error serializing result: {}", e),
        }
    } else {
        if result.is_valid {
            println!("✓ CUE file is valid");
            println!("  All referenced files exist and track count matches.");
        } else {
            println!("✗ CUE file validation failed:");
            if result.parsing_error {
                println!("  - Error parsing CUE file");
            }
            if result.file_missing {
                println!("  - Referenced audio file(s) missing");
            }
            if result.track_count_mismatch {
                println!("  - Track count mismatch between CUE and audio files");
            }
        }
    }
}
