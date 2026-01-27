# musicctl

> A deterministic, AI-friendly **music metadata compiler**.

`musicctl` is a CLI-first Rust tool that **parses, infers, normalizes, and emits audio metadata**.
It is designed for correctness, explainability, and safe automation — not for organizing files or managing libraries.

Think of it as a compiler for music metadata:

**Parse → Infer → Normalize → Emit**

---

## Why musicctl?

Most music tools conflate *discovery*, *reasoning*, and *mutation*.
That makes them unsafe for automation and opaque for humans.

`musicctl` is built on different principles:

* 🧠 **Machine-first output** (stable JSON by default)
* 🔍 **Explicit provenance** for every metadata field
* 🔁 **Deterministic behavior** (same input → same output)
* 🧪 **Dry-run by default** (no accidental mutation)
* 🤖 **AI-agent friendly** (explainable inference, schema versioning)

This makes it suitable for:

* Metadata auditing
* Large-scale cleanup pipelines
* AI-assisted tagging
* Tooling and research workflows

---

## Non-Goals

`musicctl` deliberately does **not**:

* Organize or move your music files
* Maintain a database or cache
* Run as a daemon or service
* Guess silently or overwrite data implicitly

If you want a media player or library manager, this is not it.

---

## Architecture Overview

The project is organized into **four strictly isolated layers**:

```
CLI
 └─ commands, flags, output formatting

Application
 └─ workflows and use-cases (scan, inspect, update)

Domain
 └─ pure, format-agnostic logic
    inference, normalization, validation

Infrastructure
 └─ filesystem, audio formats (FLAC), CUE parsing
```

Hard rules:

* Lower layers never depend on higher layers
* Domain logic is pure and fully testable
* Format-specific code is isolated behind traits

See `app_spec.txt` for the full specification.

---

## Key Concepts

### Metadata with Provenance

Every metadata field is optional and carries its origin:

* Embedded (from the file)
* Inferred (from folders or filenames)
* User-edited

Each inferred value includes a confidence score.
Nothing is hidden.

---

### Read → Reason → Write

`musicctl` enforces a strict pipeline:

1. **Read** — discover files and extract raw metadata
2. **Reason** — infer and normalize metadata in memory
3. **Write** — explicitly persist changes (only with `--apply`)

Discovery commands never mutate data.

---

## CLI Overview (v1)

```bash
musicctl scan <path>
```

* Recursively discovers supported audio files
* Outputs deterministic JSON
* No mutation

```bash
musicctl tree <path>
```

* Human-readable hierarchy view
* Optional `--json` for machine output

```bash
musicctl read <file.flac>
```

* Shows embedded, inferred, and normalized metadata
* JSON by default, `--pretty` for humans

```bash
musicctl write <file.flac> --set key=value [...]
```

* Explicit metadata mutation
* Dry-run by default
* Requires `--apply` to persist

---

## Project Structure

```
src/
├── cli/        # Argument parsing and output formatting
├── app/        # Use-cases and workflows
├── domain/     # Pure business logic
├── infra/      # Filesystem and audio formats
└── tests/
    └── fixtures/
```

This structure is enforced by convention and review.

---

## Getting Started (Development)

### Prerequisites

* Rust (stable)
* cargo

### Setup

```bash
chmod +x init.sh
./init.sh
```

This will:

* Verify your Rust toolchain
* Build the project
* Print next steps

---

## Testing Philosophy

> If a public function exists, it must be testable without touching the real filesystem.

Test types:

* Unit tests (domain logic)
* Fixture-based tests (fake music trees)
* Golden tests (CLI output snapshots)
* Property tests (idempotency)

All tests favor determinism and explainability over convenience.

---

## Working With AI Agents

This project is designed for autonomous agents.

Key files:

* `CLAUDE.md` — **agent operating rules** (authoritative)
* `feature_list.json` — exhaustive end-to-end test plan
* `claude-progress.txt` — continuity between sessions

Agents may ONLY mark features as passing.
They must never edit or remove features.

---

## Roadmap

* v1: FLAC support, scanning, inference, normalization
* v2: CUE parsing and generation
* v2+: Additional audio formats (MP3, WAV, DSF)
* MCP-friendly output envelopes

See `app_spec.txt` and `feature_list.json` for details.

---

## Philosophy

`musicctl` should feel less like a media organizer and more like a compiler.

It does not guess quietly.
It does not mutate implicitly.
It explains its reasoning.

If the output changes, there should be a reason — and you should be able to see it.

---

## License

TBD

---

*Parse. Infer. Normalize. Emit.*
