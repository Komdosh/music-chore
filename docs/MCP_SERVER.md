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
- Normalize track titles and genres automatically
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
claude mcp add -e MUSIC_LIBRARY_PATH="/path/to/music" music-chore -- musicctl-mcp
```

Gemini CLI:

```bash
gemini mcp add -e MUSIC_LIBRARY_PATH="/path/to/music" music-chore musicctl-mcp
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
- `path` (string, required): Path to the music file

**Returns:**
Complete metadata object including title, artist, album, duration, format, and source information.

**Example:**
```json
{
  "name": "read_file_metadata",
  "arguments": {
    "path": "/Users/music/FLAC/Artist/Album/track.flac"
  }
}
```

### 4. `normalize`

Normalize track titles and genres. This tool reports proposed changes to track titles (to proper title case) and genres (to standardized forms) without modifying any files.

**Parameters:**
- `path` (string, required): Directory path containing music files to normalize
- `json_output` (boolean, optional): Return results as JSON (true) or human-readable summary (false). Default: false

**Returns:**
If `json_output=false`: Human-readable summary of proposed title and genre normalization changes.
If `json_output=true`: A JSON object containing `title_reports` (array of `TitleNormalizationReport` objects), `genre_reports` (array of `GenreNormalizationReport` objects), and a `summary` string.

**Example (Human-readable):**
```json
{
  "name": "normalize",
  "arguments": {
    "path": "/Users/music/FLAC",
    "json_output": false
  }
}
```

**Example (JSON Output):**
```json
{
  "name": "normalize",
  "arguments": {
    "path": "/Users/music/FLAC",
    "json_output": true
  }
}
```

**JSON Response Example:**
```json
{
  "title_reports": [
    {
      "original_path": "/path/to/music/01 - come together.flac",
      "original_title": "come together",
      "normalized_title": "Come Together",
      "changed": true,
      "error": null
    }
  ],
  "genre_reports": [
    {
      "original_path": "/path/to/music/01 - come together.flac",
      "original_genre": "rock and roll",
      "normalized_genre": "Rock",
      "changed": true,
      "error": null
    }
  ],
  "summary": "Combined normalization report"
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
    "json_output": false
  }
}
```

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

## Available Prompts

Prompts are predefined complex workflows that guide AI agents through multi-step analysis or maintenance tasks. They provide the agent with a strategy and the necessary tool-calling sequence to achieve a specific goal.

### 1. Analysis & Insights
- `top-tracks-analysis`: Predicts favorite tracks based on library patterns and metadata richness.
- `genre-breakdown`: Analyzes genre distribution and discovers the user's "listening identity".
- `decade-analysis`: Temporal analysis of the collection across decades.
- `collection-story`: Generates a narrative about the library's themes, diversity, and emotional arc.
- `artist-deep-dive`: Deep dive into a specific artist's discography coverage and standout tracks.

### 2. Recommendations & Discovery
- `instrument-to-learn`: Recommends an instrument to learn based on library taste.
- `similar-artists-discovery`: Suggests new artists based on library DNA.
- `mood-playlist`: Creates a curated playlist for a specific mood or activity.
- `hidden-gems`: Uncovers overlooked and underappreciated tracks.
- `album-marathon`: Designs a themed album listening marathon.
- `concert-setlist`: Builds a dream concert setlist from the library.

### 3. Library Maintenance & Quality
- `library-health-check`: Comprehensive assessment of metadata, structure, and duplicates.
- `metadata-cleanup-guide`: Step-by-step guide to fix metadata issues using normalization tools.
- `duplicate-resolution`: Intelligent recommendations for resolving duplicate tracks.
- `reorganization-plan`: Strategic plan to restructure folders into `Artist/Album/Track` hierarchy.
- `format-quality-audit`: Audit of audio formats and quality tiers (Lossless vs Lossy).
- `year-in-review`: Annual summary of library additions and milestones.
- `cue-sheet-assistant`: Analyze, generate, or troubleshoot CUE sheets.

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

### Combined Normalization Report Object
```json
{
  "title_reports": [
    {
      "original_path": "/path/to/music/01 - come together.flac",
      "original_title": "come together",
      "normalized_title": "Come Together",
      "changed": true,
      "error": null
    }
  ],
  "genre_reports": [
    {
      "original_path": "/path/to/music/01 - come together.flac",
      "original_genre": "rock and roll",
      "normalized_genre": "Rock",
      "changed": true,
      "error": null
    }
  ],
  "summary": "Combined normalization report"
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
- Unsupported file formats (currently supports FLAC, MP3, WAV, DSF, WavPack)
- Permission issues
- Corrupted metadata
- Network/disk I/O errors

All errors are returned in the standardized MCP error format with human-readable descriptions.

## Environment Variables

The musicctl-mcp server supports several environment variables for configuration:

### Logging Configuration

**RUST_LOG**: Set the logging level
- `RUST_LOG=error`: Only error messages
- `RUST_LOG=warn`: Warnings and errors only
- `RUST_LOG=info`: General information (default)
- `RUST_LOG=debug`: Detailed debug information
- `RUST_LOG=trace`: Full trace information

### Library Configuration

**MUSIC_LIBRARY_PATH**: Default music library path
- Sets a default path that can be used when no path is explicitly provided
- Example: `export MUSIC_LIBRARY_PATH=/Users/username/Music`

**MUSIC_SCAN_TIMEOUT**: Scan timeout in seconds
- Controls maximum time for directory scanning operations
- Default: 300 seconds (5 minutes)
- Example: `export MUSIC_SCAN_TIMEOUT=600`

### Security Configuration

**MUSIC_ALLOWED_PATHS**: Comma-separated list of allowed paths
- Restricts MCP server access to specific directories for security
- If not set, all paths are allowed (backwards compatibility)
- Example: `export MUSIC_ALLOWED_PATHS=/Users/username/Music,/Volumes/Music`

## Configuration Examples

### Basic Setup
```bash
# Set logging level
export RUST_LOG=info

# Set default library path
export MUSIC_LIBRARY_PATH=/Users/username/Music

# Run the MCP server
musicctl-mcp
```

### Security-Restricted Setup
```bash
# Enable debug logging
export RUST_LOG=debug

# Restrict access to specific directories
export MUSIC_ALLOWED_PATHS=/Users/username/Music,/Volumes/Music,/Backup/Music

# Set longer timeout for large libraries
export MUSIC_SCAN_TIMEOUT=600

# Run with security restrictions
musicctl-mcp
```

### MCP Client Configuration with Environment Variables

Claude Desktop:
```json
{
  "mcpServers": {
    "music-chore": {
      "command": "musicctl-mcp",
      "args": [],
      "env": {
        "RUST_LOG": "info",
        "MUSIC_LIBRARY_PATH": "/Users/username/Music",
        "MUSIC_ALLOWED_PATHS": "/Users/username/Music,/Volumes/Music"
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
      "command": ["musicctl-mcp"],
      "env": {
        "RUST_LOG": "debug",
        "MUSIC_ALLOWED_PATHS": "/Users/music"
      }
    }
  }
}
```

## Performance Considerations

- Large directories (>10,000 files) may take several minutes to process
- Metadata reading is I/O bound - SSD storage recommended for best performance
- JSON responses are more compact but text responses are optimized for AI consumption
- The `normalize` tool now processes both titles and genres in a single pass.

## Limitations

**Current v1 limitations:**
- No internet-based metadata lookup

**Planned v2 enhancements:**
- Additional audio format support (OGG, M4A)
- Batch metadata operations
- Advanced CUE file validation

## Security

The MCP server operates with the same permissions as the user running it:
- Read access to specified music directories
- No network access (air-gapped operation)
- No external dependencies or services
- All operations are deterministic and reproducible

### Path Security

When `MUSIC_ALLOWED_PATHS` is configured, the server validates all file operations:
- All tool parameters that reference paths are validated against allowed paths
- Attempts to access paths outside allowed directories are blocked
- Returns descriptive error messages for blocked access

**Security Best Practices:**
1. Always set `MUSIC_ALLOWED_PATHS` in production environments
2. Use specific paths rather than broad parent directories
3. Consider using absolute paths to avoid ambiguity
4. Test path restrictions before deploying in sensitive environments

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
