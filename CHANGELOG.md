# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
