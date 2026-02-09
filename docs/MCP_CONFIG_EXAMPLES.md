# MCP Server Configuration Examples

This document provides configuration examples for integrating the Music Chore MCP server with various AI clients and platforms.

## 📅 Last Updated

- **Date**: February 9, 2026
- **Version**: v0.3.2
- **Features**: 8 MCP tools, 18 Expert AI Prompts, Path-based Security

## Claude Desktop

### 🚀 Automated Setup (Recommended)

```bash
claude mcp add -e MUSIC_LIBRARY_PATH="/path/to/music" music-chore -- musicctl-mcp
```

### Basic Configuration

```json
{
  "mcpServers": {
    "music-chore": {
      "command": "musicctl-mcp",
      "args": [],
      "env": {
        "RUST_LOG": "info",
        "MUSIC_LIBRARY_PATH": "/Users/username/Music"
      }
    }
  }
}
```

### Security-Enhanced Configuration

```json
{
  "mcpServers": {
    "music-chore": {
      "command": "musicctl-mcp",
      "args": ["--verbose"],
      "env": {
        "RUST_LOG": "debug",
        "MUSIC_LIBRARY_PATH": "/Users/username/Music",
        "MUSIC_SCAN_TIMEOUT": "600",
        "MUSIC_ALLOWED_PATHS": "/Users/username/Music,/Volumes/Music,/Backup/Music"
      }
    }
  }
}
```

## Gemini CLI

### 🚀 Automated Setup

```bash
gemini mcp add -e MUSIC_LIBRARY_PATH="/path/to/music" music-chore musicctl-mcp
```

## Qwen

### 🚀 Automated Setup

```bash
qwen mcp add music-chore musicctl-mcp -e MUSIC_LIBRARY_PATH="/path/to/music"
```

## IDE Integration (Windsurf / Cursor / Cline)

### Windsurf Configuration

```json
{
  "mcpServers": {
    "music-chore": {
      "command": "musicctl-mcp",
      "env": {
        "MUSIC_LIBRARY_PATH": "/Users/username/Music",
        "MUSIC_ALLOWED_PATHS": "/Users/username/Music"
      }
    }
  }
}
```

### Cursor Configuration

1. Go to **Settings** > **Features** > **MCP**
2. Click **+ Add New MCP Server**
3. Name: `music-chore`
4. Type: `command`
5. Command:
   ```bash
   MUSIC_LIBRARY_PATH="/Users/username/Music" musicctl-mcp
   ```

## Custom Node.js Integration

### MCP Client Setup

```javascript
import { createMcpClient } from 'mcp-client';

const client = await createMcpClient({
  name: 'music-chore',
  command: 'musicctl-mcp',
  args: ['--verbose'],
  env: {
    'RUST_LOG': 'info',
    'MUSIC_LIBRARY_PATH': '/Users/username/Music'
  }
});

// Use the new unified normalize tool
const normResult = await client.callTool('normalize', {
  path: '/Users/music/FLAC',
  json_output: true
});

// Use CUE sheet assistant prompt
const promptResult = await client.getPrompt('cue-sheet-assistant', {
  path: '/Users/music/FLAC/Album'
});
```

## Python Integration

### MCP Python Client

```python
import asyncio
import json
from mcp_client import AsyncMCPClient

class MusicChoreClient:
    def __init__(self, command="musicctl-mcp", env=None):
        self.client = AsyncMCPClient(command, env=env or {
            'RUST_LOG': 'info',
            'MUSIC_LIBRARY_PATH': '/Users/username/Music'
        })
    
    async def get_library_health(self, path):
        """Use the expert prompt for a full health check"""
        result = await self.client.get_prompt('library-health-check', {
            'path': path
        })
        return result.messages[0].content.text

    async def normalize(self, path, json_output=True):
        """Standardize titles and genres in one pass"""
        result = await self.client.call_tool('normalize', {
            'path': path,
            'json_output': json_output
        })
        return json.loads(result.content[0].text)

# Usage example
async def main():
    client = MusicChoreClient()
    health_report = await client.get_library_health('/Users/username/Music')
    print(health_report)

if __name__ == "__main__":
    asyncio.run(main())
```

## Security Considerations

1. **Path Restrictions**: Always use `MUSIC_ALLOWED_PATHS` to prevent the AI from accessing files outside your music library.
2. **Read-Only by Default**: MCP tools never modify files directly unless using the CLI with `--apply`. Normalization and CUE generation are safe previews.
3. **Environment Isolation**: Run the server with its own environment variables to keep your system clean.
