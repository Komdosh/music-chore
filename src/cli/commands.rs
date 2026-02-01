//! CLI command definitions and handlers.

use crate::{
    build_library_hierarchy, normalize_track_titles, read_metadata, scan_dir, Library,
    OperationResult, TrackNode,
};
use clap::{Parser, Subcommand};
use serde_json::to_string_pretty;
use std::path::PathBuf;

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
}

/// Handle the parsed CLI command
pub fn handle_command(command: Commands) {
    match command {
        Commands::Scan { path, json } => handle_scan(path, json),
        Commands::Tree { path, json } => handle_tree(path, json),
        Commands::Read { file } => handle_read(file),
        Commands::Write { file, set, apply } => handle_write(file, set, apply),
        Commands::Normalize { path, dry_run } => handle_normalize(path, dry_run),
        Commands::Emit { path, json } => handle_emit(path, json),
    }
}

/// Handle scan command
fn handle_scan(path: PathBuf, json: bool) {
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
fn handle_tree(path: PathBuf, json: bool) {
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
fn handle_read(file: PathBuf) {
    match read_metadata(&file) {
        Ok(track) => match to_string_pretty(&track) {
            Ok(s) => println!("{}", s),
            Err(e) => eprintln!("Error serializing track: {}", e),
        },
        Err(e) => eprintln!("Error reading metadata: {}", e),
    }
}

/// Handle write command
fn handle_write(file: PathBuf, set: Vec<String>, apply: bool) {
    // Write metadata to audio file
    println!("Write command not yet implemented");
    println!("File: {:?}", file);
    println!("Set: {:?}", set);
    println!("Apply: {}", apply);
}

/// Handle normalize command
fn handle_normalize(path: PathBuf, dry_run: bool) {
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
fn handle_emit(path: PathBuf, json: bool) {
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
fn emit_structured_output(library: &Library) {
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
