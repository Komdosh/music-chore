# AGENTS.md - Unified Coding Agent Guide

You are Staff Rust Engineer. Prioritize correctness, clarity, deterministic behavior, and practical outcomes.

This file is the single source of truth for agent guidance in this repository.

## Start Here

- Read `prompts/AGENT_INSTRUCTIONS.md` for task-specific agent behavior.
- Read `prompts/APP_SPEC.md` for product scope/details.
- Work only inside this repository.
- For feature work, use a dedicated branch and open a PR.

## Project Overview

`music-chore` is a Rust CLI + MCP server for local music library management.

Primary goals:
- Scan local libraries recursively.
- Infer and manage Artist -> Album -> Track structure.
- Read/write audio metadata locally.
- Normalize metadata.
- Validate library health.
- Detect duplicates.
- Generate/parse/validate CUE files.
- Provide machine-readable outputs for AI-agent workflows.

## Hard Constraints

- CLI only (no GUI)
- macOS and Linux
- Rust only
- No playback functionality
- No external metadata services by default (local-data-first design)
- Modular/extensible architecture (trait-based format handlers)
- Every public function must be tested
- Prefer correctness and maintainability over cleverness

## Architecture

- Domain: `src/core/domain`
- Services: `src/core/services`
- Adapters/format handlers: `src/adapters`
- CLI layer: `src/presentation/cli`
- MCP server: `src/mcp`

Main binaries:
- `musicctl`
- `musicctl-mcp`

## Supported Formats

Current codebase includes handlers for:
- FLAC
- MP3
- WAV
- DSF
- WavPack

Future expansion should remain trait-driven via `AudioFile`-style abstractions.

## Key CLI Capabilities

- `scan`
- `tree`
- `read`
- `write`
- `normalize`
- `emit`
- `validate`
- `duplicates`
- `cue`

## MCP Tools

- `scan_directory`
- `get_library_tree`
- `read_file_metadata`
- `normalize`
- `emit_library_metadata`
- `validate_library`
- `find_duplicates`
- `cue_file`

## Design Principles

1. Small, composable modules
2. Clear and explicit data models
3. Format-agnostic interfaces
4. Metadata provenance awareness
5. Deterministic outputs
6. AI-friendly structured output
7. Graceful handling of partial failures
8. Incremental workflows where possible

## Development Commands

```bash
cargo build
cargo build --release
cargo test
cargo test test_name
cargo test --test integration_test_name
cargo check
cargo fmt
cargo clippy
cargo clippy --all-targets --all-features
cargo run --bin musicctl
cargo run --bin musicctl-mcp
```

## Coding Standards

### Imports and naming
- Prefer `use crate::...` for internal modules.
- Keep imports grouped and clean.
- Types: PascalCase
- Functions/files: snake_case
- Constants: SCREAMING_SNAKE_CASE

### Error handling
- Use explicit custom error types.
- Implement useful `Display` messages.
- Return `Result<T, E>` for fallible paths.
- Avoid `unwrap()` in production code.

### Data/model conventions
- Prefer owned `String` fields in persisted structs.
- Use `PathBuf` for stored paths and `&Path` in APIs.
- Keep metadata source/provenance explicit.

### Code organization
- Keep modules focused.
- Separate domain logic from I/O and adapters.
- Re-export public API intentionally in `lib.rs`.

## Testing Requirements

- All public functions require tests.
- Unit tests in-module with `#[cfg(test)]`.
- Integration tests in `tests/`.
- Cover success and failure paths.
- Use `tempfile` for filesystem tests.
- Never mutate fixture originals; copy fixtures to temp locations.

## Documentation Requirements

- Use module docs (`//!`) for non-trivial modules.
- Add `///` docs for public functions/types.
- Keep docs concrete and behavior-focused.

## AI-Agent Output Expectations

- Prefer structured, parseable outputs.
- Avoid ambiguous prose when actionable output is expected.
- Provide exact commands for remediation workflows where helpful.

## Working Style

- Ask at most one clarifying question if truly blocked.
- Otherwise make reasonable assumptions and state them.
- Avoid over-engineering.
- Keep scope tight; do not add out-of-scope features.

## Non-Goals

- No GUI
- No streaming/player features
- No uncontrolled internet dependency for core metadata inference

## Summary

Build `music-chore` as a Unix-style toolchain component:
- small
- sharp
- predictable
- composable

When in doubt, simplify.
