# Music Chore MCP Server

The Model Context Protocol (MCP) server for Music Chore provides AI agents with programmatic access to music library management capabilities. It exposes the core functionality of the `musicctl` CLI tool through a standardized MCP interface.

## ✅ Status: Production Ready

The MCP server is **fully functional and tested** with:
- ✅ Complete MCP protocol implementation
- ✅ All 5 core tools exposed and working
- ✅ Proper initialization and shutdown handling  
- ✅ Comprehensive error handling
- ✅ AI-friendly structured output
- ✅ Graceful EOF handling (no more error spam)

## Overview

The MCP server allows AI agents to:
- Scan directories for music files
- Read and analyze metadata from individual files
- Get hierarchical library tree views
- Normalize track titles automatically
- Emit structured library metadata for analysis

## Installation

### Quick Install (Recommended)

```bash
# Automated setup with Claude CLI (easiest)
curl -fsSL https://raw.githubusercontent.com/Komdosh/music-chore/main/install_mcp.sh | bash

# Or use Claude CLI directly
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

## Usage

### Starting the MCP Server

```bash
# Basic usage
musicctl-mcp

# With verbose logging
musicctl-mcp --verbose
```

The server runs on stdio transport, which is the standard for MCP integration.

### MCP Client Configuration

Add to your MCP client configuration (e.g., Claude Desktop):

```json
{
  "mcpServers": {
    "music-chore": {
      "command": "/path/to/musicctl-mcp",
      "args": [],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

## Available Tools

### 1. `scan_directory`

Recursively scan a directory for music files and return file information.

**Parameters:**
- `path` (string, required): Base directory path to scan for music files
- `json_output` (boolean, optional): Return results as JSON (true) or simple list (false). Default: false

**Returns:**
- If `json_output=false`: `{"files": ["path1", "path2", ...], "count": N}`
- If `json_output=true`: Full JSON array of track objects

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
- `json_output` (boolean, optional): Return results as JSON (true) or formatted tree (false). Default: false

**Returns:**
- If `json_output=false`: Formatted ASCII tree with emojis
- If `json_output=true`: Full JSON library object with artist/album/track hierarchy

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

Emit complete library metadata in structured format optimized for AI analysis.

**Parameters:**
- `path` (string, required): Base directory path to analyze
- `format` (string, optional): Output format - "text" for AI-friendly structured text, "json" for JSON. Default: "text"

**Returns:**
Complete library information with summary statistics and detailed track information.

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
          "tracks": [...]
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
- Supports only FLAC format
- No write operations (metadata editing not yet implemented via MCP)
- No internet-based metadata lookup
- macOS only (matches CLI tool constraints)

**Planned v2 enhancements:**
- MP3, WAV, DSF format support
- Metadata write capabilities
- Cue sheet integration
- Playlist management

## Security

The MCP server operates with the same permissions as the user running it:
- Read access to specified music directories
- No network access (air-gapped operation)
- No external dependencies or services
- All operations are deterministic and reproducible

## Development

For development and testing:
```bash
# Run with debug logging
RUST_LOG=debug musicctl-mcp --verbose

# Test specific directory
echo '{"name":"scan_directory","arguments":{"path":"/test/music"}}' | musicctl-mcp
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