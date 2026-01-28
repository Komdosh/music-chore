# Claude Code Agent Instructions

## Project Overview

You are assisting with the design and implementation of a **Rust-based CLI tool for macOS** that organizes and normalizes a local music library using **existing file metadata and directory structure only**.

This project is intentionally narrow in scope and optimized for **AI-agent usage** via MCP. The tool should do a small number of things extremely well.

**Do not write production code unless explicitly instructed.** Planning, architecture, and incremental design are prioritized.

---

## Problem Summary

The user has a large local music library with the following characteristics:

* Music is primarily organized by folders, not metadata
* Folder names implicitly encode metadata (artist, album, genre, format, sample rate)
* Artist folders may contain album subfolders
* Album folders contain multiple tracks
* Tracks may exist in formats such as `.flac`, `.mp3`, `.wav`, `.dsf`
* Many albums include `.cue` files with inconsistent formatting
* Genres are inconsistent or duplicated (e.g. "Alt Rock" vs "Alternative Rock")
* Media players display an ugly, fragmented library due to poor metadata

The goal is to **read, normalize, display, and edit metadata locally** — without using the internet.

---

## Core Goal

Design a **CLI-only Rust program (macOS only)** that:

* Recursively scans a working directory
* Discovers music files and albums
* Reads and writes metadata
* Reads and writes `.cue` files
* Exposes structured, machine-readable output suitable for AI agents
* Supports incremental updates when new folders are added

This tool will later be used by a local AI agent through MCP.

---

## Hard Constraints

You must respect the following constraints at all times:

* CLI only (no GUI)
* macOS only
* Rust language
* No internet access or external metadata lookup
* No playback functionality
* Modular architecture (future formats must be easy to add)
* Every public function must be covered by tests
* Prefer correctness and clarity over cleverness

---

## Supported File Formats

### v1 (Initial Scope)

* `.flac` only

### v2 (Future Scope)

* `.mp3`
* `.wav`
* `.dsf`

Architecture **must anticipate future formats**, even if only `.flac` is implemented initially.

---

## Versioned Feature Scope

### v1 — Foundation

You should plan and design for:

* Recursive directory scanning
* Artist → Album → Track hierarchy inference
* CLI command to display a tree structure:

  * Artist

    * Album

      * Tracks
* CLI command to read metadata from a `.flac` file
* CLI command to write/update metadata in a `.flac` file
* Internals designed to support multiple formats later

### v2 — AI Agent Integration

Planned but **not implemented in v1**:

* MCP awareness (agent-friendly command structure and output)
* Support additional file formats
* Generate a `.cue` file for an album folder using track metadata

---

## Explicit Non-Goals

Do **NOT**:

* Fetch metadata from the internet
* Use MusicBrainz, Discogs, or similar databases
* Guess artist/album info beyond local inference
* Implement media playback
* Implement a GUI

---

## Design Principles

Follow these principles strictly:

1. **Small, composable modules**
2. **Clear data models** for Artist, Album, Track, Metadata
3. **Format-agnostic interfaces** (traits) for audio files
4. **Deterministic behavior** — same input yields same output
5. **AI-friendly output** (structured, predictable, machine-readable when useful)
6. **Incremental processing** — new folders can be scanned without full rebuild

---

## Expected Planning Outputs

When asked to plan or design, produce:

1. High-level architecture diagrams (conceptual, not graphical)
2. Proposed module layout
3. Core data structures and traits
4. CLI command and flag design
5. Step-by-step implementation plan
6. Testing strategy
7. Design trade-offs and rationale

Avoid vague statements. Prefer concrete decisions.

---

## AI-Agent Awareness

Assume this tool will be called by another AI agent via MCP.

Therefore:

* CLI output should be easy to parse
* Commands should be composable and scriptable
* Errors should be structured and descriptive
* Avoid ambiguous or human-only output unless explicitly requested

---

## Working Style Instructions

* Ask **at most one** clarifying question if absolutely necessary
* Otherwise, make reasonable assumptions and state them
* Do not over-engineer
* Do not introduce features outside the defined scope
* Keep v1 minimal and rock-solid

---

## Your Roles:

- Role 1 Initializer Agent (first session only): check prompts/INITILIZER_AGENT.md
- Role 2 Coding Agent (all subsequent sessions): check prompts/AGENT_INSTRUCTIONS.md

---

## Summary

You are building a **precise, local, metadata-focused music library tool** that serves both humans and AI agents.

Design it like a Unix tool:

* Small
* Sharp
* Predictable
* Composable

When in doubt, choose simplicity.
