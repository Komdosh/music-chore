use clap::{Parser, Subcommand};
use music_chore::{
    build_library_hierarchy, infra::audio::flac::read_flac_metadata, infra::scanner::scan_dir,
    normalize_track_titles, Library, OperationResult,
};
use serde_json::to_string_pretty;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "music-chore")]
#[command(about = "Deterministic, AI‑friendly music metadata compiler.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Recursively scan a directory for music files.
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
    /// Read metadata from a single FLAC file.
    Read {
        /// Path to the FLAC file.
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
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Scan { path, json } => {
            let tracks = scan_dir(&path);
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
        Commands::Tree { path, json } => {
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
        Commands::Read { file } => match read_flac_metadata(&file) {
            Ok(track) => match to_string_pretty(&track) {
                Ok(s) => println!("{}", s),
                Err(e) => eprintln!("Error serializing track: {}", e),
            },
            Err(e) => eprintln!("Error reading FLAC metadata: {}", e),
        },
        Commands::Write {
            file: _,
            set: _,
            apply: _,
        } => {
            // Write metadata to audio file
            println!("Write command not yet implemented");
        }
        Commands::Normalize { path, dry_run } => match normalize_track_titles(&path, dry_run) {
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
        },
    }
}

fn print_tree(library: &Library) {
    for artist in &library.artists {
        println!("📁 {}", artist.name);

        for album in &artist.albums {
            let year_str = album.year.map(|y| format!(" ({})", y)).unwrap_or_default();
            println!("├── 📂 {}{}", album.title, year_str);

            for (i, track) in album.tracks.iter().enumerate() {
                let is_last = i == album.tracks.len() - 1;
                let prefix = if is_last { "└──" } else { "├──" };

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

fn format_track_info(track: &music_chore::TrackNode) -> String {
    let mut info = Vec::new();

    if let Some(duration) = &track.metadata.duration {
        let minutes = (duration.value / 60.0) as u32;
        let seconds = (duration.value % 60.0) as u32;
        info.push(format!("{}:{:02}", minutes, seconds));
    }

    if let Some(track_number) = &track.metadata.track_number {
        info.push(format!("#{}", track_number.value));
    }

    if let Some(format) = track.metadata.format.strip_prefix(".") {
        info.push(format.to_uppercase());
    } else {
        info.push(track.metadata.format.to_uppercase());
    }

    let source = match track
        .metadata
        .title
        .as_ref()
        .map(|t| &t.source)
        .unwrap_or(&music_chore::MetadataSource::FolderInferred)
    {
        music_chore::MetadataSource::Embedded => "🎯",
        music_chore::MetadataSource::FolderInferred => "🤖",
        music_chore::MetadataSource::UserEdited => "👤",
    };

    format!("[{}] {}", source, info.join(" | "))
}
