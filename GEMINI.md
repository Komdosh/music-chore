# GEMINI.md - Project Context for Gemini AI

This document provides a comprehensive overview of the `music-chore` project, designed to assist AI agents in understanding its purpose, architecture, and operational procedures.

## Project Overview

`music-chore` is a command-line interface (CLI) tool written in Rust for organizing, normalizing, and managing local music libraries. It is designed for precision and offers a range of functionalities, including:

*   **Recursive Directory Scanning:** Efficiently discovers music files within specified directories.
*   **Metadata Extraction & Management:** Supports reading and writing metadata for FLAC, MP3, WAV, DSF, and WavPack audio formats.
*   **Hierarchical Inference:** Infers `Artist` → `Album` → `Track` structures from file paths.
*   **Normalization:** Provides tools for title capitalization and genre standardization (mapping over 40 genre variants).
*   **Library Visualization:** Generates a tree-like view of the music library structure.
*   **Duplicate Detection:** Identifies duplicate audio files using SHA256 checksums.
*   **CUE File Operations:** Supports generating, parsing, and validating CUE sheets.
*   **Metadata Validation:** Checks the quality and completeness of metadata against a defined schema.
*   **Structured Output:** Emits structured metadata, particularly useful for AI integrations.
*   **Expert AI Prompts:** Provides specialized prompts for AI agents to perform complex library analysis, health checks, and curated discovery.
*   **Model Context Protocol (MCP) Server:** Includes an integrated server (`musicctl-mcp`) that exposes `music-chore`'s functionalities as tools for AI agents, facilitating direct interaction and automation.

The project is built with AI agents in mind, offering a robust and automatable solution for music library maintenance.

## Technologies Used

*   **Primary Language:** Rust
*   **Build System:** Cargo (Rust's package manager and build system)

## Architecture Highlights

The project follows a modular architecture, organized within the `src/` directory:

*   `src/domain/`: Defines the core data models (e.g., `Artist`, `Album`, `Track`) and business entities.
*   `src/adapters/audio_formats/`: Contains format-specific handlers for different audio file types, adhering to an `AudioFile` trait for extensibility.
*   `src/core/services/`: Implements the main business logic and operations (e.g., scanning, normalization, validation).
*   `src/presentation/cli/`: Handles the command-line interface parsing and command execution for `musicctl`.
*   `src/mcp/`: Houses the implementation for the Model Context Protocol (MCP) server (`musicctl-mcp`), exposing the project's capabilities to AI agents.

This design promotes separation of concerns and allows for easy extension, particularly for adding support for new audio formats.

## Building and Running

### Prerequisites

*   Rust toolchain (installable via `rustup`).

### Building the Project

To build the `music-chore` CLI and MCP server:

```bash
cargo build
```

For an optimized release build:

```bash
cargo build --release
```

The executables (`musicctl` and `musicctl-mcp`) will be located in `target/debug/` or `target/release/` respectively.

### Running the CLI (`musicctl`)

Examples of common `musicctl` commands:

*   **Scan a music library:**
    ```bash
    ./target/debug/musicctl scan /path/to/your/music
    ```
*   **View library structure:**
    ```bash
    ./target/debug/musicctl tree /path/to/your/music
    ```
*   **Validate metadata:**
    ```bash
    ./target/debug/musicctl validate /path/to/your/music
    ```
*   **Normalize titles (dry run):**
    ```bash
    ./target/debug/musicctl normalize /path/to/music --dry-run
    ```
*   **Apply normalization (modifies files):**
    ```bash
    ./target/debug/musicctl normalize /path/to/music --apply
    ```

### Running the MCP Server (`musicctl-mcp`)

The MCP server allows AI agents to interact with `music-chore` programmatically.

```bash
./target/debug/musicctl-mcp
```

Refer to the `README.md` for specific integration instructions with AI platforms like Claude Desktop.

## Testing

The project includes a comprehensive suite of unit and integration tests.

*   **Run all tests:**
    ```bash
    cargo test
    ```
*   **Run a specific test:**
    ```bash
    cargo test test_name
    ```

## Development Conventions and Quality Checks

*   **Code Formatting:** Adheres to Rust's idiomatic formatting.
    ```bash
    cargo fmt
    ```
*   **Linting:** Uses Clippy for static analysis and linting.
    ```bash
    cargo clippy
    ```
*   **Type Checking:** Ensures type correctness without building.
    ```bash
    cargo check
    ```

These commands should be run regularly during development to maintain code quality and consistency.

## Supported Audio Formats

| Format  | Read | Write |
| :------ | :--: | :---: |
| FLAC    | ✅   | ✅    |
| MP3     | ✅   | ✅    |
| WAV     | ✅   | ✅    |
| DSF     | ✅   | ✅    |
| WavPack | ✅   | ✅    |

Planned future support includes OGG and M4A.