# Plan for v1 Implementation

## 1. Core Modules
- `src/metadata/` – data structures (`Artist`, `Album`, `Track`, `Metadata`) and trait `AudioFormat` with `read_metadata`/`write_metadata`.
- `src/fs/` – recursive directory walker, folder‑to‑hierarchy inference.
- `src/cue/` – parsing and generating `.cue` files (placeholder for future).
- `src/cli/` – command line entry point using `clap`.

## 2. CLI Commands
- `list [--json]` – prints a tree view; JSON flag for AI parsing.
- `read-metadata <path>` – outputs metadata for a specific file.
- `write-metadata <path> [options]` – updates fields (artist, album, title, track number, etc.).

## 3. Tests
- Unit tests for each module.
- Integration test for `list` using a small sample directory.

## 4. Implementation Steps
1. Create module skeletons and Cargo.toml.
2. Implement filesystem walk and hierarchy inference.
3. Implement FLAC metadata read/write via `lofty` crate.
4. Wire CLI commands and flags.
5. Add tests.

## 5. Verification
- Run `cargo test`.
- Execute CLI commands against sample data.

This plan aligns with the project spec and focuses on v1 features only.
