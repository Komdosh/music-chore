# MCP Server Configuration Examples

This document provides configuration examples for integrating the Music Chore MCP server with various AI clients and platforms.

## Claude Desktop

### Configuration File Location

- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

### Basic Configuration

```json
{
  "mcpServers": {
    "music-chore": {
      "command": "/usr/local/bin/musicctl-mcp",
      "args": [],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

### Advanced Configuration

```json
{
  "mcpServers": {
    "music-chore": {
      "command": "/path/to/musicctl-mcp",
      "args": ["--verbose"],
      "env": {
        "RUST_LOG": "debug",
        "MUSIC_LIBRARY_PATH": "/Users/username/Music"
      }
    }
  }
}
```

### Multiple Music Libraries

```json
{
  "mcpServers": {
    "music-chore-flac": {
      "command": "/usr/local/bin/musicctl-mcp",
      "args": ["--verbose"],
      "env": {
        "RUST_LOG": "info",
        "MUSIC_LIBRARY_PATH": "/Users/username/Music/FLAC"
      }
    },
    "music-chore-mp3": {
      "command": "/usr/local/bin/musicctl-mcp", 
      "args": ["--verbose"],
      "env": {
        "RUST_LOG": "info",
        "MUSIC_LIBRARY_PATH": "/Users/username/Music/MP3"
      }
    }
  }
}
```

## Continue.dev

### MCP Client Integration

```javascript
import { MCPClient } from "@continue/dev/mcp";

const musicChoreClient = new MCPClient({
  name: "music-chore",
  command: "/usr/local/bin/musicctl-mcp",
  args: ["--verbose"],
  env: {
    RUST_LOG: "info"
  }
});

// Use in custom commands
export async function analyzeMusicLibrary(path) {
  const result = await musicChoreClient.callTool('emit_library_metadata', {
    path: path,
    format: 'json'
  });
  return result.content[0].text;
}
```

## Custom Node.js Integration

### MCP Client Setup

```javascript
import { createMcpClient } from 'mcp-client';

const client = await createMcpClient({
  name: 'music-chore',
  command: '/usr/local/bin/musicctl-mcp',
  args: ['--verbose']
});

// Scan directory
const scanResult = await client.callTool('scan_directory', {
  path: '/Users/music/FLAC',
  json_output: true
});

// Get library tree
const treeResult = await client.callTool('get_library_tree', {
  path: '/Users/music/FLAC',
  json_output: false
});

// Read file metadata
const metadataResult = await client.callTool('read_file_metadata', {
  file_path: '/Users/music/FLAC/Artist/Album/track.flac'
});
```

### React Component Integration

```jsx
import React, { useState, useEffect } from 'react';
import { MCPClient } from 'mcp-react';

const MusicLibraryViewer = ({ libraryPath }) => {
  const [library, setLibrary] = useState(null);
  const [loading, setLoading] = useState(true);
  
  const musicClient = new MCPClient({
    command: '/usr/local/bin/musicctl-mcp',
    args: []
  });

  useEffect(() => {
    const loadLibrary = async () => {
      try {
        const result = await musicClient.callTool('get_library_tree', {
          path: libraryPath,
          json_output: true
        });
        setLibrary(JSON.parse(result.content[0].text));
      } catch (error) {
        console.error('Failed to load library:', error);
      } finally {
        setLoading(false);
      }
    };

    loadLibrary();
  }, [libraryPath]);

  if (loading) return <div>Loading music library...</div>;
  
  return (
    <div>
      <h2>Music Library</h2>
      <p>Artists: {library?.total_artists}</p>
      <p>Albums: {library?.total_albums}</p>
      <p>Tracks: {library?.total_tracks}</p>
      
      {library?.artists?.map(artist => (
        <div key={artist.name}>
          <h3>{artist.name}</h3>
          {artist.albums?.map(album => (
            <div key={album.title}>
              <h4>{album.title} ({album.year})</h4>
              <ul>
                {album.tracks?.map((track, index) => (
                  <li key={index}>
                    {track.metadata.title?.value} - {track.metadata.duration?.value}s
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>
      ))}
    </div>
  );
};
```

## Python Integration

### MCP Python Client

```python
import asyncio
import json
from mcp_client import AsyncMCPClient

class MusicChoreClient:
    def __init__(self, command="/usr/local/bin/musicctl-mcp"):
        self.client = AsyncMCPClient(command)
    
    async def scan_directory(self, path, json_output=True):
        """Scan a directory for music files"""
        result = await self.client.call_tool('scan_directory', {
            'path': path,
            'json_output': json_output
        })
        return json.loads(result.content[0].text)
    
    async def get_library_tree(self, path, json_output=True):
        """Get hierarchical library view"""
        result = await self.client.call_tool('get_library_tree', {
            'path': path,
            'json_output': json_output
        })
        return json.loads(result.content[0].text)
    
    async def read_file_metadata(self, file_path):
        """Read metadata from a specific file"""
        result = await self.client.call_tool('read_file_metadata', {
            'file_path': file_path
        })
        return json.loads(result.content[0].text)
    
    async def normalize_titles(self, path, dry_run=True):
        """Normalize track titles"""
        result = await self.client.call_tool('normalize_titles', {
            'path': path,
            'dry_run': dry_run
        })
        return json.loads(result.content[0].text)

# Usage example
async def main():
    client = MusicChoreClient()
    
    # Scan library
    library = await client.get_library_tree('/Users/music/FLAC')
    print(f"Found {library['total_artists']} artists")
    
    # Read specific file
    metadata = await client.read_file_metadata('/Users/music/FLAC/Artist/Album/track.flac')
    print(f"Track: {metadata['metadata']['title']['value']}")
    print(f"Artist: {metadata['metadata']['artist']['value']}")

if __name__ == "__main__":
    asyncio.run(main())
```

## Docker Integration

### Dockerfile with MCP Server

```dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/musicctl-mcp /usr/local/bin/
COPY --from=builder /app/target/release/musicctl /usr/local/bin/

ENV RUST_LOG=info
ENTRYPOINT ["musicctl-mcp"]
CMD ["--verbose"]
```

### Docker Compose with MCP

```yaml
version: '3.8'
services:
  music-chore-mcp:
    build: .
    container_name: music-chore-mcp
    volumes:
      - /Users/username/Music:/music:ro
    environment:
      - RUST_LOG=debug
    stdin_open: true
    tty: true
    
  ai-assistant:
    image: claude-assistant:latest
    depends_on:
      - music-chore-mcp
    environment:
      - MCP_SERVERS_CONFIG=/config/mcp.json
    volumes:
      - ./mcp-config.json:/config/mcp.json:ro
    command: ["--mcp-server", "music-chore-mcp"]
```

## Environment Variables

### Supported Environment Variables

```bash
# Logging level
export RUST_LOG=info  # error, warn, info, debug, trace

# Default music library path (optional)
export MUSIC_LIBRARY_PATH=/Users/username/Music

# Performance tuning
export MUSIC_SCAN_TIMEOUT=300  # seconds

# Security
export MUSIC_ALLOWED_PATHS=/Users/username/Music,/Volumes/Music
```

### Example Environment Setup

```bash
# Create environment file
cat > .env << EOF
RUST_LOG=debug
MUSIC_LIBRARY_PATH=/Users/username/Music/FLAC
MUSIC_SCAN_TIMEOUT=600
EOF

# Load environment
source .env

# Start MCP server
musicctl-mcp --verbose
```

## Troubleshooting

### Common Issues

1. **Permission Denied**
   ```bash
   chmod +x /path/to/musicctl-mcp
   ```

2. **Library Not Found**
   ```bash
   export DYLD_LIBRARY_PATH=/usr/local/lib:$DYLD_LIBRARY_PATH
   ```

3. **Logging Issues**
   ```bash
   # Enable debug logging
   RUST_LOG=debug musicctl-mcp --verbose
   
   # Check server status
   echo '{"name":"scan_directory","arguments":{"path":"/test"}}' | musicctl-mcp
   ```

### Health Check Script

```bash
#!/bin/bash
# health-check.sh

echo "Testing MCP server health..."

# Test basic connectivity
echo '{"jsonrpc": "2.0", "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0.0"}}, "id": 1}' | musicctl-mcp | jq -e '.result' > /dev/null

if [ $? -eq 0 ]; then
    echo "✅ MCP server is responding"
else
    echo "❌ MCP server failed to respond"
    exit 1
fi

# Test tool availability
echo '{"jsonrpc": "2.0", "method": "tools/list", "id": 2}' | musicctl-mcp | jq -e '.result.tools[] | select(.name == "scan_directory")' > /dev/null

if [ $? -eq 0 ]; then
    echo "✅ Tools are available"
else
    echo "❌ Tools not found"
    exit 1
fi

echo "✅ All health checks passed"
```

### Testing MCP Connection

```bash
# Simple test script
#!/bin/bash
echo "Testing scan_directory tool..."

cat << EOF | musicctl-mcp
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "scan_directory",
    "arguments": {
      "path": "/tmp",
      "json_output": false
    }
  },
  "id": 3
}
EOF
```

## Security Considerations

### File System Access

The MCP server inherits the permissions of the user running it. Consider:

1. **Principle of least privilege**
   ```bash
   # Create dedicated user
   sudo useradd -r -s /bin/false musicchore
   sudo -u musicchore musicctl-mcp
   ```

2. **Path restrictions**
   ```bash
   export MUSIC_ALLOWED_PATHS=/safe/music/directory
   ```

3. **Container isolation**
   ```dockerfile
   USER 1000:1000
   VOLUME ["/safe/music:/music:ro"]
   ```

### Network Security

- MCP server uses stdio transport (no network exposure)
- No external API calls or internet access
- All processing is local and offline

### Data Privacy

- No data is transmitted to external services
- All metadata processing happens locally
- No telemetry or analytics collection