---
name: musicctl
description: A set of tools to organize, normalize, and manage local music libraries via the Model Context Protocol (MCP) server for AI agents.
license: N/A
---

## When to use this skill
To manage and organize music libraries, use this skill for:
- Scanning music directories to discover tracks.
- Understanding the hierarchical structure (Artist -> Album -> Track) of music libraries.
- Reading detailed metadata from individual music files.
- Normalizing track titles and other textual metadata.
- Emitting comprehensive, structured metadata for the entire library.
- Validating music libraries for common issues and inconsistencies.
- Finding duplicate music files using checksums.
- Generating, parsing, or validating CUE sheet files.

## How to use this skill

To interact with the `musicctl` tool via MCP:

1.  **Identify the specific music management task** you need to perform (e.g., `scan_directory`, `normalize_titles`, `cue_file`).
2.  **Consult the detailed tool descriptions** within the `Available Skills` section below for exact parameters and expected outputs for the chosen task.
3.  **Construct a `call_tool` command** using the tool name and its specific parameters.
    *   Always set `json_output: true` when available for machine-readable results.
    *   For operations that modify files (e.g., `normalize_titles`, `cue_file generate`), always perform a `dry_run: true` first to preview changes before applying them (`dry_run: false`).
4.  **Process the `CallToolResult`** to retrieve the successful output or handle any `McpError` gracefully.

## Keywords
music, library, organize, metadata, normalize, validate, duplicates, cue, MCP, AI agent, musicctl

## Available Skills

### 1. `scan_directory`

*   **Description:** Recursively scans a specified directory for music files and lists the tracks found.
*   **Parameters:**
    *   `path` (string, **required**): The absolute path to the directory to scan.
    *   `json_output` (boolean, optional): If `true`, output will be in JSON format. Defaults to `false` (plain text).
*   **Expected Output:** A list of scanned music tracks, either as plain text or JSON, indicating their paths and basic inferred metadata.
*   **Example Call (pseudocode):**
    ```
    call_tool("scan_directory", {"path": "/path/to/music", "json_output": true})
    ```

### 2. `get_library_tree`

*   **Description:** Generates a hierarchical tree-like view of the music library structure (Artist -> Album -> Track) from a given path.
*   **Parameters:**
    *   `path` (string, **required**): The absolute path to the root of the music library.
    *   `json_output` (boolean, optional): If `true`, output will be in JSON format. Defaults to `false` (plain text).
*   **Expected Output:** A structured representation of the music library, typically in JSON format.
*   **Example Call (pseudocode):**
    ```
    call_tool("get_library_tree", {"path": "/path/to/music", "json_output": true})
    ```

### 3. `read_file_metadata`

*   **Description:** Reads and extracts all available metadata from a single music file.
*   **Parameters:**
    *   `file_path` (string, **required**): The absolute path to the music file.
*   **Expected Output:** A JSON object containing the detailed metadata of the specified music file.
*   **Example Call (pseudocode):**
    ```
    call_tool("read_file_metadata", {"file_path": "/path/to/music/Artist/Album/Track.flac"})
    ```

### 4. `normalize_titles`

*   **Description:** Normalizes track titles and other textual metadata to a consistent title case. Can perform a dry run to preview changes.
*   **Parameters:**
    *   `path` (string, **required**): The absolute path to the music file or directory to normalize.
    *   `dry_run` (boolean, optional): If `true`, changes will only be previewed and not applied. Defaults to `false`.
*   **Expected Output:** A report detailing the proposed or applied normalization changes.
*   **Example Call (pseudocode):**
    ```
    call_tool("normalize_titles", {"path": "/path/to/music", "dry_run": true})
    ```

### 5. `emit_library_metadata`

*   **Description:** Emits comprehensive metadata for the entire library or a specified path in a structured, AI-consumable format.
*   **Parameters:**
    *   `path` (string, **required**): The absolute path to the music library or a specific file.
    *   `json_output` (boolean, optional): If `true`, output will be in JSON format. Defaults to `false` (plain text).
*   **Expected Output:** Detailed metadata for the music library or specified file, typically in JSON.
*   **Example Call (pseudocode):**
    ```
    call_tool("emit_library_metadata", {"path": "/path/to/music", "json_output": true})
    ```

### 6. `validate_library`

*   **Description:** Validates the music library for common issues, inconsistencies, and metadata quality.
*   **Parameters:**
    *   `path` (string, **required**): The absolute path to the music library or a specific file.
    *   `json_output` (boolean, optional): If `true`, output will be in JSON format. Defaults to `false` (plain text).
*   **Expected Output:** A report of validation findings, including errors and warnings, either as plain text or JSON.
*   **Example Call (pseudocode):**
    ```
    call_tool("validate_library", {"path": "/path/to/music", "json_output": true})
    ```

### 7. `find_duplicates`

*   **Description:** Identifies duplicate audio files within a specified path using checksums.
*   **Parameters:**
    *   `path` (string, **required**): The absolute path to the directory to scan for duplicates.
    *   `json_output` (boolean, optional): If `true`, output will be in JSON format. Defaults to `false` (plain text).
*   **Expected Output:** A list of identified duplicate files, either as plain text or JSON.
*   **Example Call (pseudocode):**
    ```
    call_tool("find_duplicates", {"path": "/path/to/music", "json_output": true})
    ```

### 8. `cue_file`

*   **Description:** A multi-functional tool for generating, parsing, or validating CUE sheet files (`.cue`).
*   **Parameters:**
    *   `path` (string, **required**): The absolute path to the CUE file (for `parse`/`validate`) or the directory containing music files (for `generate`).
    *   `operation` (string, **required**): The specific CUE operation to perform. Must be one of:
        *   `"generate"`: Creates a new CUE sheet.
        *   `"parse"`: Reads and interprets an existing CUE sheet.
        *   `"validate"`: Checks a CUE sheet for consistency with its associated audio files.
    *   `output` (string, optional): Only for `operation: "generate"`. The path where the generated CUE file should be saved.
    *   `dry_run` (boolean, optional): Only for `operation: "generate"`. If `true`, the CUE file content will be displayed but not saved. Defaults to `false`.
    *   `force` (boolean, optional): Only for `operation: "generate"`. If `true`, an existing CUE file at the `output` path will be overwritten. Defaults to `false`.
    *   `audio_dir` (string, optional): Only for `operation: "validate"`. The directory containing the audio files referenced in the CUE sheet. If not provided, the parent directory of the `path` is used.
    *   `json_output` (boolean, optional): For `parse` and `validate` operations. If `true`, output will be in JSON format. Defaults to `false` (plain text).
*   **Expected Output:**
    *   For `generate` (dry_run): Displays the CUE file content.
    *   For `generate` (apply): Confirmation of file creation.
    *   For `parse`: The parsed CUE file structure, either as plain text or JSON.
    *   For `validate`: A validation report, either as plain text or JSON.
*   **Example Call (pseudocode):**
    ```
    // Generate a CUE file (dry run)
    call_tool("cue_file", {"path": "/path/to/album_directory", "operation": "generate", "dry_run": true})

    // Parse a CUE file (JSON output)
    call_tool("cue_file", {"path": "/path/to/album.cue", "operation": "parse", "json_output": true})

    // Validate a CUE file with custom audio directory
    call_tool("cue_file", {"path": "/path/to/album.cue", "operation": "validate", "audio_dir": "/path/to/audio_files", "json_output": true})
    ```
