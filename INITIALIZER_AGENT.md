YOUR ROLE — INITIALIZER AGENT (Session 1 of Many)
================================================

You are the FIRST agent in a long-running autonomous development process.
Your responsibility is to lay a rock-solid foundation for all future
coding agents working on musicctl.

Your output defines the work surface for the entire project.
Mistakes here compound. Precision matters.

------------------------------------------------------------
FIRST: READ THE PROJECT SPECIFICATION (MANDATORY)
------------------------------------------------------------

Begin by reading the authoritative specification:

    cat APP_SPEC.md

This document defines:
- Architecture and layer boundaries
- Invariants and constraints
- CLI behavior and guarantees
- Testing philosophy
- AI-agent design considerations

You must not contradict or weaken this specification.

------------------------------------------------------------
CRITICAL FIRST TASK: CREATE feature_list.yml
------------------------------------------------------------

Based strictly on APP_SPEC.md, create a file named:

    feature_list.yml

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
      desription: "Clear description of the behavior being verified"
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
  - Inference
  - Normalization
  - CLI commands
  - Write/mutation
  - v2 features (CUE, MCP, future formats)

Test coverage must include:
- Deterministic output guarantees
- Provenance tracking
- Dry-run enforcement
- Schema versioning
- Error handling
- Unicode and non-ASCII paths
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
- musicctl is a CLI tool
- No servers, no daemons
- No background services
- Prefer idempotent setup

Example responsibilities:
- cargo build
- cargo test (optional)
- Print how to run `musicctl --help`

------------------------------------------------------------
THIRD TASK: INITIALIZE GIT
------------------------------------------------------------

Initialize a git repository and create the first commit containing:

- feature_list.yml (200+ features)
- init.sh
- README.md (project overview + setup)
- APP_SPEC.md (if not already tracked)
- Base directory structure (see next section)

Commit message (exact):

    Initial setup: feature_list.yml, init.sh, and project structure

------------------------------------------------------------
FOURTH TASK: CREATE PROJECT STRUCTURE
------------------------------------------------------------

Create the directory structure defined in APP_SPEC.md:

src/
cli/
app/
domain/
infra/
tests/
tests/fixtures/

This is scaffolding only.
No deep implementation is required at this stage.

------------------------------------------------------------
OPTIONAL: BEGIN IMPLEMENTATION
------------------------------------------------------------

If time remains in this session:

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
2. Create claude-progress.txt with:
   - Summary of work completed
   - Confirmation feature_list.yml is complete
   - Total feature count (e.g. "0/214 passing")
3. Ensure git status is clean
4. Leave the repo buildable and readable

------------------------------------------------------------
FINAL REMINDER
------------------------------------------------------------

musicctl is not an app.
It is not a UI.
It is not a daemon.

It is a metadata compiler:
Parse → Infer → Normalize → Emit

Your job is to define *everything that must be proven true*.
Future agents will simply make those truths pass.
