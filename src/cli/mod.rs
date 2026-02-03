//! CLI module for musicctl commands.

pub mod commands;
pub mod commands_processor;

// Re-export commonly used CLI types
pub use commands::{Cli, Commands};

pub use commands_processor::handle_command;