YOUR ROLE — INITIALIZER AGENT (Session 1 of Many)
===============================================

You are the FIRST agent in a long-running autonomous development process.
Your responsibility is to lay a rock-solid foundation for all future
coding agents working on music-chore.

**IMPORTANT**: music-chore is ALREADY IMPLEMENTED up to v0.1.9
This document is for historical context and project reset scenarios.

Your output defines the work surface for entire project.
Mistakes here compound. Precision matters.

------------------------------------------------------------
CURRENT PROJECT STATE (as of v0.1.9)
------------------------------------------------------------

music-chore is a PRODUCTION-READY CLI tool with:

✅ **Complete Implementation**:
- Multi-format support (FLAC + MP3)
- Full CLI with 9 commands
- MCP server with 6 tools
- 67+ comprehensive tests
- Unicode support
- Duplicate detection
- Metadata validation
- Title normalization

✅ **Architecture**:
- Clean 4-layer architecture (CLI → Services → Domain → Infrastructure)
- Format-agnostic AudioFile trait
- Comprehensive error handling
- Deterministic output
- AI-friendly structured output

✅ **Build System**:
- Cargo-based Rust project
- Two binaries: musicctl, musicctl-mcp
- Comprehensive test suite
- Integration with GitHub Actions

⚠️ **This initialization document is mainly for project reset scenarios**

------------------------------------------------------------
FIRST: READ THE PROJECT SPECIFICATION (MANDATORY)
------------------------------------------------------------

Begin by reading the authoritative specification:

    cat prompts/APP_SPEC.md

This document defines:
- Current implementation status
- Architecture and layer boundaries
- Invariants and constraints
- CLI behavior and guarantees
- Testing philosophy
- AI-agent design considerations
- Future roadmap

You must not contradict or weaken this specification.

------------------------------------------------------------
UNDERSTANDING THE CURRENT STATE
------------------------------------------------------------

If this is a greenfield project (no existing code):
- Follow INITIALIZER workflow below
- Create complete project structure

If code already exists:
- Assess current implementation vs APP_SPEC.md
- Identify gaps and next steps
- Plan incremental improvements

------------------------------------------------------------
CRITICAL FIRST TASK: CREATE feature_list.yml
------------------------------------------------------------

**ONLY FOR GREENFIELD PROJECTS**

Based strictly on @prompts/APP_SPEC.md, create a file named:

    prompts/feature_list.yml

This file is the SINGLE SOURCE OF TRUTH for all work to be done.

It defines the complete set of end-to-end behaviors that musicctl
must support. Future agents may ONLY mark features as passing.
They must never remove, edit, reorder, or merge features.

------------------------------------------------------------
feature_list.yml FORMAT
------------------------------------------------------------

Each entry must follow this exact structure:

features:
    - category: "functional"
      description: "Clear description of the behavior being verified"
      steps:
        - "Step 1: Prepare fixture directory"
        - "Step 2: Run musicctl scan <path>"
        - "Step 3: Verify deterministic JSON output"
      passes: false

------------------------------------------------------------
feature_list.yml REQUIREMENTS
------------------------------------------------------------

- Minimum: 200 features
- Categories:
  - "functional"
  - "style" (for CLI output formatting, readability, stability, UX)
- ALL features start with `"passes": false`
- Features must be ordered by priority:
  - Core filesystem scanning
  - Metadata reading
  - Format support (FLAC, MP3)
  - Inference
  - Normalization
  - CLI commands
  - Write/mutation
  - v2 features (MCP, future formats, CUE)

Test coverage must include:
- Deterministic output guarantees
- Provenance tracking
- Dry-run enforcement
- Schema versioning
- Error handling
- Unicode and non-ASCII paths
- Multi-format support
- Broken or malformed inputs
- Edge cases in folder inference
- CLI JSON vs human-readable modes

Step requirements:
- Mix short tests (2–5 steps) and long tests (10+ steps)
- At least 25 tests MUST have 10+ detailed steps
- Steps must describe *real CLI execution*, not internal calls

------------------------------------------------------------
CRITICAL WARNING
------------------------------------------------------------

IT IS CATASTROPHIC TO REMOVE OR EDIT FEATURES IN FUTURE SESSIONS.

Future agents may ONLY change:
    "passes": false → "passes": true

They must NEVER:
- Remove features
- Edit descriptions
- Modify steps
- Reorder features
- Consolidate tests

This guarantees no silent scope loss.

------------------------------------------------------------
SECOND TASK: CREATE init.sh
------------------------------------------------------------

Create a script named:

    init.sh

This script must allow any future agent to quickly get started.

Requirements:
1. Verify Rust toolchain availability (cargo, rustc)
2. Run any required setup steps
3. Build the project if possible
4. Print helpful next steps

Notes:
- music-chore is a CLI tool with MCP server
- No daemons or background services
- Two binaries: musicctl and musicctl-mcp
- Prefer idempotent setup

Example responsibilities:
- cargo build
- cargo test (optional)
- Print how to run `musicctl --help`
- Print how to test MCP server

------------------------------------------------------------
THIRD TASK: INITIALIZE GIT
------------------------------------------------------------

Initialize a git repository and create first commit containing:

- @prompts/feature_list.yml (200+ features) [GREENFIELD ONLY]
- init.sh [GREENFIELD ONLY]
- README.md (project overview + setup)
- @prompts/APP_SPEC.md (if not already tracked)
- Base directory structure (see below) [GREENFIELD ONLY]

Commit message (exact):

    Initial setup: feature_list.yml, init.sh, and project structure

------------------------------------------------------------
FOURTH TASK: CREATE PROJECT STRUCTURE
------------------------------------------------------------

**GREENFIELD PROJECTS ONLY**

Create the directory structure defined in @prompts/APP_SPEC.md:

src/
├── bin/
│   ├── musicctl.rs
│   └── musicctl-mcp.rs
├── cli/
├── domain/
├── services/
└── mcp/

tests/
├── fixtures/
│   ├── flac/
│   ├── mp3/
│   └── ...

This is scaffolding only.
Deep implementation comes from following agents.

------------------------------------------------------------
OPTIONAL: BEGIN IMPLEMENTATION
------------------------------------------------------------

If time remains in this session AND project is greenfield:

- Select the single highest-priority feature in feature_list.yml
- Implement it following the normal agent workflow
- Verify via real CLI execution and fixtures
- Commit before session end

This is optional.
Completeness and correctness of feature_list.yml take priority.

------------------------------------------------------------
ENDING THIS SESSION (MANDATORY)
------------------------------------------------------------

Before context fills up:

1. Commit all work
2. Update agent-progress.txt with:
   - Summary of work completed
   - Confirmation feature_list.yml is complete [GREENFIELD ONLY]
   - Total feature count (e.g. "0/214 passing")
   - Current assessment of existing implementation [IF CODE EXISTS]
3. Ensure git status is clean
4. Leave the repo buildable and readable

------------------------------------------------------------
FINAL REMINDER
------------------------------------------------------------

music-chore is not an app.
It is not a UI.
It is not a daemon.

It is a metadata compiler with AI agent integration:
Parse → Infer → Normalize → Emit

**IF IMPLEMENTATION EXISTS**: Focus on incremental improvements, gap analysis, and future planning.

Your job is to define *everything that must be proven true*.
Future agents will simply make those truths pass.

------------------------------------------------------------
ASSESSMENT GUIDANCE (FOR EXISTING CODE)
------------------------------------------------------------

If you find existing implementation:
1. Compare against APP_SPEC.md requirements
2. Identify what's implemented vs what's missing
3. Assess code quality and test coverage
4. Recommend next priority features
5. Update agent-progress.txt with current state

The goal is continuous improvement, not rebuilding from scratch.