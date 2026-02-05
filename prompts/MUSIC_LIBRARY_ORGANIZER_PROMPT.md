# Guide for AI Agents: Organizing Music Libraries with `musicctl` (MCP)

This document provides instructions and a recommended workflow for AI agents to effectively use the `musicctl` CLI tool, exposed via the Model Context Protocol (MCP) server, to organize and manage local music libraries.

## Goal

The primary goal is to empower AI agents to perform comprehensive music library organization tasks, including scanning, metadata management, validation, and duplicate detection, leveraging `musicctl`'s specialized MCP tools.

## General Principles for AI Agents

*   **Prioritize Structured Output:** Always prefer `json_output: true` when available in a tool's parameters. This ensures machine-readable and parsable results for further processing.
*   **Safety First (Dry Runs):** For any operation that modifies files (e.g., `normalize_titles`, `cue_file generate`), always perform a `dry_run: true` first to preview changes before applying them with `dry_run: false`.
*   **Iterative Refinement:** Music library organization can be complex. Approach tasks iteratively, using validation and tree views to verify changes.
*   **Error Handling:** Be prepared to handle `McpError` responses from tool calls and interpret the error messages for debugging or reporting.

---

## Recommended Workflow for Music Library Organization

Here's a step-by-step approach an AI agent can follow to organize a music library:

### Step 1: Initial Scan and Overview

Start by scanning the target directory to discover all music files and then get a hierarchical view of the current library structure.

*   **Action 1.1: Scan Directory for Tracks**
    *   **Tool:** `scan_directory`
    *   **Purpose:** Discover all music files within a specified path.
    *   **Parameters:**
        *   `path`: `/path/to/music/library` (the root directory of the library)
        *   `json_output`: `true`
    *   **Expected Output:** A JSON array of discovered tracks, each with basic path information.

*   **Action 1.2: Get Library Tree View**
    *   **Tool:** `get_library_tree`
    *   **Purpose:** Understand the inferred Artist -> Album -> Track hierarchy.
    *   **Parameters:**
        *   `path`: `/path/to/music/library`
        *   `json_output`: `true`
    *   **Expected Output:** A JSON object representing the hierarchical structure of the library.

### Step 2: Validate and Identify Issues

Check the library for common inconsistencies, missing metadata, and potential errors.

*   **Action 2.1: Validate Library Metadata**
    *   **Tool:** `validate_library`
    *   **Purpose:** Identify tracks with missing or inconsistent metadata, incorrect naming conventions, etc.
    *   **Parameters:**
        *   `path`: `/path/to/music/library`
        *   `json_output`: `true`
    *   **Expected Output:** A JSON report detailing validation results (errors, warnings).

### Step 3: Normalize and Clean Metadata

Apply normalization rules to standardize metadata fields like titles and genres.

*   **Action 3.1: Preview Title Normalization**
    *   **Tool:** `normalize_titles`
    *   **Purpose:** See what changes would be made to track titles without applying them.
    *   **Parameters:**
        *   `path`: `/path/to/music/library`
        *   `dry_run`: `true`
    *   **Expected Output:** A text report showing proposed title changes.

*   **Action 3.2: Apply Title Normalization (if preview is satisfactory)**
    *   **Tool:** `normalize_titles`
    *   **Purpose:** Apply the title casing and other normalization rules.
    *   **Parameters:**
        *   `path`: `/path/to/music/library`
        *   `dry_run`: `false`
    *   **Expected Output:** A text report confirming applied changes.

### Step 4: Detect and Manage Duplicates

Find and categorize duplicate audio files within the library.

*   **Action 4.1: Find Duplicate Tracks**
    *   **Tool:** `find_duplicates`
    *   **Purpose:** Locate identical audio files using checksums.
    *   **Parameters:**
        *   `path`: `/path/to/music/library`
        *   `json_output`: `true`
    *   **Expected Output:** A JSON array of duplicate groups, listing paths for each duplicate.

### Step 5: CUE Sheet Management

Handle CUE sheets for albums where a single audio file contains multiple tracks.

*   **Action 5.1: Parse a CUE File**
    *   **Tool:** `cue_file`
    *   **Operation:** `"parse"`
    *   **Purpose:** Extract structured information from an existing CUE sheet.
    *   **Parameters:**
        *   `path`: `/path/to/album/album.cue`
        *   `operation`: `"parse"`
        *   `json_output`: `true`
    *   **Expected Output:** A JSON object representing the parsed CUE file structure.

*   **Action 5.2: Validate a CUE File**
    *   **Tool:** `cue_file`
    *   **Operation:** `"validate"`
    *   **Purpose:** Check consistency between a CUE sheet and its audio files.
    *   **Parameters:**
        *   `path`: `/path/to/album/album.cue`
        *   `operation`: `"validate"`
        *   `audio_dir`: `/path/to/album/audio_files` (optional, defaults to CUE file's parent directory)
        *   `json_output`: `true`
    *   **Expected Output:** A JSON report indicating validation success/failure and any discrepancies.

*   **Action 5.3: Generate a CUE File (Dry Run)**
    *   **Tool:** `cue_file`
    *   **Operation:** `"generate"`
    *   **Purpose:** Preview the content of a new CUE sheet for a given album directory.
    *   **Parameters:**
        *   `path`: `/path/to/album_directory_with_tracks`
        *   `operation`: `"generate"`
        *   `dry_run`: `true`
    *   **Expected Output:** The generated CUE file content as a string.

*   **Action 5.4: Generate and Save a CUE File (Apply)**
    *   **Tool:** `cue_file`
    *   **Operation:** `"generate"`
    *   **Purpose:** Create and save a new CUE sheet.
    *   **Parameters:**
        *   `path`: `/path/to/album_directory_with_tracks`
        *   `operation`: `"generate"`
        *   `dry_run`: `false`
        *   `output`: `/path/to/album/new_album.cue` (optional, defaults to `path`/album_name.cue)
        *   `force`: `true` (optional, to overwrite existing)
    *   **Expected Output:** Confirmation message that the CUE file was written.

### Step 6: Detailed File Inspection

For specific files, agents can retrieve full metadata.

*   **Action 6.1: Read Individual File Metadata**
    *   **Tool:** `read_file_metadata`
    *   **Purpose:** Get all metadata for a single track.
    *   **Parameters:**
        *   `file_path`: `/path/to/music/Artist/Album/Track.flac`
    *   **Expected Output:** A JSON object with all available metadata for the file.

### Step 7: Comprehensive Metadata Extraction

Generate a complete metadata dump for the entire library.

*   **Action 7.1: Emit Library Metadata**
    *   **Tool:** `emit_library_metadata`
    *   **Purpose:** Obtain a full, structured JSON representation of the entire music library's metadata.
    *   **Parameters:**
        *   `path`: `/path/to/music/library`
        *   `json_output`: `true`
    *   **Expected Output:** A large JSON object containing all extracted library metadata.

---

## Example Scenario: Standardizing an Album

**Task:** Organize an album located at `/media/music/new_releases/ArtistName - AlbumTitle`.

1.  **Scan the album directory:**
    ```
    call_tool("scan_directory", {"path": "/media/music/new_releases/ArtistName - AlbumTitle", "json_output": true})
    ```
2.  **Get tree view of the album:**
    ```
    call_tool("get_library_tree", {"path": "/media/music/new_releases/ArtistName - AlbumTitle", "json_output": true})
    ```
3.  **Validate the album's metadata:**
    ```
    call_tool("validate_library", {"path": "/media/music/new_releases/ArtistName - AlbumTitle", "json_output": true})
    ```
    (Review output for errors/warnings and plan corrective actions if needed, e.g., using a manual `musicctl` command or noting for user intervention.)
4.  **Preview title normalization:**
    ```
    call_tool("normalize_titles", {"path": "/media/music/new_releases/ArtistName - AlbumTitle", "dry_run": true})
    ```
5.  **Apply title normalization:**
    ```
    call_tool("normalize_titles", {"path": "/media/music/new_releases/ArtistName - AlbumTitle", "dry_run": false})
    ```
6.  **Check for duplicates within the album (unlikely but good practice):**
    ```
    call_tool("find_duplicates", {"path": "/media/music/new_releases/ArtistName - AlbumTitle", "json_output": true})
    ```
7.  **If a CUE file is present, parse and validate it:**
    ```
    call_tool("cue_file", {"path": "/media/music/new_releases/ArtistName - AlbumTitle/album.cue", "operation": "parse", "json_output": true})
    call_tool("cue_file", {"path": "/media/music/new_releases/ArtistName - AlbumTitle/album.cue", "operation": "validate", "json_output": true})
    ```
    (If no CUE file, consider generating one if appropriate for the album's structure.)

This structured approach ensures a thorough and methodical organization process.