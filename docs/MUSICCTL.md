# 🎵 musicctl - Complete User Guide

`musicctl` is the command-line interface for the music-chore library management system. This comprehensive guide covers all commands, options, and usage patterns.

## 📅 Last Updated

- **Date**: February 7, 2026
- **Version**: v0.3.2 (post normalize refactor)
- **Features**: CLI with 9 commands + MCP server with 8 tools

## 📋 Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Command Reference](#command-reference)
  - [scan - Discover Music Files](#-scan---discover-music-files)
  - [tree - Display Library Hierarchy](#-tree---display-library-hierarchy)
  - [read - Extract File Metadata](#-read---extract-file-metadata)
  - [write - Update File Metadata](#-write---update-file-metadata)
  - [normalize - Normalize Track Metadata](#-normalize---normalize-track-metadata)
  - [validate - Validate Metadata Completeness](#-validate---validate-metadata-completeness)
  - [duplicates - Find Duplicate Tracks](#-duplicates---find-duplicate-tracks)
  - [cue - Generate, Parse, or Validate CUE Files](#-cue---generate-parse-or-validate-cue-files)
  - [emit - Export Library Metadata](#-emit---export-library-metadata)
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

### 4. Normalize Track Metadata

```bash
# Normalize titles and genres, outputs proposed changes (no file modification)
musicctl normalize ~/Music
```

### 5. Export Library Data

```bash
# AI-friendly format
musicctl emit ~/Music

# JSON for programming
musicctl emit ~/Music --json
```

### 6. Find Duplicate Tracks

```bash
# Find duplicates with human-readable output
musicctl duplicates ~/Music

# JSON output for automation
musicctl duplicates ~/Music --json
```

### 7. CUE File Operations

```bash
# Generate CUE sheet for an album (preview only)
musicctl cue --generate /path/to/album

# Parse existing CUE file
musicctl cue --parse /path/to/album.cue

# Validate CUE against audio files
musicctl cue --validate /path/to/album.cue
```

---

## 📖 Command Reference

### `musicctl` - Global Options

```bash
musicctl [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS]

Global Options:
  -h, --help        Print help information
  -V, --version     Print version information
```

### 🔍 `scan` - Discover Music Files

Recursively scan directories for supported audio files.

```bash
musicctl scan [OPTIONS] <PATH>

Options:
  --verbose         Show progress output during scanning
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

### 🔤 `normalize` - Normalize Track Metadata

Analyzes and reports on proposed title and genre normalization. This command will output details of recommended changes to track titles (to proper title case) and genres (to standardized forms) without modifying any files.

```bash
musicctl normalize [OPTIONS] <PATH>

Options:
  -j, --json        Output results as JSON

Arguments:
  <PATH>            Directory to process (required)
```

**Output Examples:**

```
# Human-readable output
--- Title Normalization ---
NORMALIZED: Title 'come together' -> 'Come Together' in /path/to/music/01 - come together.flac
NO CHANGE: Title 'Something' already normalized in /path/to/music/02 - Something.flac
Title Summary: 1 normalized, 1 no change, 0 errors

--- Genre Normalization ---
NORMALIZED: Genre 'rock and roll' -> 'Rock' in /path/to/music/01 - come together.flac
NO CHANGE: Genre 'Rock' already normalized in /path/to/music/02 - Something.flac
Genre Summary: 1 normalized, 1 no change, 0 errors
```

**JSON Output:**
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

**Use Cases:**
- Preview title and genre normalization changes
- Identify tracks with inconsistent metadata
- Generate reports for library cleanup efforts

### 🔍 `duplicates` - Find Duplicate Tracks

Identify duplicate audio files using SHA256 checksums.

```bash
musicctl duplicates [OPTIONS] <PATH>

Options:
  -j, --json        Output results as JSON

Arguments:
  <PATH>            Directory to scan (required)
```

**Output Examples:**

```
# Human-readable output
Found 2 duplicate groups:

Duplicate Group 1 (2 files):
  /music/The Beatles/Abbey Road/01 - Come Together.flac
  /music/Compilations/Best Of/01 - Come Together.flac

Duplicate Group 2 (3 files):
  /music/Pink Floyd/Dark Side/01 - Speak to Me.flac
  /music/Compilations/Psychedelic/01 - Speak to Me.flac
  /music/Backup/Pink Floyd Collection/01 - Speak to Me.flac
```

**JSON Output:**
```json
[
  [
    {
      "file_path": "/music/The Beatles/Abbey Road/01 - Come Together.flac",
      "metadata": {
        "title": { "value": "Come Together", "source": "Embedded", "confidence": 1.0 },
        "artist": { "value": "The Beatles", "source": "Embedded", "confidence": 1.0 },
        "album": { "value": "Abbey Road", "source": "Embedded", "confidence": 1.0 }
      },
      "checksum": "ae8850161fcc2cbda1d34e22d6813a75785128ca4c7d8df0ea05f89a16b53e22"
    },
    {
      "file_path": "/music/Compilations/Best Of/01 - Come Together.flac",
      "metadata": {
        "title": { "value": "Come Together", "source": "Embedded", "confidence": 1.0 },
        "artist": { "value": "The Beatles", "source": "Embedded", "confidence": 1.0 },
        "album": { "value": "Best Of", "source": "Embedded", "confidence": 1.0 }
      },
      "checksum": "ae8850161fcc2cbda1d34e22d6813a75785128ca4c7d8df0ea05f89a16b53e22"
    }
  ]
]
```

**Advanced Examples:**

```bash
# Find duplicates and count them
musicctl duplicates ~/Music --json | jq 'length'

# Get total files that are duplicates
musicctl duplicates ~/Music --json | jq 'map(length) | add'

# Find duplicates by specific artist
musicctl emit ~/Music --json | jq '.artists[] | select(.name == "The Beatles")'

# Export duplicate list for cleanup
musicctl duplicates ~/Music --json > duplicates.json
```

**Use Cases:**
- **Library Cleanup**: Remove redundant copies to save disk space
- **Organization**: Identify duplicate tracks for playlist management
- **Quality Control**: Keep only the best quality versions
- **Migration Planning**: Avoid importing duplicates when consolidating libraries
- **Storage Analysis**: Understand duplicate impact on storage usage
- **Metadata Comparison**: Compare metadata quality between duplicates

**Performance Notes:**
- Checksum calculation is CPU-intensive for large libraries
- First run will be slower as checksums are calculated
- Results are deterministic - same input always produces same output
- JSON output includes full metadata for each duplicate file

### 💿 `cue` - Generate, Parse, or Validate CUE Files

Performs various operations on CUE sheet files. This command replaces the separate `cue-parse` and `cue-validate` commands.

```bash
musicctl cue [OPTIONS] <PATH>

Arguments:
  <PATH>              Path to the album directory or .cue file (required)

Options:
  --generate          Generate a CUE sheet from an album directory.
  --parse             Parse an existing CUE file.
  --validate          Validate a CUE file against its referenced audio files.
  -o, --output <PATH> Output path for generated .cue file (--generate only).
  --audio-dir <PATH>  Directory containing audio files (--validate only, defaults to CUE file directory).
  -d, --dry-run       Preview changes without writing (--generate only).
  -f, --force         Overwrite existing .cue file (--generate only).
  -j, --json          Output results as JSON (--parse and --validate only).
```

**Output Examples (Generate Dry Run):**

```
# Dry run preview
Would write to: /music/Album/Album Name.cue

PERFORMER "Artist Name"
TITLE "Album Name"
REM GENRE Rock
REM DATE 2024
FILE "01. Track One.flac" WAVE
  TRACK 01 AUDIO
    TITLE "Track One"
    PERFORMER "Artist Name"
    INDEX 01 00:00:00
  TRACK 02 AUDIO
    TITLE "Track Two"
    PERFORMER "Artist Name"
    INDEX 01 00:03:00
```

**Output Examples (Parse Human-readable):**

```
Cue File: /music/Album/Album.cue
  Performer: Super Artist
  Title: Meanings
  Files:
    - 01. A New Journey Begins.flac
    - 02. Time Goes On.flac
  Tracks: 2
    Track 01: A New Journey Begins [01. A New Journey Begins.flac]
    Track 02: Time Goes On [02. Time Goes On.flac]
```

**Output Examples (Parse JSON):**
```json
{
  "performer": "Super Artist",
  "title": "Meanings",
  "genre": "Ambient",
  "date": "2024",
  "files": [
    "01. A New Journey Begins.flac",
    "02. Time Goes On.flac"
  ],
  "tracks": [
    {
      "number": 1,
      "title": "A New Journey Begins",
      "performer": "Super Artist",
      "index": "00:00:00",
      "file": "01. A New Journey Begins.flac"
    }
  ]
}
```

**Output Examples (Validate Human-readable):**

```
# Valid CUE file
✓ CUE file is valid
  All referenced files exist and track count matches.
```

**Output Examples (Validate JSON):**
```json
{
  "is_valid": false,
  "parsing_error": false,
  "file_missing": true,
  "track_count_mismatch": false
}
```

**CUE Generation Features:**
- Extracts metadata from FLAC, MP3, WAV, DSF, and WavPack files
- Generates standard CUE sheets compatible with most audio players
- Supports multiple files per album (one FILE entry per track)
- Includes genre and year from track metadata
- Normalizes text to title case
- Embeds album artist if available (takes precedence over track artist)

**CUE Parsing Features:**
- Parses album-level metadata (performer, title, genre, date, files)
- Parses track-level metadata (number, title, performer, index)
- Handles multi-file CUE sheets correctly
- JSON output for automation and AI agents

**CUE Validation Features:**
- Checks that referenced audio files exist in the directory
- Validates track count consistency between CUE and audio files
- Supports custom audio directory for validation
- Parsing errors are reported as validation failures

**Use Cases:**
- Create disc images for burning or virtual drives
- Generate playlists for audio software
- Archive album metadata in standard format
- Share album tracklists with precise timing information
- Verify CUE file contents before burning
- Extract tracklist information
- Integrate with automated workflows
- Parse CUE files for AI analysis
- Verify CUE file integrity before burning discs
- Check that all referenced audio files are present
- Validate CUE files in automated workflows
- Debug CUE file issues

### 📤 `emit` - Export Library Metadata

Export structured library data for analysis or AI processing.

```bash
musicctl emit [OPTIONS] <PATH>

Options:
  -j, --json             Output results as JSON

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
  musicctl normalize "$dir"
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
musicctl scan ~/Music --verbose
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

# 3. Normalize titles and genres
echo "=== NORMALIZING TITLES AND GENRES ==="
musicctl normalize ~/Music

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
musicctl normalize "$BASE_DIR"
```

### Quality Assurance

```bash
# Check for common issues
echo "=== QUALITY CHECK ==="

# 1. Files with no metadata
musicctl emit ~/Music --json | jq -c '.artists[] | .albums[] | .tracks[] | select(.metadata.title.value == null or .metadata.artist.value == null or .metadata.album.value == null)' | while read track; do
  file_path=$(echo "$track" | jq -r '.file_path')
  echo "File missing metadata: $file_path"
done


# 2. Inconsistent capitalization (example: check for non-normalized titles)
echo "Files with non-normalized titles:"
musicctl normalize ~/Music --json | jq -c '.title_reports[] | select(.changed == true)' | while read report; do
  file_path=$(echo "$report" | jq -r '.original_path')
  original_title=$(echo "$report" | jq -r '.original_title')
  normalized_title=$(echo "$report" | jq -r '.normalized_title')
  echo "$file_path: '$original_title' -> '$normalized_title'"
done


# 3. Inconsistent genres
echo "Files with non-normalized genres:"
musicctl normalize ~/Music --json | jq -c '.genre_reports[] | select(.changed == true)' | while read report; do
  file_path=$(echo "$report" | jq -r '.original_path')
  original_genre=$(echo "$report" | jq -r '.original_genre')
  normalized_genre=$(echo "$report" | jq -r '.normalized_genre')
  echo "$file_path: '$original_genre' -> '$normalized_genre'"
done
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
musicctl write --help
musicctl normalize --help
musicctl validate --help
musicctl duplicates --help
musicctl cue --help
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

1. **Back up before bulk operations**: Export metadata before making changes
2. **Use JSON for scripting**: JSON output is more reliable for automation
3. **Process in chunks**: Large libraries benefit from incremental processing
4. **Validate after operations**: Use read commands to verify changes

---

**musicctl** - Your precision tool for music library organization. Built for humans, optimized for AI agents.