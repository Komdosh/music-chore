# Useful Thoughts - music-chore Development

Love this problem. It's *exactly* the kind of thing Rust + a disciplined CLI + AI agents are good at. The project is already in an excellent state (v0.1.9) with a solid foundation for future enhancements.

I'll assume:

* You care about **correctness and explainability**
* You want **machine-readable output first**, human-friendly second
* You want to avoid painting yourself into a format-specific corner
* You value **incremental improvement** over rewrites

---

# 1. Current State Understanding

## What's Implemented ✅
music-chore is a **production-ready CLI tool** with:

**Core Features:**
- Multi-format support (FLAC + MP3) via format-agnostic AudioFile trait
- Complete CLI with 9 commands: scan, tree, read, write, normalize, emit, validate, duplicates, help, version
- MCP server with 6 tools for AI agent integration
- Comprehensive metadata operations with provenance tracking

**Architecture Quality:**
- Clean 4-layer separation (CLI → Services → Domain → Infrastructure)
- Format-agnostic design enabling easy addition of new audio formats
- Deterministic output with stable JSON schemas
- Comprehensive error handling and validation
- Unicode support throughout

**Testing:**
- 67+ tests (unit + integration + MCP)
- Test fixtures for multiple scenarios
- Golden/snapshot testing for CLI output
- Property-based testing for critical functions

**AI Agent Integration:**
- Complete MCP server implementation
- Structured output (JSON + AI-friendly text)
- Tool-based design for agent reasoning
- Comprehensive MCP test coverage

## Current Architecture (IMPLEMENTED)

```
┌────────────────────────────┐
│           CLI              │  ← Human & AI entrypoint
│  (commands, flags, output) │
│  musicctl + musicctl-mcp   │
└────────────┬───────────────┘
             │
┌────────────▼───────────────┐
│        Services            │  ← Orchestration & business logic
│  (scan, library, tree)   │
│  (validate, normalize)    │
└────────────┬───────────────┘
             │
┌────────────▼───────────────┐
│          Domain            │  ← Format-agnostic logic
│  (models, traits)        │
│  (TrackMetadata, Library) │
└────────────┬───────────────┘
             │
┌────────────▼───────────────┐
│     Infrastructure        │  ← Format-specific I/O
│  (FLAC, MP3, filesystem)│
│  (AudioFile trait)       │
└─────────────────────────────┘
```

### Key Architectural Principles (ESTABLISHED)

* **Format isolation**: FLAC and MP3 support behind AudioFile trait; new formats plug in cleanly.
* **Read → reason → write** pipeline: never mutate during discovery.
* **Deterministic output**: same input tree → same output ordering.
* **Explicit provenance**: inferred metadata marked with source and confidence.
* **Stateless CLI**: no daemon, cache unless explicitly added later.
* **AI-first design**: structured output, MCP integration, composable operations.

---

# 2. Current Module Structure (REALITY)

```
src/
├── bin/                           # Entry points
│   ├── musicctl.rs               # Main CLI (9 commands)
│   └── musicctl-mcp.rs          # MCP server (6 tools)
├── cli/                          # CLI layer (thin)
│   ├── mod.rs
│   ├── commands.rs               # Command definitions
│   └── commands_processor.rs     # Command routing
├── domain/                       # Pure business logic
│   ├── mod.rs
│   ├── models.rs                 # Core data models
│   └── traits.rs                # AudioFile trait (format-agnostic)
├── services/                     # Business services
│   ├── mod.rs
│   ├── scanner.rs                # Format-agnostic scanning
│   ├── library.rs               # Library hierarchy
│   ├── format_tree.rs           # Tree formatting
│   ├── duplicates.rs            # SHA256 duplicate detection
│   ├── inference.rs             # Path-based metadata inference
│   ├── normalization.rs        # Text normalization
│   ├── validation.rs           # Metadata validation
│   └── formats/               # Audio format implementations
│       ├── mod.rs              # Format registry
│       ├── flac.rs             # FLAC handler (lofty)
│       └── mp3.rs              # MP3 handler (ID3v2, lofty)
└── mcp/                         # MCP server
    ├── mod.rs
    ├── music_chore_server.rs   # Main MCP implementation
    └── params.rs              # MCP parameter definitions

tests/
├── fixtures/                     # Test data
│   ├── flac/                  # FLAC test files
│   ├── mp3/                   # MP3 test files
│   ├── unicode/                # Unicode path tests
│   ├── normalization/          # Title case tests
│   └── ...
└── *.rs                        # 67+ test files
```

---

# 3. Key Data Models (IMPLEMENTED)

### Core Metadata Model

```rust
TrackMetadata {
    // All fields carry provenance and confidence
    title: Option<MetadataValue<String>>,
    artist: Option<MetadataValue<String>>,
    album: Option<MetadataValue<String>>,
    album_artist: Option<MetadataValue<String>>,
    track_number: Option<MetadataValue<u32>>,
    disc_number: Option<MetadataValue<u32>>,
    year: Option<MetadataValue<u32>>,
    genre: Option<MetadataValue<String>>,
    duration: Option<MetadataValue<f64>>,
    format: String,              // "flac" | "mp3"
    path: PathBuf,
}

MetadataValue<T> {
    value: T,
    source: MetadataSource,    // Embedded | FolderInferred | UserEdited
    confidence: f32,          // 0.0 - 1.0
}
```

Every field carries explicit provenance - this is **gold** for AI agents.

### Library Representation

```rust
Library {
    artists: Vec<ArtistNode>,
    total_artists: usize,
    total_albums: usize,
    total_tracks: usize,
}

ArtistNode { name: String, albums: Vec<AlbumNode> }
AlbumNode { title: String, year: Option<u32>, tracks: Vec<TrackNode> }
TrackNode { file_path: PathBuf, metadata: TrackMetadata }
```

This representation is **read-only** until explicit edit operations.

### Audio Format Abstraction (PROVEN)

```rust
trait AudioFile: Send + Sync {
    fn can_handle(&self, path: &Path) -> bool;
    fn supported_extensions(&self) -> Vec<&'static str>;
    fn read_metadata(&self, path: &Path) -> Result<Track, AudioFileError>;
    fn write_metadata(&self, path: &Path, metadata: &TrackMetadata) -> Result<(), AudioFileError>;
    fn read_basic_info(&self, path: &Path) -> Result<TrackMetadata, AudioFileError>;
}
```

FLAC and MP3 implementations exist and are tested. This pattern works for WAV, DSF, etc.

---

# 4. CLI Commands (CURRENT STATE)

### Implemented Commands ✅

```bash
# Core discovery
musicctl scan <path>              # Find music files (FLAC+MP3)
musicctl tree <path>              # Human-friendly view with format indicators
musicctl read <file>              # Metadata extraction (FLAC+MP3)

# Metadata operations
musicctl write <file> --set "key=value" --apply  # Update (dry-run by default)
musicctl normalize <path>          # Title case normalization
musicctl validate <path>          # Metadata completeness check

# Advanced features
musicctl emit <path>             # Structured export (JSON/AI-text)
musicctl duplicates <path>        # SHA256 duplicate detection

# Misc
musicctl --version               # v0.1.9
musicctl help <command>          # Command help
```

### Global Options
- `--json` - Structured output where applicable
- `--dry-run` - Preview changes (write command)
- `--apply` - Apply changes (write command)

---

# 5. Future Enhancement Opportunities

## High-Impact Areas

### 1. Additional Audio Formats
**Pattern is established** - just implement AudioFile trait:

```rust
impl AudioFile for WavHandler {
    fn can_handle(&self, path: &Path) -> bool {
        path.extension().map_or(false, |ext| ext == "wav")
    }
    // ... rest of trait
}
```

- **WAV**: Microsoft WAV format (using lofty)
- **DSF**: DSD Stream File (high-resolution audio)

### 2. CUE Sheet Support
Parse existing .cue files and generate new ones from metadata:

```bash
musicctl cue generate <album-dir>     # Generate from existing tracks
musicctl cue parse <file.cue>        # Parse and validate
```

### 3. Performance Optimizations
- Parallel scanning for large libraries
- Incremental scanning (cache metadata)
- Streaming output for large datasets

### 4. Advanced Metadata Operations
- Bulk metadata updates
- Genre normalization/standardization
- Automatic duplicate resolution suggestions

## Medium-Impact Areas

### 5. Enhanced MCP Capabilities
- Batch operations
- Real-time file watching
- Plugin system for custom operations

### 6. User Experience
- Interactive mode
- Progress bars for large operations
- Configuration file support

---

# 6. Testing Strategy (PROVEN)

### Current Test Coverage
- **67+ tests** across unit, integration, and MCP
- **Golden snapshot testing** for CLI output stability
- **Property-based testing** for critical functions
- **Cross-platform testing** via CI

### Guiding Principle
> If a public function exists, it must be testable without touching the real filesystem.

### Test Categories
- **Unit tests**: normalization, inference, parsing logic
- **Integration tests**: end-to-end CLI workflows
- **MCP tests**: JSON-RPC protocol compliance
- **Property tests**: idempotency, deterministic behavior
- **Edge case tests**: Unicode, malformed files, empty directories

### Test Fixtures Structure
```
tests/fixtures/
├── flac/           # FLAC format tests
├── mp3/            # MP3 format tests
├── unicode/         # Non-ASCII path tests
├── normalization/  # Title case tests
├── duplicates/     # Duplicate detection tests
└── ...
```

---

# 7. Design Patterns That Work (ESTABLISHED)

### 1. Format Extension Pattern
The AudioFile trait pattern is proven and works perfectly:

1. Create new handler implementing AudioFile
2. Add to format registry in `services/formats/mod.rs`
3. Add comprehensive tests
4. Everything else works automatically

### 2. Command Addition Pattern
Adding new CLI commands follows established path:

1. Define command in `cli/commands.rs`
2. Implement handler in `cli/commands_processor.rs`
3. Add help text and usage examples
4. Add integration tests
5. Update documentation

### 3. MCP Tool Addition Pattern
For new MCP capabilities:

1. Define tool in `mcp/params.rs`
2. Implement handler in `mcp/music_chore_server.rs`
3. Add MCP integration tests
4. Update MCP documentation

---

# 8. Quality Standards (MAINTAIN)

### Current Quality Bar
- **Deterministic output**: same input → same output, always
- **Zero silent failures**: all errors are structured and visible
- **Explicit metadata provenance**: every field knows its source
- **No hidden mutation**: read operations never modify files
- **Comprehensive testing**: every public function covered
- **Multi-format compatibility**: FLAC+MP3 work seamlessly
- **AI-friendly output**: structured, parseable, predictable

### Maintain These Standards
- **Schema stability**: JSON output changes are versioned
- **Unicode support**: all paths and metadata support non-ASCII
- **Error clarity**: structured, actionable error messages
- **Documentation**: every feature documented with examples
- **Performance**: reasonable speed for large libraries

---

# 9. Development Workflow (OPTIMIZED)

### For Incremental Features
1. **Understand current state** via `agent-progress.txt` and git log
2. **Identify next logical feature** based on APP_SPEC.md
3. **Follow established patterns** (don't reinvent)
4. **Add comprehensive tests** before declaring done
5. **Update documentation** in the same PR
6. **Version bump** only for significant features

### Code Review Focus
- **Layer boundary violations** (domain depending on CLI)
- **Format-specific code** in wrong layer
- **Deterministic output** regressions
- **Test coverage gaps**
- **Documentation drift**

---

## Final Thoughts

### What Makes This Project Special
- **Production-ready**: Not a prototype, but a real tool
- **AI-native**: Built from ground up for agent interaction
- **Extensible**: Clean architecture for future growth
- **Correct**: Proven with comprehensive testing
- **User-friendly**: Both human and AI interfaces

### Guiding Philosophy
This tool should feel like a **precision metadata compiler**:

```
Parse → Infer → Normalize → Emit
```

### Future Development Guidance
1. **Use existing patterns** - don't reinvent what works
2. **Maintain quality bar** - keep the high standards
3. **Think multi-format** - ensure new features work with all formats
4. **Test thoroughly** - comprehensive tests are non-negotiable
5. **Document incrementally** - keep docs in sync with code

The foundation is solid. Future work is enhancement, not foundation-laying.