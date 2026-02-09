# Guide for AI Agents: Organizing Music Libraries with `musicctl` (MCP)

This document provides instructions and a recommended workflow for AI agents to effectively use the `musicctl` CLI tool, exposed via the Model Context Protocol (MCP) server, to organize and manage local music libraries.

## Goal

The primary goal is to empower AI agents to perform comprehensive music library organization tasks, including scanning, metadata management, validation, and duplicate detection, leveraging `musicctl`'s specialized MCP tools and expert prompts.

## General Principles for AI Agents

*   **Prioritize Structured Output:** Always prefer `json_output: true` when available in a tool's parameters. This ensures machine-readable results for further processing.
*   **Safety First (Dry Runs):** For any operation that modifies files (e.g., `normalize`, `cue_file generate`), always perform a preview first. `musicctl` write operations are dry-runs by default.
*   **Leverage Expert Prompts:** Use the 18 specialized prompts (e.g., `library-health-check`, `top-tracks-analysis`) to get pre-formatted analysis and action plans for complex tasks.
*   **Error Handling:** Be prepared to handle `McpError` responses and security restrictions if you attempt to access paths outside the configured allowed paths.

---

## Recommended Workflow for Music Library Organization

### Step 1: Initial Assessment

Start by getting a comprehensive overview of the library's state.

*   **Action 1.1: Run Health Check**
    *   **Prompt:** `library-health-check`
    *   **Purpose:** Get a prioritized improvement plan and overall health score.
    
*   **Action 1.2: Get Library Tree**
    *   **Tool:** `get_library_tree`
    *   **Purpose:** Understand the current Artist -> Album -> Track structure.

### Step 2: Standardization

Apply automated cleanup to bring metadata into a consistent state.

*   **Action 2.1: Metadata Cleanup Guide**
    *   **Prompt:** `metadata-cleanup-guide`
    *   **Purpose:** Identify categorized issues and get specific commands to fix them.

*   **Action 2.2: Normalize Titles and Genres**
    *   **Tool:** `normalize`
    *   **Parameters:** `path`, `json_output: true`
    *   **Note:** Review changes before suggesting the user apply them with `musicctl normalize --apply`.

### Step 3: Optimization

Clean up redundant data and ensure structural integrity.

*   **Action 3.1: Duplicate Resolution**
    *   **Prompt:** `duplicate-resolution`
    *   **Purpose:** Find duplicates and get intelligent recommendations on which copies to keep based on quality.

*   **Action 3.2: Reorganization Plan**
    *   **Prompt:** `reorganization-plan`
    *   **Purpose:** Get a strategic plan to move files into a standard `Artist/Year - Album/Track` hierarchy.

### Step 4: CUE Sheet Management

Handle albums stored as single files or needing better track marking.

*   **Action 4.1: CUE Sheet Assistant**
    *   **Prompt:** `cue-sheet-assistant`
    *   **Purpose:** Find missing CUE files and validate existing ones.

---

## Available Expert Prompts (18)

Agents can call these prompts to perform deep-dives without manual tool chaining:

| Category | Prompts |
|----------|---------|
| **Analysis** | `top-tracks-analysis`, `genre-breakdown`, `decade-analysis`, `collection-story`, `artist-deep-dive` |
| **Discovery** | `instrument-to-learn`, `similar-artists-discovery`, `mood-playlist`, `hidden-gems`, `album-marathon`, `concert-setlist` |
| **Maintenance** | `library-health-check`, `metadata-cleanup-guide`, `duplicate-resolution`, `reorganization-plan`, `format-quality-audit`, `year-in-review`, `cue-sheet-assistant` |

---

## Example Scenario: Comprehensive Cleanup

**Task:** "The music folder at `/media/music` is a mess. Fix it."

1.  **Analyze**: Call `library-health-check`.
2.  **Report**: Tell the user their health score and the top 5 urgent issues.
3.  **Deduplicate**: Call `duplicate-resolution` and suggest which files to delete.
4.  **Standardize**: Call `normalize` (dry run) and show the user the improvements.
5.  **Plan**: Call `reorganization-plan` to show how the folders *could* look.

This workflow ensures the agent acts as an expert consultant rather than just a basic file scanner.
