# Useful thoughts

Love this problem. It’s *exactly* the kind of thing Rust + a disciplined CLI + AI agents are good at. I’ll approach this like a real greenfield systems tool that you’ll live with for years, not a throwaway script.

I’ll assume:

* You care about **correctness and explainability**
* You want **machine-readable output first**, human-friendly second
* You want to avoid painting yourself into a format-specific corner

---

# 1. High-Level Architecture Overview

Think in **four layers**, with very hard boundaries:

```
┌────────────────────────────┐
│           CLI              │  ← Human & AI entrypoint
│  (commands, flags, output) │
└────────────┬───────────────┘
             │
┌────────────▼───────────────┐
│        Application         │  ← Use-cases / workflows
│  (scan, inspect, update)   │
└────────────┬───────────────┘
             │
┌────────────▼───────────────┐
│          Domain            │  ← Format-agnostic logic
│  (metadata, normalization) │
└────────────┬───────────────┘
             │
┌────────────▼───────────────┐
│      Infrastructure        │  ← Format-specific I/O
│  (FLAC, CUE, filesystem)   │
└────────────────────────────┘
```

### Core Architectural Principles

* **Format isolation**: FLAC support lives behind a trait; v2 formats plug in cleanly.
* **Read → reason → write** pipeline: never mutate during discovery.
* **Deterministic output**: same input tree → same output ordering.
* **Explicit uncertainty**: inferred metadata must be marked as inferred.
* **Stateless CLI**: no daemon, no cache unless explicitly added later.

---

# 2. Proposed Module Structure

```
src/
├── main.rs
├── cli/
│   ├── mod.rs
│   ├── commands/
│   │   ├── scan.rs
│   │   ├── tree.rs
│   │   ├── read.rs
│   │   ├── write.rs
│   │   └── cue.rs        (v2)
│   └── output.rs
│
├── app/
│   ├── mod.rs
│   ├── scanner.rs
│   ├── library.rs
│   ├── inspector.rs
│   ├── editor.rs
│   └── cue_generator.rs (v2)
│
├── domain/
│   ├── mod.rs
│   ├── models/
│   │   ├── artist.rs
│   │   ├── album.rs
│   │   ├── track.rs
│   │   ├── genre.rs
│   │   └── metadata.rs
│   │
│   ├── normalize/
│   │   ├── mod.rs
│   │   ├── genre.rs
│   │   ├── artist.rs
│   │   └── album.rs
│   │
│   ├── inference/
│   │   ├── mod.rs
│   │   └── folder.rs
│   └── errors.rs
│
├── infra/
│   ├── mod.rs
│   ├── fs.rs
│   ├── audio/
│   │   ├── mod.rs
│   │   ├── traits.rs
│   │   └── flac.rs
│   └── cue/
│       ├── mod.rs
│       ├── parser.rs
│       └── writer.rs
│
└── tests/
    ├── fixtures/
    ├── cli/
    ├── domain/
    └── infra/
```

This keeps:

* **CLI logic thin**
* **Domain logic pure and testable**
* **All unsafe / format-specific code isolated**

---

# 3. Key Data Models

### Core Metadata Model (Format-Agnostic)

```text
TrackMetadata
- title
- artist
- album
- album_artist
- track_number
- disc_number
- year
- genre
- duration
- format
- source (Embedded | Inferred | Mixed)
```

Every field should be:

* `Option<T>`
* Carry **provenance**

```text
MetadataValue<T>
- value: T
- source: Embedded | FolderInferred | UserEdited
- confidence: f32
```

This is *gold* for AI agents.

---

### Library Representation

```text
Library
- artists: Vec<ArtistNode>

ArtistNode
- name
- albums: Vec<AlbumNode>

AlbumNode
- title
- year
- tracks: Vec<TrackNode>
- path

TrackNode
- path
- metadata
```

This is **read-only** until an explicit edit operation.

---

### Audio Format Abstraction

```text
trait AudioFile {
  fn read_metadata(&self) -> Result<TrackMetadata>
  fn write_metadata(&self, TrackMetadata) -> Result<()>
  fn format(&self) -> AudioFormat
}
```

FLAC is just one implementation.

---

# 4. CLI Command Structure

### v1 Commands

```
musicctl scan <path>
```

* Discovers supported files
* Outputs machine-readable JSON
* No mutation

```
musicctl tree <path>
```

Human-friendly tree view:

```
Artist
  Album (Year)
    01 - Track.flac
```

Optional:

```
--json
```

---

```
musicctl read <file.flac>
```

Outputs:

* Raw embedded metadata
* Inferred metadata
* Normalized view

Default: JSON
Optional: `--pretty`

---

```
musicctl write <file.flac> --set key=value [...]
```

* Explicit mutation
* Dry-run by default
* `--apply` required to write

---

### v2 Commands

```
musicctl cue generate <album-dir>
```

```
musicctl scan --mcp
```

Outputs MCP-friendly envelopes.

---

# 5. Step-by-Step Implementation Plan

## Phase 0: Foundations

1. Project scaffolding
2. Error handling strategy (`thiserror`)
3. Deterministic filesystem traversal
4. Golden test fixtures

---

## Phase 1 (v1 Core)

### Step 1: Filesystem Scanner

* Recursive walk
* Filter `.flac`
* Stable ordering
* Path normalization

Tests:

* Mixed directories
* Symlinks
* Empty folders

---

### Step 2: FLAC Metadata I/O

* Read Vorbis comments
* Write metadata safely
* Preserve unrelated tags

Tests:

* Read/write roundtrip
* Partial updates
* Non-ASCII tags

---

### Step 3: Domain Models

* TrackMetadata
* Provenance tracking
* Validation rules

Tests:

* Metadata merge logic
* Source precedence

---

### Step 4: Folder-Based Inference

* Artist from parent dir
* Album from folder
* Track number from filename

Tests:

* Weird folder names
* Multi-disc folders

---

### Step 5: Normalization

* Genre canonicalization (local map)
* Whitespace / casing normalization

Tests:

* Genre alias collapse
* Idempotency

---

### Step 6: CLI Commands

* `scan`
* `tree`
* `read`
* `write`

Tests:

* Snapshot tests
* JSON schema stability

---

## Phase 2 (v2)

### Step 7: Audio Trait Expansion

* MP3, WAV, DSF stubs
* No CLI exposure yet

---

### Step 8: CUE Support

* Parse inconsistent cues
* Generate new cues from metadata

Tests:

* Broken cue files
* Mixed format albums

---

### Step 9: MCP Optimization

* Stable JSON envelopes
* Explicit tool metadata
* Versioned output schemas

---

# 6. Testing Strategy

### Guiding Rule

> If a public function exists, it must be testable without touching the real filesystem.

### Types of Tests

* **Unit tests**: normalization, inference, parsing
* **Fixture-based tests**: fake music trees
* **Golden tests**: CLI output snapshots
* **Property tests**: normalization idempotency

### Test Fixtures

```
tests/fixtures/
├── flac/
├── cue/
├── messy_tree/
└── unicode/
```

---

# 7. Design Decisions for AI-Agent Effectiveness

This is where you really win.

### 1. Machine-First Output

* JSON by default
* Stable field ordering
* Explicit provenance

### 2. Explainability

Every inferred field explains:

* Why it exists
* Where it came from
* How confident it is

### 3. Idempotency

Running the same command twice:

* Produces identical output
* Makes agents safer

### 4. Dry-Run Everywhere

AI agents can reason *before* mutating.

### 5. Schema Versioning

Every JSON output includes:

```
schema_version
tool_version
```

### 6. Narrow, Sharp Commands

No “do everything” command.
Small verbs → composable reasoning.

---

## Final Thought

This tool should feel less like a media organizer and more like a **metadata compiler**:

* Parse
* Infer
* Normalize
* Emit

If you want, next we can:

* Design the exact JSON schema
* Design the normalization ruleset
* Or walk through *one* command (e.g. `scan`) down to function signatures