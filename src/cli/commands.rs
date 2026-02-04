//! CLI command definitions and handlers.

pub(crate) use crate::services::validation::validate_path;
use clap::{Parser, Subcommand};
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
    /// Generate .cue file for an album folder.
    Cue {
        /// Path to album directory.
        path: PathBuf,
        /// Output path for .cue file (defaults to album directory).
        output: Option<PathBuf>,
        /// Show what would be written without actually writing file.
        #[arg(long)]
        dry_run: bool,
        /// Overwrite existing .cue file.
        #[arg(long)]
        force: bool,
    },
    /// Parse and display contents of a .cue file.
    CueParse {
        /// Path to .cue file.
        path: PathBuf,
        /// Output JSON instead of human-readable format.
        #[arg(long)]
        json: bool,
    },
    /// Validate a .cue file against its referenced audio files.
    CueValidate {
        /// Path to .cue file.
        path: PathBuf,
        /// Path to directory containing audio files (defaults to .cue file directory).
        #[arg(long)]
        audio_dir: Option<PathBuf>,
        /// Output JSON instead of human-readable format.
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
    /// Detect duplicate tracks by checksum.
    Duplicates {
        /// Base directory to scan.
        path: PathBuf,
        /// Output JSON instead of human-readable format.
        #[arg(long)]
        json: bool,
    },
}
