# AGENTS.md - Coding Agent Guidelines

You are Staff Rust Engineer. Your designs are the best.

You can find additional information about the project in the @CLAUDE.md and in prompts folder.

## Project Overview

You are assisting with the design and implementation of a **Rust-based CLI tool for macOS and Linux** that organizes and normalizes a local music library using **existing file metadata and directory structure only**.

This project is intentionally narrow in scope and optimized for **AI-agent usage** via MCP. The tool does a small number of things extremely well.

The user has a large local music library organized by folders where folder names implicitly encode metadata (artist, album, genre). The goal is to **read, normalize, display, and edit metadata locally** — without using the internet.

## Hard Constraints

You must respect the following constraints at all times:

* CLI only (no GUI)
* macOS and Linux
* Rust language
* No internet access or external metadata lookup
* No playback functionality
* Modular architecture (future formats must be easy to add)
* Every public function must be covered by tests
* Prefer correctness and clarity over cleverness

## Development Commands

### Building and Testing
```bash
# Build the project
cargo build

# Build in release mode
cargo build --release

# Run all tests
cargo test

# Run a single test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test integration_test_name

# Run specific test file
cargo test --test format_registry_tests

# Check code without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy

# Run clippy with all targets
cargo clippy --all-targets --all-features

# Run specific binary
cargo run --bin musicctl
cargo run --bin musicctl-mcp
```

## Project Structure

This is a Rust CLI tool for music library metadata management with MCP integration.

**Key Components:**
- `src/domain/` - Core models and traits (Artist, Album, Track, MetadataValue with source tracking)
- `src/services/` - Business logic (scanner, formats, normalization, inference, validation, duplicates)
- `src/cli/` - Command-line interface (`musicctl` binary)
- `src/mcp/` - MCP server integration (`musicctl-mcp` binary)
- `tests/` - Integration tests and fixtures (134+ tests across 22 test files)
- `tests/fixtures/` - Test audio files in FLAC, MP3, and WAV formats

**Current Binaries:**
- `musicctl` - Main CLI for music library operations
- `musicctl-mcp` - MCP server for AI agent integration

## Supported File Formats

### Currently Implemented (v1)

* `.flac` - Full metadata read/write support
* `.mp3` - Full metadata read/write support  
* `.wav` - Full metadata read/write support

### Future Scope (v2)

* `.dsf` - DSD audio format
* `.ogg` - Ogg Vorbis
* `.m4a` - Apple AAC format

Architecture uses traits for format extensibility - adding new formats requires implementing the `AudioFile` trait.

## Code Style Guidelines

### Imports and Dependencies
- Order imports: std, external crates, internal modules, current module
- Prefer `use crate::` for internal imports
- Group related imports together
- Use `serde::{Deserialize, Serialize}` for serde derives

### Naming Conventions
- **Types**: PascalCase (e.g., `FlacHandler`, `TrackMetadata`)
- **Functions**: snake_case (e.g., `read_metadata`, `infer_album_from_path`)
- **Constants**: SCREAMING_SNAKE_CASE
- **File names**: snake_case (e.g., `models.rs`, `flac.rs`)
- **Traits**: PascalCase with descriptive names (e.g., `AudioFile`)

### Error Handling
- Use custom error enums with `thiserror` or manual implementations
- All errors must implement `Display` and `Error`
- Use `Result<T, CustomError>` for fallible functions
- Provide context with error messages
- Handle `Option` types explicitly, avoid `unwrap()` in production code

### Types and Patterns
- Use `#[derive(Debug, Clone, Serialize, Deserialize)]` for data models
- Implement `Default` for structs with reasonable defaults
- Use `PathBuf` for file paths, `&Path` for function arguments
- Prefer `String` over `&str` in struct fields
- Use metadata wrapper pattern: `MetadataValue<T>` with source tracking (Embedded, FolderInferred, UserEdited)

### Code Organization
- Keep modules focused and small (<300 lines when possible)
- Use traits for format extensibility (`AudioFile` trait)
- Separate domain logic from implementation details
- Use re-exports in `lib.rs` for public API

### Testing Requirements
- Every public function must have tests
- Write unit tests in the same module with `#[cfg(test)]`
- Write integration tests in `tests/` directory
- Use `tempfile` for file system tests
- Test error cases, not just success paths
- Use fixtures in `tests/fixtures/` for consistent test data
- Copy fixture files to temp directories for write operations (don't modify originals)

### Documentation
- Add module-level documentation with `//!`
- Document public functions with `///`
- Include examples in doc comments for CLI commands
- Use `#[allow(dead_code)]` sparingly with justification

### CLI Development
- Use `clap` with derive macros for command parsing
- Structure commands as separate modules
- Provide helpful error messages for invalid input
- Support both human-readable and JSON output where appropriate
- Commands: scan, tree, read, write, normalize, emit, validate, duplicates, cue, cue-parse, cue-validate

### MCP Integration
- Follow MCP schema patterns for parameter structs
- Use `schemars` for JSON schema generation
- Handle JSON serialization/deserialization errors gracefully
- Provide clear error responses for MCP operations
- MCP tools: scan_directory, read_file_metadata, get_library_tree, emit_library_metadata, normalize_titles, validate_library, find_duplicates, generate_cue_file, parse_cue_file, validate_cue_file

## Design Principles

1. **Format Agnostic**: Use traits to support multiple audio formats
2. **Metadata Provenance**: Track where metadata comes from (embedded, inferred, user-set)
3. **Incremental Processing**: Support scanning new files without full rebuild
4. **AI-Friendly Output**: Structure output for machine readability
5. **Modular Architecture**: Keep components loosely coupled
6. **Error Recovery**: Handle partial failures gracefully
7. **Deterministic Behavior**: Same input yields same output
8. **Small, Composable Modules**: Unix philosophy - small, sharp, predictable

## Common Patterns

### Reading Metadata
```rust
fn read_metadata(&self, path: &Path) -> Result<Track, AudioFileError> {
    if !self.can_handle(path) {
        return Err(AudioFileError::UnsupportedFormat);
    }
    // Implementation using lofty crate
}
```

### Writing Metadata
```rust
fn write_metadata(&self, path: &Path, metadata: &TrackMetadata) -> Result<(), AudioFileError> {
    if !self.can_handle(path) {
        return Err(AudioFileError::UnsupportedFormat);
    }
    // Use lofty to write tags
}
```

### Error Creation
```rust
#[derive(Debug, Clone)]
pub enum CustomError {
    IoError(String),
    InvalidFormat(String),
}

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CustomError::IoError(msg) => write!(f, "I/O error: {}", msg),
            CustomError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
        }
    }
}
```

### Testing with Fixtures
```rust
#[test]
fn test_functionality() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.flac");
    fs::copy("tests/fixtures/flac/simple/track1.flac", &test_file).unwrap();
    // Test implementation - modify test_file, not the fixture
}
```

### Metadata Value Creation
```rust
// Embedded from file tags
let title = MetadataValue::embedded("Song Title".to_string());

// Inferred from directory structure  
let artist = MetadataValue::inferred("Artist Name".to_string(), FOLDER_INFERRED_CONFIDENCE);

// Set by user/cli
let album = MetadataValue::user_set("Album Name".to_string());
```

## Constraints

- CLI only, no GUI
- macOS and Linux support only
- No internet access or external metadata lookup
- Use `lofty` crate for audio file metadata
- Follow existing MCP patterns for server integration
- All public APIs must be tested

## AI-Agent Awareness

This tool is designed to be called by AI agents via MCP.

Therefore:

* CLI output should be easy to parse
* Commands should be composable and scriptable
* Errors should be structured and descriptive
* Avoid ambiguous or human-only output unless explicitly requested
* Support JSON output for machine readability

## Working Style Instructions

* Ask **at most one** clarifying question if absolutely necessary
* Otherwise, make reasonable assumptions and state them
* Do not over-engineer
* Do not introduce features outside the defined scope
* Keep changes minimal and focused

## Current Version Status

**v1 - Foundation (COMPLETE)**

✅ Recursive directory scanning
✅ Artist → Album → Track hierarchy inference
✅ CLI tree structure display
✅ CLI metadata read from FLAC/MP3/WAV
✅ CLI metadata write to FLAC/MP3/WAV
✅ MCP server with all core tools
✅ Title normalization
✅ Duplicate detection by checksum
✅ Validation of metadata completeness
✅ .cue file generation from track metadata

**v2 - Future Enhancements**

* Additional audio format support (DSF, OGG, M4A)
* Batch metadata operations
* Genre normalization
* CUE file validation

---

**Summary**: You are building a precise, local, metadata-focused music library tool that serves both humans and AI agents. Design it like a Unix tool: small, sharp, predictable, composable. When in doubt, choose simplicity.
