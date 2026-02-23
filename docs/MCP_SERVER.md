# MCP Server Guide

`musicctl-mcp` exposes `music-chore` functionality to AI agents via Model Context Protocol (MCP).

## Scope

This server provides:
- 8 MCP tools for scanning, metadata operations, validation, duplicates, and CUE workflows
- 6 high-value MCP prompts for listening decisions and maintenance
- path-based security controls through environment variables

## Install and Register

### Claude CLI
```bash
claude mcp add -e MUSIC_LIBRARY_PATH="/path/to/music" music-chore -- musicctl-mcp
```

### Gemini CLI
```bash
gemini mcp add -e MUSIC_LIBRARY_PATH="/path/to/music" music-chore musicctl-mcp
```

### Qwen
```bash
qwen mcp add music-chore musicctl-mcp -e MUSIC_LIBRARY_PATH="/path/to/music"
```

## Available Tools (8)

1. `scan_directory`
2. `get_library_tree`
3. `read_file_metadata`
4. `normalize`
5. `emit_library_metadata`
6. `validate_library`
7. `find_duplicates`
8. `cue_file`

## Available Prompts (6)

- `listen-now`
- `web-perfect-match`
- `library-health-check`
- `metadata-cleanup-guide`
- `duplicate-resolution`
- `cue-sheet-assistant`

## Environment Variables

- `RUST_LOG`: logging level (`error|warn|info|debug|trace`)
- `MUSIC_LIBRARY_PATH`: default path when tool request omits `path`
- `MUSIC_SCAN_TIMEOUT`: scan timeout in seconds (default `300`)
- `MUSIC_ALLOWED_PATHS`: comma-separated allowed roots

Example:
```bash
export RUST_LOG=info
export MUSIC_LIBRARY_PATH=/Users/username/Music
export MUSIC_ALLOWED_PATHS=/Users/username/Music,/Volumes/Music
musicctl-mcp
```

## Security Model

If `MUSIC_ALLOWED_PATHS` is set, the server rejects access outside these paths.
Use this in all shared or agent-driven environments.

## Minimal Config Snippets

### Claude Desktop-style JSON
```json
{
  "mcpServers": {
    "music-chore": {
      "command": "musicctl-mcp",
      "env": {
        "RUST_LOG": "info",
        "MUSIC_LIBRARY_PATH": "/Users/username/Music",
        "MUSIC_ALLOWED_PATHS": "/Users/username/Music"
      }
    }
  }
}
```

### Node example (tool call)
```javascript
const result = await client.callTool('normalize', {
  path: '/Users/username/Music',
  json_output: true,
});
```

## Manual Smoke Test

```bash
rlwrap musicctl-mcp
```

Initialize request:
```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test"}}}
```
