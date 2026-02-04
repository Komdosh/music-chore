# 🎵 music-chore

<div align="center">

**A precision CLI tool for organizing and normalizing local music libraries**

[![Rust](https://img.shields.io/badge/rust-2024-blue.svg)](https://www.rust-lang.org)
[![macOS](https://img.shields.io/badge/platform-macOS-lightgrey.svg)](https://www.apple.com/macos)
[![linux](https://img.shields.io/badge/platform-linux-lightgrey.svg)](https://www.linux.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

*Built by AI agents for AI agents, perfect for humans 🤖 → 👤*

</div>

---

## ✨ Features

<div align="center">

| Feature | Description |
|:--------:|:------------|
| 🔍 | Recursive directory scanning |
| 🏷️ | Metadata extraction (FLAC, MP3, WAV) |
| 📂 | Artist → Album → Track inference |
| 🔤 | Title/genre normalization |
| 🌳 | Tree visualization |
| 🔄 | Duplicate detection (SHA256) |
| 📊 | Structured output for AI/MCP |
| 📝 | CUE file operations |

</div>

---

## 🚀 Quick Start

### Installation

```bash
# Install music-chore CLI
curl -fsSL https://github.com/Komdosh/music-chore/releases/latest/download/install.sh | bash

# Install MCP server for AI agents
curl -fsSL https://github.com/Komdosh/music-chore/releases/latest/download/install_mcp.sh | bash
```

### Your First Scan

```bash
# Scan your music library
musicctl scan /path/to/your/music

# View the structure
musicctl tree /path/to/your/music

# Check metadata quality
musicctl validate /path/to/your/music
```

---

## 📖 Usage

### Core Commands

| Command | Description | Example |
|---------|-------------|---------|
| `scan` | Discover music files | `musicctl scan ~/Music` |
| `tree` | Visual library view | `musicctl tree ~/Music` |
| `read` | Extract file metadata | `musicctl read track.flac` |
| `write` | Update metadata | `musicctl write track.flac --title "New Title"` |
| `normalize` | Title case normalization | `musicctl normalize ~/Music` |
| `normalize-genres` | Genre standardization | `musicctl normalize-genres ~/Music` |
| `validate` | Check metadata quality | `musicctl validate ~/Music` |
| `duplicates` | Find duplicate files | `musicctl duplicates ~/Music` |
| `emit` | Export structured metadata | `musicctl emit ~/Music --json` |

### CUE Operations

```bash
# Generate CUE sheet from album
musicctl cue --generate /path/to/album

# Parse existing CUE file
musicctl cue --parse /path/to/album.cue

# Validate CUE against audio files
musicctl cue --validate /path/to/album.cue
```

### Dry Run Mode

```bash
# See what would change (no modifications)
musicctl normalize /path/to/music --dry-run
musicctl normalize-genres /path/to/music --dry-run
```

---

## 🤖 MCP Server

AI agents can integrate directly with music-chore via MCP (Model Context Protocol).

### Setup

```bash
# Add to Claude Desktop
claude mcp add music-chore -- musicctl-mcp
```

### Available Tools (9 total)

| Tool | Purpose |
|------|---------|
| `scan_directory` | Discover music files recursively |
| `get_library_tree` | Get hierarchical library view |
| `read_file_metadata` | Extract metadata from audio files |
| `normalize_titles` | Fix title capitalization |
| `normalize_genres` | Standardize genre names |
| `emit_library_metadata` | Full library export (JSON) |
| `validate_library` | Check metadata completeness |
| `find_duplicates` | Detect duplicate files |
| `cue_file` | Generate/parse/validate CUE sheets |

---

## 🏗️ Architecture

```
music-chore/
├── src/
│   ├── domain/           # Core models (Artist, Album, Track)
│   ├── infrastructure/    # Scanner, format handlers
│   ├── services/         # Business logic & operations
│   ├── cli/              # Command-line interface
│   └── mcp/             # MCP server integration
├── tests/
│   ├── fixtures/         # Test audio files
│   └── integration/     # Integration tests
└── Cargo.toml
```

**Extensible Design:** Add new audio formats by implementing the `AudioFile` trait.

---

## 🛠️ Development

```bash
# Build the project
cargo build

# Build release version
cargo build --release

# Run all tests (165+ tests)
cargo test

# Run specific test
cargo test test_name

# Check code without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy
```

---

## 📦 Supported Formats

| Format | Read | Write |
|--------|:----:|:-----:|
| FLAC | ✅ | ✅ |
| MP3 | ✅ | ✅ |
| WAV | ✅ | ✅ |
| DSF | 🔜 | 🔜 |
| OGG | 🔜 | 🔜 |
| M4A | 🔜 | 🔜 |

---

## 🎵 Genre Normalization

Maps 40+ genre variants to standards:

| Input | Output |
|-------|--------|
| `"rock and roll"` | `"Rock"` |
| `"hip hop"` | `"Hip-Hop"` |
| `"smooth jazz"` | `"Jazz"` |
| `"electronic dance music"` | `"Electronic"` |
| `"r&b"` | `"R&B"` |


---

<div align="center">

**Made with ❤️ for music lovers and AI agents**

⭐ Star us on GitHub | 🐛 Report issues | 💡 Feature requests

</div>
