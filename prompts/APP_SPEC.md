# App Spec - music-chore

## Purpose

`music-chore` is a Rust CLI + MCP server for local music library organization and metadata management.

Core outcomes:
- scan local libraries
- infer Artist -> Album -> Track structure
- read/write metadata
- normalize metadata reports
- validate library quality
- detect duplicates
- manage CUE files
- provide AI-friendly structured outputs

## Constraints

- CLI only (no GUI)
- macOS/Linux
- Rust
- no playback
- local-library-first metadata workflows
- deterministic behavior
- modular, extensible design
- test coverage for public functions

## Architecture

- `src/core/domain`: models + schema wrappers
- `src/core/services`: business logic
- `src/adapters`: format handlers
- `src/presentation/cli`: CLI commands/handlers
- `src/mcp`: MCP tools/prompts/server

Binaries:
- `musicctl`
- `musicctl-mcp`

## Current CLI Commands

- `scan`
- `tree`
- `read`
- `write`
- `normalize`
- `emit`
- `validate`
- `duplicates`
- `cue`

## Current MCP Tools

- `scan_directory`
- `get_library_tree`
- `read_file_metadata`
- `normalize`
- `emit_library_metadata`
- `validate_library`
- `find_duplicates`
- `cue_file`

## Current MCP Prompts (minimal set)

- `listen-now`
- `web-perfect-match`
- `library-health-check`
- `metadata-cleanup-guide`
- `duplicate-resolution`
- `cue-sheet-assistant`

## Format Support (current codebase)

- FLAC
- MP3
- WAV
- OGG
- DSF
- WavPack

## Design Principles

1. small, composable modules
2. explicit metadata provenance
3. format-agnostic abstractions
4. deterministic outputs
5. machine-readable output first where applicable
6. safe, explicit mutation paths

## Definition of Done (feature changes)

1. behavior implemented in correct layer
2. CLI/MCP behavior verified with fixtures
3. tests added/updated where needed
4. docs updated when interfaces changed
5. clean, reviewable commit
