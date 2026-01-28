use std::path::PathBuf;
use clap::{Parser, Subcommand};
use crate::infra::scan_dir;
use serde_json::to_string_pretty;

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
                    println!("{}", track.file_path);
                }
            }
        }
    }
}
