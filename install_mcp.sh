#!/bin/bash

# Music Chore MCP Server Installation Script
# This script installs the music-chore MCP server for Claude Desktop

set -e

echo "🎵 Installing Music Chore MCP Server..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Please install Rust first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Please run this script from the music-chore repository root"
    exit 1
fi

# Build the project
echo "🔨 Building music-chore..."
cargo build --release

# Check if build succeeded
if [ ! -f "target/release/musicctl-mcp" ]; then
    echo "❌ Build failed. Please check the error messages above."
    exit 1
fi

# Install to system location
echo "📦 Installing to /usr/local/bin..."
sudo cp target/release/musicctl-mcp /usr/local/bin/
sudo chmod +x /usr/local/bin/musicctl-mcp

# Create Claude Desktop config directory
echo "📁 Setting up Claude Desktop configuration..."
CONFIG_DIR="$HOME/Library/Application Support/Claude"
mkdir -p "$CONFIG_DIR"

# Backup existing config if it exists
CONFIG_FILE="$CONFIG_DIR/claude_desktop_config.json"
if [ -f "$CONFIG_FILE" ]; then
    cp "$CONFIG_FILE" "$CONFIG_FILE.backup.$(date +%Y%m%d_%H%M%S)"
    echo "💾 Backed up existing config to: $CONFIG_FILE.backup.$(date +%Y%m%d_%H%M%S)"
fi

# Create or update the config
cat > "$CONFIG_FILE" << EOF
{
  "mcpServers": {
    "music-chore": {
      "command": "/usr/local/bin/musicctl-mcp",
      "args": ["--verbose"],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
EOF

echo "✅ Installation complete!"
echo ""
echo "🎯 Next steps:"
echo "1. Restart Claude Desktop"
echo "2. Test with: echo '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2024-11-05\",\"capabilities\":{},\"clientInfo\":{\"name\":\"test\",\"version\":\"1.0.0\"}}}' | musicctl-mcp"
echo ""
echo "📖 For more information, see: https://github.com/Komdosh/music-chore/blob/main/docs/MCP_SERVER.md"

# Get the script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "📁 Project directory: $PROJECT_ROOT"

# Build the project
echo "🔨 Building Music Chore..."
cd "$PROJECT_ROOT"
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Build failed"
    exit 1
fi

# Get the binary path
BINARY_PATH="$PROJECT_ROOT/target/release/musicctl-mcp"
echo "📦 Binary built at: $BINARY_PATH"

# Create symbolic link in user bin directory
USER_BIN="$HOME/.local/bin"
mkdir -p "$USER_BIN"
ln -sf "$BINARY_PATH" "$USER_BIN/musicctl-mcp"

# Add to PATH if not already there
if [[ ":$PATH:" != *":$USER_BIN:"* ]]; then
    echo "🔧 Adding $USER_BIN to PATH..."
    echo 'export PATH="$PATH:$HOME/.local/bin"' >> "$HOME/.bashrc"
    echo "export PATH=\"$PATH:\$HOME/.local/bin\"" >> "$HOME/.zshrc"
    echo "⚠️  Please restart your shell or run: export PATH=\"\$PATH:\$HOME/.local/bin\""
fi

# Test the binary
echo "🧪 Testing installation..."
if [ -x "$BINARY_PATH" ]; then
    echo "✅ Installation successful!"
    echo ""
    echo "🚀 Quick test:"
    echo "   echo '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2024-11-05\",\"capabilities\":{},\"clientInfo\":{\"name\":\"test\",\"version\":\"1.0.0\"}}}' | $USER_BIN/musicctl-mcp"
    echo ""
    echo "📖 For documentation visit: $PROJECT_ROOT/docs/MCP_SERVER.md"
    echo "🔧 For configuration examples visit: $PROJECT_ROOT/docs/MCP_CONFIG_EXAMPLES.md"
else
    echo "❌ Binary not found at expected location"
    exit 1
fi