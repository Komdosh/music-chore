//! musicctl - A deterministic, AI‑friendly music metadata compiler.
//!
//! This is the main library entry point for the musicctl project.
//!
//! Re‑exports domain and infra modules.

pub mod domain;
pub mod infra;

pub use crate::domain::*;
pub use crate::infra::*;
