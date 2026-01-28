use clap::{Parser, Subcommand};
use music_chore::domain::Track;
use music_chore::infra::audio::flac::read_flac_metadata;
use music_chore::infra::scanner::scan_dir;
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
    /// Generate a .cue file for an album directory.
    CueGenerate { dir: PathBuf },
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
            // For now, we'll just show the same output as scan since we don't have the hierarchy built yet
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
        Commands::Read { file } => match read_flac_metadata(&file) {
            Ok(track) => match to_string_pretty(&track) {
                Ok(s) => println!("{}", s),
                Err(e) => eprintln!("Error serializing track: {}", e),
            },
            Err(e) => eprintln!("Error reading FLAC metadata: {}", e),
        },
        Commands::Write { file, set, apply } => {
            eprintln!("Not implemented yet");
        }
        Commands::CueGenerate { dir } => {
            eprintln!("Not implemented yet");
        }
    }
}
