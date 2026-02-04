# 🎵 music-chore

**A precision CLI tool for organizing and normalizing local music libraries**

[![Rust](https://img.shields.io/badge/rust-2024-blue.svg)](https://www.rust-lang.org)
[![macOS](https://img.shields.io/badge/platform-macOS-lightgrey.svg)](https://www.apple.com/macos)
[![linux](https://img.shields.io/badge/platform-linux-lightgrey.svg)](https://www.linux.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

*Built for AI agents, perfect for humans*

---

## Overview

Local-only CLI tool for music library management. No internet, no external services.

**Features:**
- 🔍 Recursive directory scanning
- 🏷️ Metadata extraction (FLAC, MP3, WAV)
- 📂 Artist → Album → Track inference
- 🔤 Title/genre normalization
- 🌳 Tree visualization
- 🔄 Duplicate detection (SHA256)
- 📊 Structured output for AI/MCP
- 📝 CUE file operations

**Install:**
```bash
curl -fsSL https://github.com/Komdosh/music-chore/releases/latest/download/install.sh | bash
```

---

## Commands

| Command | Description |
|---------|-------------|
| `scan` | Discover music files |
| `tree` | Visual library view |
| `read` | Extract file metadata |
| `write` | Update metadata |
| `normalize` | Title case normalization |
| `normalize-genres` | Genre normalization |
| `validate` | Check metadata quality |
| `duplicates` | Find duplicate files |
| `cue` | Generate/parse/validate CUE files |
| `emit` | Export structured metadata |

### Quick Examples

```bash
# Scan and tree view
musicctl scan /path/to/music
musicctl tree /path/to/music

# Read metadata
musicctl read /path/to/track.flac

# Normalize (dry run first!)
musicctl normalize /path/to/music --dry-run
musicctl normalize /path/to/music

# Validate library
musicctl validate /path/to/music

# Find duplicates
musicctl duplicates /path/to/music

# CUE operations
musicctl cue --generate /path/to/album
musicctl cue --parse /path/to/album.cue
musicctl cue --validate /path/to/album.cue

# Export for AI
musicctl emit /path/to/music --json
```

### CUE Command

```bash
musicctl cue --generate /path/to/album        # Generate CUE file
musicctl cue --parse /path/to/album.cue        # Parse CUE contents
musicctl cue --validate /path/to/album.cue     # Validate against audio files
```

### Genre Normalization

Maps variants to standards: `"rock and roll"` → `"Rock"`, `"hip hop"` → `"Hip-Hop"`, `"smooth jazz"` → `"Jazz"`. Supports 40+ genres.

---

## MCP Server

Install for AI agents:
```bash
# Install MCP
curl -fsSL https://github.com/Komdosh/music-chore/releases/latest/download/install_mcp.sh | bash

# Claude
claude mcp add music-chore -- musicctl-mcp
```

**Tools (8 total):**
| Tool | Purpose |
|------|---------|
| `scan_directory` | Discover music files |
| `get_library_tree` | Library hierarchy |
| `read_file_metadata` | Extract metadata |
| `normalize_titles` | Fix capitalization |
| `normalize_genres` | Standardize genres |
| `emit_library_metadata` | Full export |
| `validate_library` | Check quality |
| `find_duplicates` | Find dupes |
| `cue_file` | Generate/parse/validate CUE |

---

## Architecture

```
src/
├── domain/           # Models (Artist, Album, Track)
├── infrastructure/  # Scanner, format handlers
├── services/         # Business logic
├── cli/              # Command-line interface
└── mcp/             # MCP server
```

**Extensible:** Add formats by implementing `AudioFile` trait.

---

## Development

```bash
cargo build              # Build
cargo test               # Run tests
cargo run --bin musicctl # Quick test
```

---

## License

MIT
