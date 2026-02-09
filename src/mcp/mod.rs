//! MCP-specific functionality and formatting
//!
//! This crate contains MCP-specific logic that bridges CLI functionality
//! with Model Context Protocol interface.

pub mod music_chore_server;
mod params;
mod prompt_handler_requests;
pub mod config;
mod music_chore_server_impl;
mod cue_helper_methods;
mod call_tool_result;
mod prompts;
