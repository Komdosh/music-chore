//! CLI module for music-chore commands.

pub mod commands;

// Re-export commonly used CLI types
pub use commands::{handle_command, Cli, Commands};
