# music-chore Project Context

## Project Overview

**music-chore** is a precision CLI tool for organizing and normalizing local music libraries, written in Rust. It's designed as a deterministic, AI-friendly music metadata compiler that works with various audio formats including FLAC, MP3, WAV, DSF, and WavPack.

The project serves dual purposes:
1. A command-line interface (`musicctl`) for manual music library management
2. An MCP (Model Context Protocol) server (`musicctl-mcp`) for AI agent integration

## Architecture

The project follows a modular, clean architecture with distinct layers:

- **Domain Layer** (`src/core/domain`): Contains core data models like `Track`, `AlbumNode`, `ArtistNode`, and `Library`
- **Services Layer** (`src/core/services`): Implements business logic for scanning, normalization, validation, etc.
- **Adapters Layer** (`src/adapters`): Handles external integrations and format-specific operations
- **Presentation Layer** (`src/presentation/cli`): CLI command definitions and processing
- **MCP Layer** (`src/mcp`): Model Context Protocol server for AI agent integration

## Key Features

### CLI Commands
- `scan`: Recursively scan directories for music files
- `tree`: Display library hierarchy in tree format
- `read`: Extract metadata from individual files
- `write`: Update metadata with dry-run capabilities
- `normalize`: Normalize track titles and genres to standard formats
- `emit`: Export library metadata in structured JSON format
- `validate`: Check metadata completeness and consistency
- `duplicates`: Find duplicate tracks by checksum
- `cue`: Generate, parse, or validate CUE files

### Data Models
- **Track**: Represents individual music files with path and metadata
- **TrackMetadata**: Contains title, artist, album, year, genre, etc. with provenance tracking
- **MetadataValue**: Wraps values with source information (embedded, inferred, user-edited)
- **Library**: Hierarchical structure of Artists → Albums → Tracks

### AI Integration (MCP)
The project includes an MCP server that exposes 8 tools for AI agents:
- `scan_directory`, `get_library_tree`, `read_file_metadata`
- `normalize`, `emit_library_metadata`, `validate_library`
- `find_duplicates`, `cue_file`

## Building and Running

### Prerequisites
- Rust toolchain (cargo, rustc)

### Build Commands
```bash
# Build debug version
cargo build

# Build release version
cargo build --release

# Run tests (165+ tests)
cargo test

# Format code
cargo fmt

# Run linter
cargo clippy
```

### Running the CLI
```bash
# After building, run:
./target/debug/musicctl --help

# Example usage:
./target/debug/musicctl scan ~/Music
./target/debug/musicctl tree ~/Music
```

## Development Conventions

- **Clean Architecture**: Clear separation between domain, services, adapters, and presentation
- **Error Handling**: Comprehensive error handling with custom error types
- **Testing**: Extensive test suite with 165+ tests covering unit and integration scenarios
- **Format Support**: Extensible design allowing new audio formats via the `AudioFile` trait
- **Dry Run Capability**: Most write operations support dry-run mode for previewing changes

## File Structure
```
music-chore/
├── src/
│   ├── core/           # Domain models and business logic
│   │   ├── domain/     # Data structures (Track, Album, etc.)
│   │   └── services/   # Business logic implementations
│   ├── adapters/       # External integrations and format handlers
│   ├── presentation/   # CLI interface
│   │   └── cli/        # Command definitions and processors
│   └── mcp/           # Model Context Protocol server
├── tests/             # Comprehensive test suite
├── Cargo.toml         # Rust project configuration
└── README.md          # Detailed documentation
```

## Supported Audio Formats
- FLAC (Read/Write)
- MP3 (Read/Write)
- WAV (Read/Write)
- DSF (Read/Write)
- WavPack (Read/Write)

## Special Features
- **Genre Normalization**: Maps 40+ genre variants to standardized formats
- **Metadata Provenance**: Tracks source of metadata (embedded, inferred, user-edited)
- **Checksum Validation**: Detects duplicate files using SHA256 checksums
- **CUE File Operations**: Generate, parse, and validate CUE sheet files
- **Hierarchical Organization**: Builds artist → album → track hierarchies from file structure