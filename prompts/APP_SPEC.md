# Music Library Organizer - App Specification

## Project Overview
A Rust-based CLI tool that organizes and normalizes local music libraries using existing file metadata and directory structure only.

## Core Goal
Design a CLI-only Rust program that:
- Recursively scans a working directory
- Discovers music files and albums
- Reads and writes metadata
- Reads and writes .cue files
- Exposes structured, machine-readable output suitable for AI agents
- Supports incremental updates when new folders are added
- **NOW SUPPORTS MULTIPLE AUDIO FORMATS** (FLAC, MP3, WAV with extensible architecture)

## Hard Constraints
- CLI only (no GUI)
- Rust language
- No internet access or external metadata lookup
- No playback functionality
- Modular architecture (future formats must be easy to add)
- Every public function must be covered by tests
- Prefer correctness and clarity over cleverness
- **MCP server integration for AI agents**

## Supported File Formats
### v0.2 (Current Scope - IMPLEMENTED)
- **.flac** files ✅
- **.mp3** files ✅ (ID3v2 tag support)
- **.wav** files ✅ (INFO chunk support)

### v1 (Future Scope)
- .dsf

## Versioned Feature Scope

### v1 — Foundation ✅ COMPLETED
- Recursive directory scanning ✅
- Artist → Album → Track hierarchy inference ✅
- CLI command to display a tree structure ✅
  * Artist
    * Album
      * Tracks
- CLI command to read metadata from .flac file ✅
- CLI command to write/update metadata in .flac file ✅
- Internals designed to support multiple formats later ✅

### v0.1.9 — MP3 Support ✅ COMPLETED
- MP3 format support with ID3v2 tag reading/writing ✅
- Multi-format architecture working ✅
- All CLI commands work with both FLAC and MP3 ✅
- Format-agnostic scanner and metadata processing ✅

### v0.2.1 — WAV Support ✅ COMPLETED
- WAV format support with INFO chunk reading/writing ✅
- Multi-format architecture extended to three formats ✅
- All CLI commands work with FLAC, MP3, and WAV ✅
- Format-agnostic scanner and metadata processing ✅

### v2 — AI Agent Integration ✅ COMPLETED
**✅ COMPLETED in v1.1**:
- ✅ MCP awareness (agent-friendly command structure and output)
- ✅ Complete MCP server with 6 core tools
- ✅ Proper initialization and shutdown handling
- ✅ Claude Desktop integration via CLI
- ✅ JSON-RPC protocol implementation
- ✅ Comprehensive MCP test coverage (18 tests)

**Planned for future versions**:
- Support additional audio formats (DSF)
- Generate a .cue file for an album folder using track metadata
- Advanced metadata operations

### Additional Features Implemented ✅
- **Unicode Support**: Full support for non-ASCII characters in paths and metadata
- **Duplicate Detection**: SHA256-based duplicate track identification
- **Metadata Validation**: Comprehensive validation of required and recommended fields
- **Title Normalization**: Convert track titles to proper title case
- **Dry-run Mode**: Safe preview of all write operations
- **Comprehensive Testing**: 67+ integration and unit tests

## Explicit Non-Goals
Do NOT:
- Fetch metadata from the internet
- Use MusicBrainz, Discogs, or similar databases
- Guess artist/album info beyond local inference
- Implement media playback
- Implement a GUI

## Design Principles
Follow these principles strictly:

1. **Small, composable modules**
2. **Clear data models** for Artist, Album, Track, Metadata
3. **Format-agnostic interfaces** (traits) for audio files
4. **Deterministic behavior** — same input yields same output
5. **AI-friendly output** (structured, predictable, machine-readable when useful)
6. **Incremental processing** — new folders can be scanned without full rebuild
7. **Multi-format support** — architecture must handle multiple audio formats seamlessly

## Current Architecture (IMPLEMENTED)

```
src/
├── bin/                           # CLI binaries
│   ├── musicctl.rs               # Main CLI tool
│   └── musicctl-mcp.rs          # MCP server
├── cli/                          # CLI layer
│   ├── mod.rs
│   ├── commands.rs               # Command definitions
│   └── commands_processor.rs     # Command handling
├── domain/                       # Pure business logic
│   ├── mod.rs
│   ├── models.rs                 # Artist, Album, Track, Metadata
│   └── traits.rs                # AudioFile trait (format-agnostic)
├── services/                     # Business services
│   ├── mod.rs
│   ├── scanner.rs                # Format-agnostic directory scanning
│   ├── library.rs               # Library hierarchy building
│   ├── format_tree.rs           # Tree formatting utilities
│   ├── duplicates.rs            # Duplicate detection
│   ├── inference.rs             # Path-based metadata inference
│   ├── normalization.rs        # Text normalization
│   ├── validation.rs           # Metadata validation
│   └── formats/               # Audio format implementations
│       ├── mod.rs              # Format registry
│       ├── flac.rs             # FLAC format handler
│       └── mp3.rs              # MP3 format handler (ID3v2)
└── mcp/                         # MCP server implementation
    ├── mod.rs
    ├── music_chore_server.rs   # Main MCP server
    └── params.rs              # MCP parameter definitions
```

## CLI Commands (IMPLEMENTED)

### Core Commands
- `scan <path>` - Recursively scan directory, output file paths
- `tree <path>` - Show hierarchical library view with format indicators
- `read <file>` - Read metadata from individual files (FLAC/MP3)
- `write <file> --set "key=value" --apply` - Update metadata (dry-run by default)
- `normalize <path>` - Normalize track titles to title case
- `emit <path>` - Emit structured metadata (JSON or AI-friendly text)

### Additional Commands
- `validate <path>` - Check metadata completeness and consistency
- `duplicates <path>` - Find duplicate tracks by checksum
- `--version` - Show version information
- `help` - Show help

### Global Options
- `--json` - JSON output format (where applicable)
- `--dry-run` - Preview changes without applying (write command)
- `--apply` - Apply changes (write command)

## MCP Server Tools (IMPLEMENTED)

1. **scan_directory** - Scan directories for music files
2. **get_library_tree** - Get hierarchical library view
3. **read_file_metadata** - Read metadata from individual files
4. **normalize_titles** - Normalize track titles to title case
5. **emit_library_metadata** - Get complete structured library data
6. **validate_library** - Validate metadata completeness

All tools support both FLAC and MP3 formats.

## Data Models (IMPLEMENTED)

### Core Metadata Model (Format-Agnostic)

```rust
TrackMetadata {
    title: Option<MetadataValue<String>>,
    artist: Option<MetadataValue<String>>,
    album: Option<MetadataValue<String>>,
    album_artist: Option<MetadataValue<String>>,
    track_number: Option<MetadataValue<u32>>,
    disc_number: Option<MetadataValue<u32>>,
    year: Option<MetadataValue<u32>>,
    genre: Option<MetadataValue<String>>,
    duration: Option<MetadataValue<f64>>,
    format: String,
    path: PathBuf,
}

MetadataValue<T> {
    value: T,
    source: MetadataSource,
    confidence: f32,
}

enum MetadataSource {
    Embedded,      // From file metadata
    FolderInferred, // From directory structure
    UserEdited,     // From manual updates
}
```

### Audio Format Abstraction

```rust
trait AudioFile: Send + Sync {
    fn can_handle(&self, path: &Path) -> bool;
    fn supported_extensions(&self) -> Vec<&'static str>;
    fn read_metadata(&self, path: &Path) -> Result<Track, AudioFileError>;
    fn write_metadata(&self, path: &Path, metadata: &TrackMetadata) -> Result<(), AudioFileError>;
    fn read_basic_info(&self, path: &Path) -> Result<TrackMetadata, AudioFileError>;
}
```

### Library Representation

```rust
Library {
    artists: Vec<ArtistNode>,
    total_artists: usize,
    total_albums: usize,
    total_tracks: usize,
}

ArtistNode {
    name: String,
    albums: Vec<AlbumNode>,
}

AlbumNode {
    title: String,
    year: Option<u32>,
    tracks: Vec<TrackNode>,
}

TrackNode {
    file_path: PathBuf,
    metadata: TrackMetadata,
}
```

## Expected Planning Outputs
When asked to plan or design, produce:

1. High-level architecture diagrams (conceptual, not graphical)
2. Proposed module layout
3. Core data structures and traits
4. CLI command and flag design
5. Step-by-step implementation plan
6. Testing strategy
7. Design trade-offs and rationale

Avoid vague statements. Prefer concrete decisions.

## AI-Agent Awareness

Assume this tool will be called by another AI agent via MCP.

Therefore:
- CLI output should be easy to parse
- Commands should be composable and scriptable
- Errors should be structured and descriptive
- Avoid ambiguous or human-only output unless explicitly requested
- Support both JSON and AI-friendly text formats
- Ensure deterministic behavior for agent reasoning

## Working Style Instructions

- Ask at most one clarifying question if absolutely necessary
- Otherwise, make reasonable assumptions and state them
- Do not over-engineer
- Do not introduce features outside the defined scope
- Keep v1 minimal and rock-solid
- Maintain backward compatibility
- Add comprehensive tests for new features
- Update documentation with each new feature

## Summary

You are building a **precise, local, metadata-focused music library tool** that serves both humans and AI agents.

Design it like a Unix tool:
- Small
- Sharp  
- Predictable
- Composable
- Multi-format

**Current State**: Production-ready v0.2.0 with FLAC+MP3+WAV support, complete CLI, MCP server, and comprehensive testing.

When in doubt, choose simplicity and maintain the high quality standards established by the existing implementation.