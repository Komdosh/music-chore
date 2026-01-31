//! Music Chore CLI Entry Point

use clap::Parser;
use music_chore::cli::{handle_command, Cli};

fn main() {
    let cli = Cli::parse();
    handle_command(cli.command);
}
