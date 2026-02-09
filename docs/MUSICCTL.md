# 🎵 musicctl - Complete User Guide

`musicctl` is the command-line interface for the music-chore library management system. This guide covers all commands, options, and usage patterns.

## 📅 Last Updated

- **Date**: February 9, 2026
- **Version**: v0.3.2
- **Features**: CLI with 10 commands + MCP server with 8 tools & 18 prompts

---

## ⚡ Quick Start

### 1. Scan Your Music Library
```bash
# Basic scan
musicctl scan ~/Music

# Scan with progress output
musicctl scan ~/Music --verbose
```

### 2. View Library Structure
```bash
# Tree view with emojis
musicctl tree ~/Music
```

### 3. Normalize Metadata
```bash
# Preview changes (titles and genres)
musicctl normalize ~/Music

# Apply changes to files
musicctl normalize ~/Music --apply
```

### 4. Manage CUE Sheets
```bash
# Generate CUE sheet for an album
musicctl cue --generate /path/to/album
```

---

## 📖 Command Reference

### `scan` - Discover Music Files
Recursively scan for FLAC, MP3, WAV, DSF, and WavPack files.
- `--verbose`: Show progress bars and file counts.
- `--json`: Output machine-readable path list.

### `tree` - Display Hierarchy
Show Artist -> Album -> Track structure inferred from metadata and paths.
- `--json`: Get the full tree as a JSON object.

### `read` - Extract Metadata
Read full tags from a single file.
- `--compact`: One-line summary.
- `--json`: Full provenance and confidence data.

### `write` - Update Metadata
Manually update tags in a file.
- `--set "artist=New Artist"`: Update specific field.
- `--apply`: Required to actually write to disk (defaults to dry-run).

### `normalize` - Standardize Library
Standardize titles to Title Case and map genres to a standard list.
- `--apply`: Commit changes to file tags.

### `validate` - Check Quality
Check for missing fields, inconsistent naming, or unusual durations.
- `--json`: Get a detailed validation report.

### `duplicates` - Find Redundant Files
Locate identical audio files using SHA256 checksums.

### `cue` - CUE Sheet Operations
- `--generate`: Create `.cue` from track metadata.
- `--parse`: Read and display `.cue` content.
- `--validate`: Check if audio files match the `.cue` definitions.

### `emit` - Export Data
Export the entire library metadata in structure format optimized for AI.

---

## 🔧 Pro Tips

### JQ Integration
```bash
# Get all tracks from a specific year
musicctl emit ~/Music --json | jq '.artists[].albums[] | select(.year == 1994) | .tracks[].title'
```

### Security & Safety
- **Dry-run by Default**: All commands that modify files require `--apply` or `--force`.
- **Allowed Paths**: The MCP server respects `MUSIC_ALLOWED_PATHS` to keep the AI within your music directory.

### Performance
For very large libraries, use `musicctl scan` first to check for any accessibility issues before running complex analysis like `duplicates`.
