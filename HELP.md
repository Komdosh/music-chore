# musicctl - Music Library Organizer and Normalizer

A CLI tool for organizing and normalizing local music libraries using existing file metadata and directory structure only.

## Overview

musicctl is a Rust-based CLI tool for macOS and Linux that organizes and normalizes a local music library using existing file metadata and directory structure only. It reads, normalizes, displays, and edits metadata locally without using the internet.

## Installation

```bash
# Build from source
cargo build --release
./target/release/musicctl --help
```

## Commands

### scan
Recursively scan a directory for music files.

```bash
# Basic scan
musicctl scan /path/to/music/library

# Scan with JSON output
musicctl scan /path/to/music/library --json

# Verbose scan
musicctl scan /path/to/music/library --verbose
```

### tree
Show a human-friendly tree view of your music library.

```bash
# Display library as tree
musicctl tree /path/to/music/library

# Display with JSON output
musicctl tree /path/to/music/library --json
```

### read
Read metadata from a single audio file.

```bash
# Read metadata from a file
musicctl read /path/to/file.flac
musicctl read /path/to/file.mp3
musicctl read /path/to/file.wav
musicctl read /path/to/file.dsf
musicctl read /path/to/file.wv
```

### write
Write metadata to an audio file.

```bash
# Dry run - show what would be changed
musicctl write /path/to/file.flac --set title="New Title" --set artist="New Artist" --dry-run

# Actually apply changes
musicctl write /path/to/file.flac --set title="New Title" --set artist="New Artist" --apply
```

### normalize
Normalize track titles to title case, or normalize genres.

```bash
# Normalize titles (dry run)
musicctl normalize /path/to/music/library --dry-run

# Normalize titles (apply changes)
musicctl normalize /path/to/music/library

# Normalize genres (dry run)
musicctl normalize /path/to/music/library --genres --dry-run

# Normalize genres (apply changes)
musicctl normalize /path/to/music/library --genres
```

### emit
Emit library metadata in structured JSON format.

```bash
# Emit library metadata
musicctl emit /path/to/music/library

# Emit with JSON output
musicctl emit /path/to/music/library --json
```

### cue
Generate, parse, or validate .cue files.

```bash
# Generate a CUE file for an album
musicctl cue --generate /path/to/album/directory

# Parse a CUE file
musicctl cue --parse /path/to/file.cue

# Validate a CUE file
musicctl cue --validate /path/to/file.cue
```

### validate
Validate metadata completeness and consistency.

```bash
# Validate library
musicctl validate /path/to/music/library

# Validate with JSON output
musicctl validate /path/to/music/library --json
```

### duplicates
Detect duplicate tracks by checksum.

```bash
# Find duplicates
musicctl duplicates /path/to/music/library

# Find duplicates with JSON output
musicctl duplicates /path/to/music/library --json
```

## Supported Audio Formats

- **FLAC** (.flac) - Full metadata read/write support
- **MP3** (.mp3) - Full metadata read/write support
- **WAV** (.wav) - Full metadata read/write support
- **DSF** (.dsf) - Full metadata read/write support
- **WavPack** (.wv) - Full metadata read/write support

## Metadata Source Tracking

The tool tracks where metadata comes from:

- **Embedded**: From file tags (confidence: 1.0)
- **FolderInferred**: Inferred from directory structure (confidence: 0.3)
- **UserEdited**: Set by user/cli (confidence: 1.0)

## Directory Structure Inference

The tool infers metadata from directory structure:

```
/Artist Name/
  /Album Name/
    track1.flac
    track2.flac
```

Will infer:
- Artist: "Artist Name"
- Album: "Album Name"
- Track: Individual track names

## Examples

### Organize a music library
```bash
# Scan your library
musicctl scan ~/Music

# View the structure
musicctl tree ~/Music/my-band

# Read metadata from a specific file
musicctl read ~/Music/my-band/album/track01.flac

# Normalize titles in an album
musicctl normalize ~/Music/my-band/album --dry-run
musicctl normalize ~/Music/my-band/album  # Apply changes

# Validate the library
musicctl validate ~/Music/my-band

# Find duplicates
musicctl duplicates ~/Music
```

### Working with CUE files
```bash
# Generate a CUE file for an album
musicctl cue --generate ~/Music/artist/album

# Validate the generated CUE file
musicctl cue --validate ~/Music/artist/album/album.cue
```

## Configuration

No configuration file is needed. All settings are provided via command-line arguments.

## Troubleshooting

### Common Issues

1. **Permission errors**: Ensure you have read/write permissions for the directories and files you're working with.

2. **Unsupported format errors**: The tool only supports the formats listed above. Convert unsupported files to a supported format first.

3. **Metadata not updating**: Remember to use the `--apply` flag when writing metadata; by default, operations are dry runs.

### Verbose Output

Use the `--verbose` flag with commands to get more detailed output and debugging information.

## Exit Codes

- 0: Success
- 1: General error
- 2: Command-line argument error

## MCP Server

The application also includes an MCP server for AI agent integration:

```bash
# Start the MCP server
cargo run --bin musicctl-mcp
```

## Contributing

See the project's GitHub repository for contribution guidelines.

## License

This project is licensed under the terms specified in the LICENSE file.