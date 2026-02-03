use crate::build_library_hierarchy;
use crate::cli::Commands;
use crate::cli::commands::validate_path;
use crate::services::apply_metadata::write_metadata_by_path;
use crate::services::duplicates::find_duplicates;
use crate::services::format_tree::{emit_by_path, format_tree_output};
use crate::services::formats::read_metadata;
use crate::services::normalization::normalize;
use crate::services::scanner::{scan_dir, scan_tracks};
use serde_json::to_string_pretty;
use std::path::PathBuf;

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
        } => {
            handle_write(file, set, apply, dry_run);
            Ok(())
        }
        Commands::Normalize { path, dry_run } => {
            handle_normalize(path, dry_run);
            Ok(())
        }
        Commands::Emit { path, json } => {
            handle_emit(path, json);
            Ok(())
        }
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

pub fn handle_scan(path: PathBuf, json: bool) {
    match scan_tracks(path, json) {
        Ok(result) => println!("{}", result),
        Err(err) => eprintln!("{}", err),
    }
}

pub fn handle_tree(path: PathBuf, json: bool) {
    let tracks = scan_dir(&path);
    let library = build_library_hierarchy(tracks);

    if json {
        match to_string_pretty(&library) {
            Ok(s) => println!("{}", s),
            Err(e) => eprintln!("Error serializing to JSON: {}", e),
        }
    } else {
        println!("{}", format_tree_output(&library));
    }
}

pub fn handle_read(file: PathBuf) {
    match read_metadata(&file) {
        Ok(track) => match to_string_pretty(&track) {
            Ok(s) => println!("{}", s),
            Err(e) => eprintln!("Error serializing track: {}", e),
        },
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
