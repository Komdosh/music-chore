//! Music Chore CLI Entry Point

use clap::Parser;
use music_chore::cli::{handle_command, Cli};

fn main() {
    let cli = Cli::parse();

    // Handle version flag
    if cli.version {
        println!("musicctl {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    // Handle subcommand if provided
    if let Some(command) = cli.command {
        handle_command(command);
    } else {
        // Show help if no command provided
        println!("Deterministic, AI‑friendly music metadata compiler.");
        println!();
        println!("Usage: musicctl [OPTIONS] <COMMAND>");
        println!();
        println!("Commands:");
        println!("  scan       Recursively scan a directory for music files");
        println!("  tree       Show a human‑friendly tree view");
        println!("  read       Read metadata from a single file");
        println!("  write      Write metadata to a file");
        println!("  normalize  Normalize track titles to title case");
        println!("  emit       Emit library metadata in structured JSON format");
        println!("  help       Print this message or the help of the given subcommand(s)");
        println!();
        println!("Options:");
        println!("  -v, --version  Show version information");
        println!("  -h, --help     Print help");
    }
}
