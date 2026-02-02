# Music Library Organizer - App Specification

## Project Overview
A Rust-based CLI tool that organizes and normalizes local music libraries using existing file metadata and directory structure only.

## Core Goal
Design a CLI-only Rust program that:
- Recursively scans a working directory
- Discovers music files and albums
- Reads and writes metadata
- Reads and writes .cue files
- Exposes structured, machine-readable output suitable for AI agents
- Supports incremental updates when new folders are added

## Hard Constraints
- CLI only (no GUI)
- Rust language
- No internet access or external metadata lookup
- No playback functionality
- Modular architecture (future formats must be easy to add)
- Every public function must be covered by tests
- Prefer correctness and clarity over cleverness

## Supported File Formats
### v1 (Initial Scope)
- .flac only

### v2 (Future Scope)
- .mp3
- .wav
- .dsf

## Versioned Feature Scope

### v1 — Foundation
- Recursive directory scanning
- Artist → Album → Track hierarchy inference
- CLI command to display a tree structure:
  * Artist
    * Album
      * Tracks
- CLI command to read metadata from a .flac file
- CLI command to write/update metadata in a .flac file
- Internals designed to support multiple formats later

### v2 — AI Agent Integration
Planned but not implemented in v1:
- MCP awareness (agent-friendly command structure and output)
- Support additional file formats
- Generate a .cue file for an album folder using track metadata

## Explicit Non-Goals
- Fetch metadata from the internet
- Use MusicBrainz, Discogs, or similar databases
- Guess artist/album info beyond local inference
- Implement media playback
- Implement a GUI

## Design Principles
1. Small, composable modules
2. Clear data models for Artist, Album, Track, Metadata
3. Format-agnostic interfaces (traits) for audio files
4. Deterministic behavior — same input yields same output
5. AI-friendly output (structured, predictable, machine-readable when useful)
6. Incremental processing — new folders can be scanned without full rebuild

## Expected Planning Outputs
1. High-level architecture diagrams (conceptual, not graphical)
2. Proposed module layout
3. Core data structures and traits
4. CLI command and flag design
5. Step-by-step implementation plan
6. Testing strategy
7. Design trade-offs and rationale

## AI-Agent Awareness
Assume this tool will be called by another AI agent via MCP.
Therefore:
- CLI output should be easy to parse
- Commands should be composable and scriptable
- Errors should be structured and descriptive
- Avoid ambiguous or human-only output unless explicitly requested

## Working Style Instructions
- Ask at most one clarifying question if absolutely necessary
- Otherwise, make reasonable assumptions and state them
- Do not over-engineer
- Do not introduce features outside the defined scope
- Keep v1 minimal and rock-solid

## Summary
Building a precise, local, metadata-focused music library tool that serves both humans and AI agents.
Design it like a Unix tool:
- Small
- Sharp
- Predictable
- Composable

