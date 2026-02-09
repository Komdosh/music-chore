# Useful Thoughts - music-chore Development

Love this problem. It's *exactly* the kind of thing Rust + a disciplined CLI + AI agents are good at. The project is currently in a production-ready state (v0.3.2) with a solid foundation for specialized AI-driven music management.

I'll assume:

* You care about **correctness and explainability**
* You want **machine-readable output first**, human-friendly second
* You want to avoid painting yourself into a format-specific corner
* You value **incremental improvement** over rewrites

---

# 1. Current State Understanding

## What's Implemented ✅
music-chore is a **precision metadata compiler** with:

**Core Features:**
- Multi-format support (FLAC, MP3, WAV, DSF, WavPack)
- Complete CLI with 10 commands: scan, tree, read, write, normalize, emit, validate, duplicates, cue, help
- MCP server with 8 tools and 18 expert prompts
- Default dry-run behavior for safety

**Architecture Quality:**
- Clean 4-layer separation (Presentation → Core Services → Domain → Adapters)
- Format-agnostic design enabling easy addition of new audio formats
- Deterministic output with stable JSON schemas
- Centralized logging and comprehensive error handling
- Unicode support throughout

**Testing:**
- 640+ tests (unit + integration + MCP)
- Extensive fixtures for multiple formats and edge cases
- High confidence in regression safety

**AI Agent Integration:**
- Advanced MCP server with expert prompts
- Structured output (JSON + AI-friendly text)
- Security path validation for multi-tenant environments

---

# 2. Module Structure (v0.3.2)

```
src/
├── bin/                           # Entry points (musicctl, musicctl-mcp)
├── core/                         # Core Business Logic
│   ├── domain/                   # Models & Schema
│   ├── services/                 # Operation implementations
│   └── logging.rs                # Centralized logging
├── adapters/                     # Format-specific I/O
│   └── audio_formats/            # FLAC, MP3, WAV, DSF, WavPack handlers
├── presentation/                 # Interface Layer
│   └── cli/                      # CLI parser and commands
└── mcp/                          # MCP Server
    ├── music_chore_server.rs     # Server implementation
    ├── prompts.rs                # Expert AI prompts
    └── config.rs                 # Security & Environment config
```

---

# 3. Key Data Models (ESTABLISHED)

### Core Metadata Model
Every field carries explicit provenance - this is **gold** for AI agents.

```rust
TrackMetadata {
    title: Option<MetadataValue<String>>,
    artist: Option<MetadataValue<String>>,
    album: Option<MetadataValue<String>>,
    // ... other fields
    format: String,              // "FLAC" | "MP3" | "WAV" | "DSF" | "WavPack"
    path: PathBuf,
}

MetadataValue<T> {
    value: T,
    source: MetadataSource,    // Embedded | FolderInferred | UserEdited
    confidence: f32,          // 0.0 - 1.0
}
```

---

# 4. Future Enhancement Opportunities

## High-Impact Areas

### 1. Additional Audio Formats
- **OGG/Vorbis**: Support for Vorbis tags.
- **M4A/AAC**: Support for MP4 containers and metadata.

### 2. Advanced AI Analysis
- **Acoustic Analysis**: Integration with audio analysis libraries for BPM or mood detection (out of current scope but high value).
- **Genre Mapping**: Comprehensive standard mapping for specialized genres (e.g., "Shoegaze" -> "Alternative Rock").

### 3. Library-Scale Operations
- **Bulk Metadata Updates**: Applying normalization or manual edits across thousands of files safely.
- **Incremental Cache**: Speeding up analysis for massive libraries (100k+ tracks).

---

# 5. Testing Strategy (640+ Tests)

### Guiding Principle
> If a public function exists, it must be testable without touching the real filesystem.

### Test Categories
- **Unit tests**: logic verification in each layer.
- **Integration tests**: end-to-end CLI and MCP workflows.
- **Fixture tests**: verification against real audio format files.
- **Security tests**: ensuring allowed path restrictions are enforced.

---

# 6. Quality Standards (MAINTAIN)

### Current Quality Bar
- **Deterministic output**: same input → same output, always.
- **Zero silent failures**: all errors are structured and visible.
- **Explicit provenance**: every field knows its source.
- **No hidden mutation**: operations are dry-runs by default.
- **Comprehensive testing**: every feature covered by multiple tests.
- **Security**: restricted filesystem access in MCP mode.

---

## Final Thoughts

This tool should feel like a **precision metadata compiler**:

```
Parse → Infer → Normalize → Emit
```

Future development should focus on **depth over breadth**. We have the formats and the tools; now we empower the AI agent to be a truly elite music curator.
