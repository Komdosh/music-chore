# AGENTS.md - Coding Agent Guidelines

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
- `src/domain/` - Core models and traits (Artist, Album, Track, AudioFile trait)
- `src/services/` - Business logic (scanner, formats, normalization, inference)
- `src/cli/` - Command-line interface (`musicctl` binary)
- `src/mcp/` - MCP server integration (`musicctl-mcp` binary)
- `tests/` - Integration tests and fixtures

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
- Use metadata wrapper pattern: `MetadataValue<T>` with source tracking

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

### MCP Integration
- Follow MCP schema patterns for parameter structs
- Use `schemars` for JSON schema generation
- Handle JSON serialization/deserialization errors gracefully
- Provide clear error responses for MCP operations

## Design Principles

1. **Format Agnostic**: Use traits to support multiple audio formats
2. **Metadata Provenance**: Track where metadata comes from (embedded, inferred, user-set)
3. **Incremental Processing**: Support scanning new files without full rebuild
4. **AI-Friendly Output**: Structure output for machine readability
5. **Modular Architecture**: Keep components loosely coupled
6. **Error Recovery**: Handle partial failures gracefully

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
    // Test implementation
}
```

## Constraints

- CLI only, no GUI
- macOS and Linux support only
- No internet access or external metadata lookup
- Use `lofty` crate for audio file metadata
- Follow existing MCP patterns for server integration
- All public APIs must be tested