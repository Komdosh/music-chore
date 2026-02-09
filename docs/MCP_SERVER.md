# Music Chore MCP Server

The Model Context Protocol (MCP) server for Music Chore provides AI agents with programmatic access to music library management capabilities. It exposes the core functionality of the `musicctl` CLI tool through a standardized MCP interface.

## ✅ Status: Production Ready

The MCP server is **fully functional and tested** with:
- ✅ Complete MCP protocol implementation using rmcp SDK
- ✅ All 8 core tools exposed and working
- ✅ 18 Expert AI Prompts for complex workflows
- ✅ Environment-based security (allowed paths restriction)
- ✅ AI-friendly structured output (JSON and text formats)
- ✅ Multi-format support: FLAC, MP3, WAV, DSF, WavPack

## Overview

The MCP server allows AI agents to:
- Scan directories for music files recursively
- Analyze metadata with source provenance tracking
- Visualize library hierarchies (Artist -> Album -> Track)
- Standardize titles and genres using the `normalize` tool
- Detect duplicate files using SHA256 checksums
- Generate, parse, and validate CUE sheets
- Run complex analysis via specialized AI Prompts

## Installation

### Quick Install (Recommended)

```bash
# Automated local setup
curl -fsSL https://github.com/Komdosh/music-chore/releases/latest/download/install_mcp.sh | bash
```

### Install in your AI Agent

**Claude CLI:**
```bash
claude mcp add -e MUSIC_LIBRARY_PATH="/path/to/music" music-chore -- musicctl-mcp
```

**Gemini CLI:**
```bash
gemini mcp add -e MUSIC_LIBRARY_PATH="/path/to/music" music-chore musicctl-mcp
```

**Qwen:**
```bash
qwen mcp add music-chore musicctl-mcp -e MUSIC_LIBRARY_PATH="/path/to/music"
```

## Available Tools (8)

### 1. `scan_directory`
Recursively scan for music files.
- `path` (string): Path to scan.
- `json_output` (bool): Get full track objects instead of paths.

### 2. `get_library_tree`
Get hierarchical organization of the library.
- `path` (string): Root directory.

### 3. `read_file_metadata`
Read full metadata from a single file.
- `path` (string): Path to audio file.

### 4. `normalize`
Standardize titles (Title Case) and genres (standard mapping).
- `path` (string): Directory to analyze.
- `json_output` (bool): Get structured before/after reports.

### 5. `emit_library_metadata`
Export complete structured metadata for the entire library.

### 6. `validate_library`
Check for missing fields, track mismatches, and schema violations.

### 7. `find_duplicates`
Find identical audio data using SHA256 checksums.

### 8. `cue_file`
Unified tool for `.cue` operations (`generate`, `parse`, `validate`).

## Expert Prompts (18)

Expert prompts provide the AI agent with a strategy and the necessary tool calls to perform high-level tasks.

| Category | Prompts |
|----------|---------|
| **Analysis** | `top-tracks-analysis`, `genre-breakdown`, `decade-analysis`, `collection-story`, `artist-deep-dive` |
| **Discovery** | `instrument-to-learn`, `similar-artists-discovery`, `mood-playlist`, `hidden-gems`, `album-marathon`, `concert-setlist` |
| **Maintenance** | `library-health-check`, `metadata-cleanup-guide`, `duplicate-resolution`, `reorganization-plan`, `format-quality-audit`, `year-in-review`, `cue-sheet-assistant` |

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Logging level (error, warn, info, debug) | `info` |
| `MUSIC_LIBRARY_PATH` | Default path if none provided to tools | None |
| `MUSIC_SCAN_TIMEOUT` | Timeout for large scan operations (seconds) | `300` |
| `MUSIC_ALLOWED_PATHS` | Whitelist of paths the AI is allowed to access | All |

## Security

The MCP server implements **Path-based Security**:
- If `MUSIC_ALLOWED_PATHS` is set, the server will block any attempts to access files outside those directories.
- This protects your private system files from accidental or intentional access by AI agents.

## Development & Testing

You can test the MCP server manually using `rlwrap`:

```bash
rlwrap musicctl-mcp
```

Initialize command:
```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test"}}}
```