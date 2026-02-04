# Music Chore MCP Server

The Model Context Protocol (MCP) server for Music Chore provides AI agents with programmatic access to music library management capabilities. It exposes the core functionality of the `musicctl` CLI tool through a standardized MCP interface.

## ✅ Status: Production Ready

The MCP server is **fully functional and tested** with:
- ✅ Complete MCP protocol implementation using rmcp SDK
- ✅ All 8 core tools exposed and working
- ✅ Proper initialization and shutdown handling
- ✅ Comprehensive error handling and parameter validation
- ✅ AI-friendly structured output (JSON and text formats)
- ✅ Duplicate detection with SHA256 checksums
- ✅ CUE file generation, parsing, and validation

## Overview

The MCP server allows AI agents to:
- Scan directories for music files
- Read and analyze metadata from individual files
- Get hierarchical library tree views with format indicators
- Normalize track titles automatically
- Find duplicate tracks by checksum
- Emit structured library metadata for analysis
- Validate library metadata quality
- Generate, parse, and validate CUE files

## Available Tools

## Installation

### Quick Install (Recommended)

```bash
# Automated local setup sudo password required
curl -fsSL https://github.com/Komdosh/music-chore/releases/latest/download/install_mcp.sh | bash
```

#### Install in you agent:

Claude CLI:

```bash
claude mcp add music-chore -- musicctl-mcp
```

**🎯 Why CLI Method is Better:**
- ✅ No manual file editing required
- ✅ Automatic path detection and validation
- ✅ Safe backup and restore of configuration
- ✅ Works even if config file doesn't exist yet
- ✅ Automatic PATH setup

### Build from Source

```bash
git clone <repository-url>
cd music-chore
cargo build --release
```

The MCP server binary will be available at `target/release/musicctl-mcp`.

Or you can install it with:

```bash
git clone <repository-url>
cd music-chore
cargo install --path .
```

To run it in terminal with just a name `musicctl-mcp`.

## Usage

### Verify that MCP is installed

```bash
cat <<EOF | musicctl-mcp | jq
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"bash","version":"0.1"}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
EOF
```

The server runs on stdio transport, which is the standard for MCP integration.

### MCP Client Configuration

Add to your MCP client configuration

Claude Desktop:

```json
{
  "mcpServers": {
    "music-chore": {
      "command": "musicctl-mcp",
      "args": [],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

OpenCode:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "music-chore": {
      "type": "local",
      "command": ["musicctl-mcp"]
    }
  }
}
```

## Available Tools

### 1. `scan_directory`

Recursively scan a directory for music files and return file information.

**Parameters:**
- `path` (string, required): Base directory path to scan for music files
- `json_output` (boolean, optional): Return results as JSON (true) or simple file list (false). Default: false

**Returns:**
- If `json_output=false`: Simple newline-separated list of file paths
- If `json_output=true`: Full JSON array of track objects with complete metadata

**Example:**
```json
{
  "name": "scan_directory",
  "arguments": {
    "path": "/Users/music/FLAC",
    "json_output": false
  }
}
```

### 2. `get_library_tree`

Get a hierarchical tree view of the music library organized by artist and album.

**Parameters:**
- `path` (string, required): Base directory path to analyze
- `json_output` (boolean, optional): Return results as JSON (true) or structured library data (false). Default: false

**Returns:**
- If `json_output=false`: JSON library object (current implementation)
- If `json_output=true`: JSON library object with artist/album/track hierarchy
- Note: Currently always returns JSON structure, tree format planned for future release

**Example:**
```json
{
  "name": "get_library_tree",
  "arguments": {
    "path": "/Users/music/FLAC",
    "json_output": false
  }
}
```

### 3. `read_file_metadata`

Read comprehensive metadata from a single music file.

**Parameters:**
- `file_path` (string, required): Path to the music file

**Returns:**
Complete metadata object including title, artist, album, duration, format, and source information.

**Example:**
```json
{
  "name": "read_file_metadata",
  "arguments": {
    "file_path": "/Users/music/FLAC/Artist/Album/track.flac"
  }
}
```

### 4. `normalize_titles`

Normalize track titles to title case (proper capitalization).

**Parameters:**
- `path` (string, required): Directory path containing music files to normalize
- `dry_run` (boolean, optional): Preview changes without applying them. Default: true

**Returns:**
Object with processing statistics and detailed results for each track.

**Example:**
```json
{
  "name": "normalize_titles",
  "arguments": {
    "path": "/Users/music/FLAC",
    "dry_run": true
  }
}
```

### 5. `emit_library_metadata`

### 6. `validate_library`
Validate music library for common issues and inconsistencies.

**Parameters:**
- `path` (string, required): Base directory path to analyze
- `json_output` (boolean, optional): Return results as JSON (true) or AI-friendly structured text (false). Default: false

**Returns:**
- If `json_output=true`: JSON validation object with errors, warnings, and summary statistics
- If `json_output=false`: Human-readable validation report with emoji indicators and detailed issue descriptions

**Features:**
- Validates required metadata fields (title, artist, album)
- Checks for recommended metadata (year, track number, genre)
- Detects duplicate track numbers within albums
- Identifies unusually short/long tracks
- Reports empty albums, artists, or missing metadata
- Provides summary statistics and detailed error/warning reports

**Example:**
```json
{
  "name": "validate_library",
  "arguments": {
    "path": "/Users/music",
    "json_output": false
  }
}
```

**Response Format:**
If `json_output=false`:
```
=== MUSIC LIBRARY VALIDATION ===
📊 Summary:
  Total files: 145
  Valid files: 143
  Files with errors: 2
  Files with warnings: 8

🔴 ERRORS:
  File: /Users/music/Artist/Album/missing-title.flac
  Field: title
  Issue: Missing required field: title

  File: /Users/music/Artist/Album/corrupted-metadata.flac
  Field: title
  Issue: Title field is empty

🟡 WARNINGS:
  File: /Users/music/Artist/Album/missing-year.flac
  Field: year
  Issue: Missing recommended field: year

  File: /Users/music/Artist/Album/track-number-missing.flac
  Field: track_number
  Issue: Missing recommended field: track_number
```

If `json_output=true`:
```json
{
  "valid": false,
  "errors": [
    {
      "file_path": "/Users/music/Artist/Album/missing-title.flac",
      "field": "title",
      "message": "Missing required field: title"
    }
  ],
  "warnings": [
    {
      "file_path": "/Users/music/Artist/Album/missing-year.flac",
      "field": "year", 
      "message": "Missing recommended field: year"
    }
  ],
  "summary": {
    "total_files": 145,
    "valid_files": 143,
    "files_with_errors": 2,
    "files_with_warnings": 8
  }
}
```

### 5. `emit_library_metadata`

Emit complete library metadata in structured format optimized for AI analysis.

**Parameters:**
- `path` (string, required): Base directory path to analyze
- `json_output` (boolean, optional): Return results as JSON (true) or AI-friendly structured text (false). Default: false

**Returns:**
Complete library information with summary statistics and detailed track information in AI-optimized format or JSON.

**Example:**
```json
{
  "name": "emit_library_metadata",
  "arguments": {
    "path": "/Users/music/FLAC",
    "format": "text"
  }
}
```

### 7. `find_duplicates`

Find duplicate tracks by comparing SHA256 checksums of audio files.

**Parameters:**
- `path` (string, required): Base directory path to scan for duplicate tracks
- `json_output` (boolean, optional): Return results as JSON (true) or human-readable groups (false). Default: false

**Returns:**
- If `json_output=false`: Human-readable groups of duplicate files with their paths
- If `json_output=true`: JSON array where each element is an array of duplicate tracks with full metadata and checksums

**Example:**
```json
{
  "name": "find_duplicates",
  "arguments": {
    "path": "/Users/music/FLAC",
    "json_output": false
  }
}
```

### 8. `cue_file`

Unified tool for generating, parsing, and validating CUE files.

**Parameters:**
- `path` (string, required): Path to album directory, .cue file, or audio directory depending on operation
- `operation` (string, required): Operation type - "generate", "parse", or "validate"
- `output` (string, optional): Output path for CUE file (generate only)
- `dry_run` (boolean, optional): Preview without writing (generate only). Default: false
- `force` (boolean, optional): Overwrite existing file (generate only). Default: false
- `audio_dir` (string, optional): Directory containing audio files (validate only)
- `json_output` (boolean, optional): Return results as JSON (parse and validate only). Default: false

**Generate Example:**
```json
{
  "name": "cue_file",
  "arguments": {
    "path": "/Users/music/FLAC/Album",
    "operation": "generate",
    "dry_run": true,
    "force": false
  }
}
```

**Parse Example:**
```json
{
  "name": "cue_file",
  "arguments": {
    "path": "/Users/music/Album.cue",
    "operation": "parse",
    "json_output": true
  }
}
```

**Parse Response Example:**
```json
{
  "performer": "Kai Engel",
  "title": "Meanings",
  "genre": "Ambient",
  "date": "2024",
  "files": ["01. A New Journey Begins.flac", "02. Time Goes On.flac"],
  "tracks": [
    {
      "number": 1,
      "title": "A New Journey Begins",
      "performer": "Kai Engel",
      "index": "00:00:00",
      "file": "01. A New Journey Begins.flac"
    }
  ]
}
```

**Validate Example:**
```json
{
  "name": "cue_file",
  "arguments": {
    "path": "/Users/music/Album.cue",
    "operation": "validate",
    "audio_dir": "/Users/music",
    "json_output": true
  }
}
```

**Validate Response Example (valid):**
```
✓ CUE file is valid
  All referenced files exist and track count matches.
```

**Validate Response Example (invalid):**
```
✗ CUE file validation failed:
  - Referenced audio file(s) missing
```

**Validate Response Example (json_output=true):**
```json
{
  "is_valid": false,
  "parsing_error": false,
  "file_missing": true,
  "track_count_mismatch": false
}
```
```

## Response Formats

### Success Response
All tools return responses in this format:
```json
{
  "content": [
    {
      "type": "text",
      "text": "<result data>"
    }
  ],
  "isError": false
}
```

### Error Response
```json
{
  "content": [
    {
      "type": "text",
      "text": "Error: <error message>"
    }
  ],
  "isError": true
}
```

## Data Structures

### Track Object
```json
{
  "file_path": "/path/to/file.flac",
  "metadata": {
    "title": {"value": "Song Title", "source": "Embedded"},
    "artist": {"value": "Artist Name", "source": "Embedded"},
    "album": {"value": "Album Name", "source": "Embedded"},
    "duration": {"value": 245.5, "source": "Embedded"},
    "track_number": {"value": 1, "source": "Embedded"},
    "format": "flac",
    "sample_rate": 44100,
    "bit_depth": 16
  }
}
```

### Library Object
```json
{
  "total_artists": 5,
  "total_albums": 12,
  "total_tracks": 145,
  "artists": [
    {
      "name": "Artist Name",
      "albums": [
        {
          "title": "Album Title",
          "year": 2023,
          "tracks": [{}]
        }
      ]
    }
  ]
}
```

## Integration Examples

### Programmatic Usage

```javascript
// Using MCP client
const response = await client.callTool('scan_directory', {
  path: '/Users/music',
  json_output: true
});
```

## Error Handling

The MCP server provides detailed error messages for:
- Invalid file paths
- Unsupported file formats (currently FLAC only)
- Permission issues
- Corrupted metadata
- Network/disk I/O errors

All errors are returned in the standardized MCP error format with human-readable descriptions.

## Logging

Configure logging with the `RUST_LOG` environment variable:
- `RUST_LOG=error`: Only error messages
- `RUST_LOG=info`: General information (default)
- `RUST_LOG=debug`: Detailed debug information
- `RUST_LOG=trace`: Full trace information

## Performance Considerations

- Large directories (>10,000 files) may take several minutes to process
- Metadata reading is I/O bound - SSD storage recommended for best performance
- JSON responses are more compact but text responses are optimized for AI consumption
- Use `dry_run=true` for normalization operations to preview changes

## Limitations

**Current v1 limitations:**
- No internet-based metadata lookup

**Planned v2 enhancements:**
- Additional audio format support (DSF, OGG, M4A)
- Batch metadata operations
- Genre normalization
- Advanced CUE file validation

## Security

The MCP server operates with the same permissions as the user running it:
- Read access to specified music directories
- No network access (air-gapped operation)
- No external dependencies or services
- All operations are deterministic and reproducible

## Development

Easiest way to interact with local stdio mcp it's `rlwrap`. 

```bash
rlwrap musicctl-mcp

{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"bash","version":"0.1"}}}

# >> {"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"serverInfo":{"name":"music-chore","version":"0.1.2"},"instructions":"Music Chore CLI - Music library metadata management tool"}}

{"jsonrpc":"2.0","method":"notifications/initialized"}

# >> client initialized

{"jsonrpc":"2.0","id":2,"method":"tools/list"}

# >> full list of tools
```

## Support

For issues, feature requests, or questions:
1. Check the GitHub repository
2. Review existing issues
3. Create detailed bug reports with:
   - Operating system version
   - Rust version (`rustc --version`)
   - Exact command and parameters
   - Error messages and logs