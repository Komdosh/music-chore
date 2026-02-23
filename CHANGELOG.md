# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [0.4.8] - 2026-02-23

### Changed
- **Duplicates Tool**: `find_duplicates` now returns a successful result when no duplicates exist instead of treating it as an error.
- **MCP Behavior**: Updated MCP `find_duplicates` behavior so "No duplicate tracks found" is returned as success text.
- **Tests**: Added duplicate service regression tests for no-duplicate text/JSON success cases and updated MCP integration expectations.
## [0.4.7] - 2026-02-23

### Added
- **Format Support**: Added M4A (`.m4a`) support with a dedicated handler registered in the audio format registry.
- **Integration Tests**: Added M4A integration coverage for format detection, scanner behavior, and read/write/basic-info routing on invalid file content.

### Changed
- **Metadata Extraction Tests**: Added exhaustive branch coverage for M4A `extract_metadata_from_tags` and `extract_basic_metadata`, including parse fallbacks and folder inference behavior.
- **Registry and Scanner Tests**: Updated format registry and unsupported-file tests to treat M4A as supported and keep unsupported sentinel coverage on truly unsupported extensions.
- **CLI/Docs/Spec**: Updated supported-format references to include M4A.

## [0.4.6] - 2026-02-23

### Added
- **Format Support**: Added OGG (`.ogg`) support with dedicated handler integration in the format registry and CLI/MCP flows.
- **Integration Tests**: Added OGG integration coverage for format detection, scanner behavior, and read/write routing for invalid files.

### Changed
- **Metadata Extraction Tests**: Added exhaustive branch coverage for OGG `extract_basic_metadata` and `extract_metadata_from_tags`, including inference fallbacks and date/number parsing branches.
- **Docs/Spec**: Updated supported format references across project docs and spec prompt files to include OGG.

## [0.4.5] - 2026-02-23

### Changed
- **MCP Prompts**: Reduced the expert prompt surface from 26 to 6 high-value prompts focused on core listening decisions and essential library maintenance.
- **Skills**: Consolidated skill catalog to a minimal set (`listen-now`, `web-perfect-match`, `library-maintenance`) to reduce overlap and improve adoption.
- **Documentation**: Updated MCP prompt counts and categories in `README.md` and `docs/MCP_SERVER.md`.

### Removed
- **Prompt Overlap**: Removed low-usage and redundant prompt variants (analysis/story/novelty and specialized sub-variants now covered by core prompts).
- **Skill Overlap**: Removed specialized skill folders that duplicated capabilities now unified under the compact core set.

## [0.4.4] - 2026-02-10

### Changed
- **MCP Server**: Wrapped `scan_directory` tool response in a structured JSON object (`{"tracks": [...]}`) for more consistent client-side handling.
- **Scanner**: Reduced log noise by changing several non-critical errors (invalid glob patterns, CUE parsing failures, invalid files) from error to debug level.

## [0.4.3] - 2026-02-10

### Added
- **MCP Server**: Derived `schemars::JsonSchema` for core models and error types, improving automated tool schema generation.

### Changed
- **Dependencies**: Upgraded `rmcp` to `0.15.0` and `lofty` to `0.23.1`.
- **Refactoring**: Improved JSON schema structure by removing the redundant `$schema` field from `SchemaVersionWrapper`.

## [0.4.2] - 2026-02-10

### Added
- **MCP Server Improvements**: Enhanced structured JSON output for several tools.
- **Tools**: Added optional `json_output` parameter to `get_library_tree` and `read_file_metadata` MCP tools.
- **Schema**: Added `$schema` field to `SchemaVersionWrapper` for better JSON schema compliance.
- **Integration Tests**: Updated tests to verify JSON output capabilities in the MCP server.

## [0.4.1] - 2026-02-10

### Added
- **Performance**: Parallelized duplicate detection using `rayon`.
- **CLI**: Added `--parallel` and `--verbose` flags to the `duplicates` command.

## [0.4.0] - 2026-02-10

### Added
- **MCP Server Enhancements**: Added expert AI prompts, security path validation, and environment-based configuration.
- **Documentation**: Comprehensive updates to `GEMINI.md`, `README.md`, and architectural docs.
- **Refactoring**: Massive core refactoring for better error handling, modularity, and performance.
- **Reliability**: Significantly expanded test coverage (640+ tests).

## [0.3.2] - 2026-02-09

### Added
- **Normalization**: Consolidated `normalize` and `normalize-genres` commands with support for artist, album, and year.
- **CLI Improvements**: Dry-run mode is now the default for operations that modify files.
- **Scan Output**: Improved `scan` command output with source icons and track names.

## [0.3.1] - 2026-02-08

### Added
- **Format Support**: Read-only support for DSF audio files.
- **CUE Prioritization**: `tree` command now prioritizes CUE file metadata for hierarchical display.

## [0.3.0] - 2026-02-07

### Added
- **Format Support**: Added support for WavPack (`.wv`) files.
- **Scanner**: Added symlink support, `--max-depth` flag, and exclusion patterns.
- **Metadata Validation**: New `validate` command for checking metadata completeness against a schema.
- **CLI Progress**: Added verbose progress reporting during scans.

## [0.2.3] - 2026-02-06

### Added
- **CUE Parsing**: Support for parsing existing `.cue` files, including multiple `FILE` entries and `REM GENRE/DATE` fields.
- **MCP Tools**: Integrated CUE parsing and validation into the MCP server.

## [0.2.2] - 2026-02-05

### Added
- **CUE Generation**: New command to generate `.cue` sheets from directory contents.
- **Dry-Run**: Support for dry-run CUE generation.

## [0.2.1] - 2026-02-04

### Added
- **Format Support**: Initial support for WAV files.
- **Tree Refinement**: Preserved directory structure in hierarchical visualization.

## [0.2.0] - 2026-02-03

### Added
- **Format Support**: Full MP3 format support with ID3 metadata handling.
- **Duplicate Detection**: SHA256-based duplicate detection command and MCP tool.
- **Unicode**: Full support for Unicode file paths.

## [0.1.7] - 2026-02-02

### Added
- **MCP Validation**: Added `validate_library` tool to the MCP server.

## [0.1.6] - 2026-02-02

### Added
- **Validation**: Initial version of the `validate` command for metadata health checks.

## [0.1.4] - 2026-02-01

### Added
- **Metadata Writing**: New `write` command with dry-run capabilities for manual metadata updates.

## [0.1.3] - 2026-02-01

### Added
- **MCP Migration**: Switched to `rmcp` library for more robust Model Context Protocol support.

## [0.1.2] - 2026-01-31

### Added
- **CLI**: Added `-v/--version` flag.

## [0.1.1] - 2026-01-31

### Added
- **Emit Command**: Added `emit` command to export library metadata as structured JSON.

## [0.1.0] - 2026-01-31

### Initial Release
- Core CLI functionality: `scan`, `tree`, `normalize`.
- Support for FLAC metadata extraction.
- Basic Model Context Protocol (MCP) server integration.
- Automated CI/CD with GitHub Actions.
