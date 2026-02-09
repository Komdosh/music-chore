# Music Library Organizer - App Specification

## Project Overview
A Rust-based CLI tool that organizes and normalizes local music libraries using existing file metadata and directory structure only.

## Core Goal
Design a CLI-only Rust program that:
- Recursively scans a working directory
- Discovers music files and albums
- Reads and writes metadata
- Reads, writes, and validates .cue files
- Exposes structured, machine-readable output suitable for AI agents
- Supports incremental updates when new folders are added
- **SUPPORTS MULTIPLE AUDIO FORMATS** (FLAC, MP3, WAV, DSF, WavPack with extensible architecture)

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
### v0.3.2 (Current Scope - IMPLEMENTED)
- **.flac** files ✅
- **.mp3** files ✅ (ID3v2 tag support)
- **.wav** files ✅ (INFO chunk support)
- **.dsf** files ✅ (ID3 tag reading, read-only)
- **.wv** (WavPack) files ✅

### v2 — AI Agent Integration ✅ COMPLETED
- ✅ MCP awareness (agent-friendly command structure and output)
- ✅ Complete MCP server with 8 core tools
- ✅ 18 Expert AI prompts for library analysis and maintenance
- ✅ Proper initialization and shutdown handling
- ✅ Claude Desktop, Gemini CLI, and Qwen integration
- ✅ JSON-RPC protocol implementation
- ✅ Comprehensive MCP test coverage

### Additional Features Implemented ✅
- **Unicode Support**: Full support for non-ASCII characters in paths and metadata
- **Duplicate Detection**: SHA256-based duplicate track identification
- **Metadata Validation**: Comprehensive validation against defined schema
- **Normalization**: Convert track titles to proper title case and map genres to standards
- **CUE Support**: Generate, parse, and validate CUE sheets
- **Dry-run Mode**: Default behavior for write operations to ensure safety
- **Comprehensive Testing**: 640+ integration and unit tests

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
5. **AI-friendly output** (structured, predictable, machine-readable)
6. **Provenance tracking** — explicitly mark sources of metadata
7. **Security first** — restrict access to allowed paths in MCP

## Current Architecture (IMPLEMENTED)

```
src/
├── bin/                           # CLI binaries
│   ├── musicctl.rs               # Main CLI tool
│   └── musicctl-mcp.rs          # MCP server
├── core/                         # Business logic & Domain
│   ├── domain/                   # Models & Schema (Artist, Album, Track)
│   ├── services/                 # Operation implementations
│   └── logging.rs                # Centralized logging
├── adapters/                     # I/O Adapters
│   └── audio_formats/            # Format-specific handlers (AudioFile trait)
├── presentation/                 # Interface layer
│   └── cli/                      # CLI parser and commands
└── mcp/                          # MCP server implementation
    ├── music_chore_server.rs     # Server routing
    ├── music_chore_server_impl.rs # Tool & Prompt logic
    ├── prompts.rs                # AI prompt templates
    └── config.rs                 # Environment & Security configuration
```

## CLI Commands (IMPLEMENTED)

### Core Commands
- `scan <path>` - Recursively scan directory, output file paths
- `tree <path>` - Show hierarchical library view with format indicators
- `read <file>` - Read metadata from individual files
- `write <file> --set "key=value" --apply` - Update metadata (dry-run by default)
- `normalize <path> --apply` - Standardize titles and genres
- `validate <path>` - Check metadata completeness and consistency
- `duplicates <path>` - Find duplicate tracks by checksum
- `cue <operation> <path>` - Generate, parse, or validate CUE sheets
- `emit <path> --json` - Export complete library metadata

## MCP Server Tools (IMPLEMENTED)

1. **scan_directory** - Scan directories for music files
2. **get_library_tree** - Get hierarchical library view
3. **read_file_metadata** - Read metadata from individual files
4. **normalize** - Normalize track titles and genres
5. **emit_library_metadata** - Get complete structured library data
6. **validate_library** - Validate metadata completeness
7. **find_duplicates** - Detect duplicate audio files
8. **cue_file** - CUE sheet operations

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
    Embedded,      // From file tags
    FolderInferred, // From path analysis
    UserEdited,     // From manual updates
}
```

## AI-Agent Awareness

Assume this tool will be called by another AI agent via MCP.

Therefore:
- CLI output should be easy to parse
- MCP tools must return clear success/error results
- 18 specialized prompts guide agent analysis (Discovery, Health, Narrative)
- Security constraints prevent unauthorized filesystem access
- Deterministic behavior ensures reliable agent reasoning

## Summary

You are building a **precise, local, metadata-focused music library tool** that serves both humans and AI agents.

Design it like a Unix tool:
- Small
- Sharp  
- Predictable
- Composable
- Multi-format

**Current State**: Production-ready v0.3.2 with multi-format support, CUE operations, 18 expert prompts, and 640+ tests.
