//! musicctl - A deterministic, AI-friendly music metadata compiler.
//!
//! This is the main library entry point for the musicctl project.

// Re-export modules for easy access
pub use crate::cli::commands;
pub use crate::domain::*;
pub use crate::infra::*;

// Import submodules
pub mod cli;
pub mod app;
pub mod domain;
pub mod infra;