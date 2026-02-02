# 🎵 musicctl - Complete User Guide

`musicctl` is the command-line interface for the music-chore library management system. This comprehensive guide covers all commands, options, and usage patterns.

## 📅 Last Updated

- **Date**: February 2, 2026
- **Version**: v0.1.7
- **Features**: CLI with 6 commands + MCP server with 6 tools

## 📋 Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Command Reference](#command-reference)
- [Advanced Usage](#advanced-usage)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)

---

## ⚡ Quick Start

### 1. Scan Your Music Library

```bash
# Basic scan
musicctl scan ~/Music

# Scan with verbose output
musicctl scan ~/Music --verbose

# Scan specific subdirectory
musicctl scan ~/Music/Rock
```

### 2. View Library Structure

```bash
# Tree view with emojis
musicctl tree ~/Music

# JSON output for scripts
musicctl tree ~/Music --json
```

### 3. Read File Metadata

```bash
# Detailed metadata
musicctl read ~/Music/The\ Beatles/Abbey\ Road/01\ -\ Come\ Together.flac

# Compact metadata
musicctl read ~/Music/The\ Beatles/Abbey\ Road/01\ -\ Come\ Together.flac --compact
```

### 4. Normalize Track Titles

```bash
# Preview changes
musicctl normalize ~/Music --dry-run

# Apply changes
musicctl normalize ~/Music
```

### 5. Export Library Data

```bash
# AI-friendly format
musicctl emit ~/Music

# JSON for programming
musicctl emit ~/Music --json
```

---

## 📖 Command Reference

### `musicctl` - Global Options

```bash
musicctl [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS]

Global Options:
  -v, --verbose     Enable verbose output
  -h, --help        Print help information
  -V, --version     Print version information
```

### 🔍 `scan` - Discover Music Files

Recursively scan directories for supported audio files.

```bash
musicctl scan [OPTIONS] <PATH>

Options:
  -v, --verbose     Show detailed scanning information
  -j, --json        Output results as JSON

Arguments:
  <PATH>            Directory to scan (required)
```

**Output Examples:**

```
# Default output
/music/The Beatles/Abbey Road/01 - Come Together.flac
/music/The Beatles/Abbey Road/02 - Something.flac
/music/Pink Floyd/The Dark Side of the Moon/01 - Speak to Me.flac

# JSON output
{
  "files": [
    "/music/The Beatles/Abbey Road/01 - Come Together.flac",
    "/music/The Beatles/Abbey Road/02 - Something.flac"
  ],
  "total_count": 2
}
```

**Use Cases:**
- Discover what files exist in a directory
- Create file lists for batch processing
- Verify directory contents
- Generate input for other tools

### 🌳 `tree` - Display Library Hierarchy

Show a beautiful tree view of your music library structure.

```bash
musicctl tree [OPTIONS] <PATH>

Options:
  -j, --json        Output results as JSON
  -v, --verbose     Show additional metadata

Arguments:
  <PATH>            Directory to scan (required)
```

**Output Examples:**

```
📁 Music Folder
├── 📂 The Beatles
│   ├── 📂 Abbey Road (1969)
│   │   ├── 🎵 01 - Come Together.flac [🤖] FLAC
│   │   ├── 🎵 02 - Something.flac [🤖] FLAC
│   │   └── 🎵 03 - Here Comes the Sun.flac [🤖] FLAC
│   └── 📂 Let It Be (1970)
│       └── 🎵 01 - Two of Us.flac [🤖] FLAC
└── 📂 Pink Floyd
    └── 📂 The Dark Side of the Moon (1973)
        ├── 🎵 01 - Speak to Me.flac [🤖] FLAC
        └── 🎵 02 - On the Run.flac [🤖] FLAC
```

**JSON Output:**
```json
{
  "name": "Music Folder",
  "path": "/music",
  "type": "directory",
  "children": [
    {
      "name": "The Beatles",
      "type": "artist",
      "albums": [
        {
          "name": "Abbey Road",
          "year": 1969,
          "tracks": [
            {
              "name": "01 - Come Together.flac",
              "format": "FLAC",
              "has_metadata": true
            }
          ]
        }
      ]
    }
  ]
}
```

**Use Cases:**
- Visualize library organization
- Identify folder structure issues
- Get overview of collection
- Plan reorganization

### 📖 `read` - Extract File Metadata

Read and display detailed metadata from audio files.

```bash
musicctl read [OPTIONS] <FILE_PATH>

Options:
  -c, --compact     Show compact metadata view
  -j, --json        Output results as JSON

Arguments:
  <FILE_PATH>       Audio file to read (required)
```

**Output Examples:**

```
# Default detailed output
📁 File: /music/The Beatles/Abbey Road/01 - Come Together.flac
🎵 Title: Come Together
👤 Artist: The Beatles
💿 Album: Abbey Road
🎤 Album Artist: The Beatles
🔢 Track: 9/17
💿 Disc: 1/1
📅 Year: 1969
🎭 Genre: Rock
⏱️  Duration: 4:19 (259 seconds)
📁 Format: FLAC
```

**Compact Output:**
```
Come Together | The Beatles | Abbey Road | 1969 | Rock | 4:19
```

**JSON Output:**
```json
{
  "file_path": "/music/The Beatles/Abbey Road/01 - Come Together.flac",
  "metadata": {
    "title": {
      "value": "Come Together",
      "source": "Embedded",
      "confidence": 1.0
    },
    "artist": {
      "value": "The Beatles", 
      "source": "Embedded",
      "confidence": 1.0
    },
    "album": {
      "value": "Abbey Road",
      "source": "Embedded", 
      "confidence": 1.0
    },
    "duration": {
      "value": 259.0,
      "source": "Embedded",
      "confidence": 1.0
    },
    "format": "flac"
  }
}
```

**Use Cases:**
- Verify metadata quality
- Extract specific information
- Debug metadata issues
- Prepare data for analysis

### 🔤 `normalize` - Normalize Track Titles

Convert track titles to proper title case formatting.

```bash
musicctl normalize [OPTIONS] <PATH>

Options:
  -d, --dry-run     Preview changes without applying them
  -v, --verbose     Show detailed changes
  -j, --json        Output results as JSON

Arguments:
  <PATH>            Directory to process (required)
```

**Output Examples:**

```
# Dry run preview
🔍 Preview Mode - No changes will be made

📁 Processing: ~/Music/The Beatles/Abbey Road

🔄 Changes to apply:
  "come together" → "Come Together"
  "SOMETHING" → "Something" 
  "here comes the sun" → "Here Comes The Sun"

📊 Summary: 3 files need normalization
```

```
# Actual changes
✅ Applied changes to 3 files:

📁 ~/Music/The Beatles/Abbey Road
  🎵 "come together" → "Come Together"
  🎵 "SOMETHING" → "Something"
  🎵 "here comes the sun" → "Here Comes The Sun"

🎉 Normalization complete!
```

**JSON Output:**
```json
{
  "processed_files": 3,
  "changed_files": 3,
  "changes": [
    {
      "file": "/music/The Beatles/Abbey Road/01 - come together.flac",
      "old_title": "come together",
      "new_title": "Come Together"
    }
  ],
  "dry_run": false
}
```

**Use Cases:**
- Fix inconsistent capitalization
- Standardize track naming
- Prepare files for display
- Clean up messy metadata

### 📤 `emit` - Export Library Metadata

Export structured library data for analysis or AI processing.

```bash
musicctl emit [OPTIONS] <PATH>

Options:
  -f, --format <FORMAT>    Output format [default: text] [possible values: text, json]
  -v, --verbose            Include additional metadata

Arguments:
  <PATH>                   Directory to scan (required)
```

**Output Examples:**

```
# Text format (AI-friendly)
=== MUSIC LIBRARY METADATA ===
Total Artists: 2
Total Albums: 2
Total Tracks: 4

ARTIST: The Beatles
  ALBUM: Abbey Road (1969)
    TRACK: "Come Together" | Duration: 4:19 | File: /music/The Beatles/Abbey Road/01 - Come Together.flac
    TRACK: "Something" | Duration: 3:03 | File: /music/The Beatles/Abbey Road/02 - Something.flac

ARTIST: Pink Floyd
  ALBUM: The Dark Side of the Moon (1973)
    TRACK: "Speak to Me" | Duration: 1:13 | File: /music/Pink Floyd/The Dark Side of the Moon/01 - Speak to Me.flac
    TRACK: "On the Run" | Duration: 3:36 | File: /music/Pink Floyd/The Dark Side of the Moon/02 - On the Run.flac

=== END METADATA ===
```

**JSON Format:**
```json
{
  "artists": [
    {
      "name": "The Beatles",
      "albums": [
        {
          "title": "Abbey Road",
          "year": 1969,
          "tracks": [
            {
              "file_path": "/music/The Beatles/Abbey Road/01 - Come Together.flac",
              "metadata": {
                "title": { "value": "Come Together", "source": "Embedded", "confidence": 1.0 },
                "artist": { "value": "The Beatles", "source": "Embedded", "confidence": 1.0 },
                "album": { "value": "Abbey Road", "source": "Embedded", "confidence": 1.0 },
                "duration": { "value": 259.0, "source": "Embedded", "confidence": 1.0 }
              }
            }
          ]
        }
      ]
    }
  ],
  "total_tracks": 4,
  "total_artists": 2,
  "total_albums": 2
}
```

**Use Cases:**
- Export data for analysis
- Generate reports
- Feed data to AI systems
- Create library inventories
- Backup metadata

---

## 🔧 Advanced Usage

### Piping and Compositing

```bash
# Find files and process them
musicctl scan ~/Music | head -10 | xargs -I {} musicctl read {}

# Export specific metadata
musicctl emit ~/Music --json | jq '.artists[] | select(.name == "The Beatles")'

# Create file list with metadata
musicctl tree ~/Music --json | jq -r '.children[].albums[].tracks[].file_path'
```

### Batch Operations

```bash
# Normalize multiple directories
for dir in ~/Music/*/; do
  echo "Processing $dir"
  musicctl normalize "$dir" --dry-run
done

# Export metadata for each artist
musicctl scan ~/Music --json | jq -r '.files[]' | while read file; do
  artist=$(musicctl read "$file" --json | jq -r '.metadata.artist.value')
  echo "$file: $artist"
done
```

### Error Handling

```bash
# Continue on errors
musicctl scan ~/Music 2>/dev/null || echo "Scan completed with some errors"

# Validate files first
musicctl scan ~/Music | while read file; do
  if musicctl read "$file" >/dev/null 2>&1; then
    echo "✅ $file"
  else
    echo "❌ $file"
  fi
done
```

### Performance Tips

```bash
# Use JSON for faster processing
musicctl emit ~/Music --json > library.json

# Limit scope for large libraries
musicctl tree ~/Music/Rock --json

# Verbose mode for debugging
musicctl normalize ~/Music --verbose --dry-run
```

---

## 💡 Examples

### Library Analysis Workflow

```bash
# 1. Get library overview
echo "=== LIBRARY OVERVIEW ==="
musicctl emit ~/Music

# 2. Find files without proper metadata
echo "=== FILES MISSING METADATA ==="
musicctl scan ~/Music | while read file; do
  if ! musicctl read "$file" --compact | grep -q "|"; then
    echo "$file"
  fi
done

# 3. Normalize titles
echo "=== NORMALIZING TITLES ==="
musicctl normalize ~/Music --dry-run
read -p "Apply these changes? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
  musicctl normalize ~/Music
fi

# 4. Generate report
echo "=== LIBRARY REPORT ==="
musicctl emit ~/Music --json > library_report.json
echo "Report saved to library_report.json"
```

### Artist-Specific Operations

```bash
# Focus on specific artist
ARTIST="The Beatles"
BASE_DIR="$HOME/Music/$ARTIST"

# Show artist's discography
echo "=== $ARTIST DISCOGRAPHY ==="
musicctl tree "$BASE_DIR"

# Export artist's metadata
musicctl emit "$BASE_DIR" --json > "${ARTIST}_metadata.json"

# Normalize artist's tracks
musicctl normalize "$BASE_DIR" --dry-run
```

### Quality Assurance

```bash
# Check for common issues
echo "=== QUALITY CHECK ==="

# 1. Files with no metadata
musicctl emit ~/Music --text | grep "No metadata" || echo "✅ All files have metadata"

# 2. Inconsistent capitalization
echo "Files with lowercase titles:"
musicctl scan ~/Music | while read file; do
  title=$(musicctl read "$file" --json | jq -r '.metadata.title.value // empty')
  if [[ "$title" =~ ^[a-z] ]]; then
    echo "$file: '$title'"
  fi
done

# 3. Missing years
echo "Files missing year:"
musicctl emit ~/Music --json | jq -r '.artists[].albums[] | select(.year == null) | .title'
```

### Backup and Migration

```bash
# Export all metadata for backup
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="music_backup_$TIMESTAMP"

mkdir "$BACKUP_DIR"

# Export structured data
musicctl emit ~/Music --json > "$BACKUP_DIR/library_metadata.json"

# Export file list
musicctl scan ~/Music > "$BACKUP_DIR/file_list.txt"

# Create directory structure report
musicctl tree ~/Music > "$BACKUP_DIR/directory_structure.txt"

echo "Backup created in $BACKUP_DIR"
```

---

## 🔍 Troubleshooting

### Common Issues

#### 1. No Files Found

```bash
# Check if directory exists
ls ~/Music

# Check file permissions
ls -la ~/Music

# Use verbose mode for debugging
musicctl scan ~/Music --verbose
```

#### 2. Metadata Reading Errors

```bash
# Test specific file
musicctl read ~/Music/song.flac --verbose

# Check file format
file ~/Music/song.flac

# Validate FLAC file
flac -t ~/Music/song.flac
```

#### 3. Permission Issues

```bash
# Check directory permissions
ls -ld ~/Music

# Fix permissions if needed
chmod -R u+r ~/Music
```

#### 4. Performance Issues

```bash
# Process smaller subsets first
musicctl tree ~/Music/Rock

# Use JSON output for faster processing
musicctl emit ~/Music --json > /tmp/library.json

# Monitor progress
musicctl scan ~/Music --verbose | pv -l
```

### Debug Mode

```bash
# Enable debug logging
RUST_LOG=debug musicctl scan ~/Music

# Trace specific operations
RUST_LOG=trace musicctl read ~/Music/song.flac
```

### Getting Help

```bash
# General help
musicctl --help

# Command-specific help
musicctl scan --help
musicctl read --help
musicctl normalize --help
musicctl emit --help
musicctl tree --help
```

### Version Information

```bash
# Check version
musicctl --version

# Get build information
musicctl --version --verbose
```

---

## 📚 Additional Resources

- [MCP Server Documentation](MCP_SERVER.md) - AI agent integration
- [Configuration Examples](MCP_CONFIG_EXAMPLES.md) - Client setup
- [Project README](../README.md) - General project information
- [GitHub Issues](https://github.com/Komdosh/music-chore/issues) - Bug reports and feature requests

---

## 🎯 Best Practices

1. **Always dry-run first**: Use `--dry-run` with normalize operations
2. **Back up before bulk operations**: Export metadata before making changes
3. **Use JSON for scripting**: JSON output is more reliable for automation
4. **Process in chunks**: Large libraries benefit from incremental processing
5. **Validate after operations**: Use read commands to verify changes

---

**musicctl** - Your precision tool for music library organization. Built for humans, optimized for AI agents.