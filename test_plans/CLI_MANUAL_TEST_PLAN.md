# musicctl - Manual Test Plan (Current State)

## Overview
This plan verifies the current `musicctl` CLI behavior against implemented commands and flags.

## Preconditions
1. Run tests from repository root.
2. Build once before manual runs:
   - `cargo build`
3. Use existing fixture files under `tests/fixtures`.

## Test Scenarios

### 1. Scan Command
Test ID: `TC-SCAN-001`

Steps:
1. `cargo run --bin musicctl -- scan tests/fixtures/flac/simple`
2. `cargo run --bin musicctl -- scan tests/fixtures/flac/simple --json`
3. `cargo run --bin musicctl -- scan tests/fixtures/flac/simple --verbose`
4. `cargo run --bin musicctl -- scan tests/fixtures/flac/simple --skip-metadata`
5. `cargo run --bin musicctl -- scan tests/fixtures --max-depth 2 --exclude "*.DS_Store"`

Expected:
1. Non-JSON output lists files with formatted track information.
2. `--json` returns a JSON array of track objects.
3. `--verbose` emits progress/summary on stderr.
4. `--skip-metadata` still finds files but relies on filename/path info.
5. `--max-depth` and `--exclude` are honored.

### 2. Tree Command
Test ID: `TC-TREE-002`

Steps:
1. `cargo run --bin musicctl -- tree tests/fixtures/flac/nested`
2. `cargo run --bin musicctl -- tree tests/fixtures/flac/nested --json`

Expected:
1. Human output shows inferred hierarchy (Artist -> Album -> Track).
2. JSON output is structured library data including schema wrapper.

### 3. Read Command
Test ID: `TC-READ-003`

Steps:
1. `cargo run --bin musicctl -- read tests/fixtures/flac/simple/track1.flac`
2. `cargo run --bin musicctl -- read tests/fixtures/mp3/simple/track1.mp3`
3. `cargo run --bin musicctl -- read tests/fixtures/wav/simple/track1.wav`
4. `cargo run --bin musicctl -- read tests/fixtures/wavpack/silent/silent.wv`

Expected:
1. Output is JSON-formatted metadata with schema wrapper.
2. File path and metadata fields are present.
3. Unsupported/non-audio files return a clear error and non-zero exit.

### 4. Write Command
Test ID: `TC-WRITE-004`

Steps:
1. Copy fixture: `cp tests/fixtures/flac/simple/track1.flac /tmp/track1-write.flac`
2. Dry-run (implicit):
   - `cargo run --bin musicctl -- write /tmp/track1-write.flac --set title="Test Title" artist="Test Artist"`
3. Explicit dry-run:
   - `cargo run --bin musicctl -- write /tmp/track1-write.flac --set title="Test Title" artist="Test Artist" --dry-run`
4. Apply:
   - `cargo run --bin musicctl -- write /tmp/track1-write.flac --set title="Test Title" artist="Test Artist" --apply`
5. Verify:
   - `cargo run --bin musicctl -- read /tmp/track1-write.flac`

Expected:
1. Without `--apply`, command behaves as dry-run.
2. `--apply` writes metadata after confirmation in interactive mode.
3. Updated values appear in read output.
4. Using both `--apply` and `--dry-run` errors.

### 5. Normalize Command
Test ID: `TC-NORMALIZE-005`

Steps:
1. `cargo run --bin musicctl -- normalize tests/fixtures/normalization`
2. `cargo run --bin musicctl -- normalize tests/fixtures/normalization --json`

Expected:
1. Command returns normalization reports (title/genre/artist/album/year sections).
2. JSON output matches combined report schema.
3. No `--apply` flag exists in current CLI; this command currently reports normalization outcomes.

### 6. Emit Command
Test ID: `TC-EMIT-006`

Steps:
1. `cargo run --bin musicctl -- emit tests/fixtures/flac/nested`
2. `cargo run --bin musicctl -- emit tests/fixtures/flac/nested --json`

Expected:
1. Human output is readable summary/tree format.
2. JSON output contains structured library metadata.

### 7. Validate Command
Test ID: `TC-VALIDATE-007`

Steps:
1. `cargo run --bin musicctl -- validate tests/fixtures/flac/nested`
2. `cargo run --bin musicctl -- validate tests/fixtures/flac/nested --json`

Expected:
1. Non-JSON output summarizes issues.
2. JSON output contains structured validation report.

### 8. Duplicates Command
Test ID: `TC-DUPLICATES-008`

Steps:
1. `cargo run --bin musicctl -- duplicates tests/fixtures/duplicates`
2. `cargo run --bin musicctl -- duplicates tests/fixtures/duplicates --json`
3. `cargo run --bin musicctl -- duplicates tests/fixtures/duplicates --verbose`
4. `cargo run --bin musicctl -- duplicates tests/fixtures/duplicates --parallel 2`

Expected:
1. Duplicate groups are detected for identical content.
2. JSON output includes checksum-based grouping.
3. Verbose mode adds diagnostic detail.
4. `--parallel` runs successfully with selected worker count.

### 9. CUE Command
Test ID: `TC-CUE-009`

Steps:
1. Generate dry-run:
   - `cargo run --bin musicctl -- cue --generate tests/fixtures/flac/nested/The\ Beatles/Abbey\ Road --dry-run`
2. Parse fixture:
   - `cargo run --bin musicctl -- cue --parse tests/fixtures/cue/album.cue`
3. Parse JSON:
   - `cargo run --bin musicctl -- cue --parse tests/fixtures/cue/album.cue --json`
4. Validate fixture:
   - `cargo run --bin musicctl -- cue --validate tests/fixtures/cue/album.cue`

Expected:
1. Generate dry-run previews without writing.
2. Parse outputs CUE structure.
3. Parse `--json` returns structured output.
4. Validate reports consistency/inconsistency clearly.

### 10. Help and Version
Test ID: `TC-HELP-010`

Steps:
1. `cargo run --bin musicctl -- --help`
2. `cargo run --bin musicctl -- --version`
3. `cargo run --bin musicctl -- scan --help`
4. `cargo run --bin musicctl -- cue --help`

Expected:
1. Top-level help lists current commands and options.
2. Version prints current version.
3. Subcommand help reflects implemented flags.

### 11. Error Handling
Test ID: `TC-ERROR-011`

Steps:
1. `cargo run --bin musicctl -- scan /nonexistent/path`
2. `cargo run --bin musicctl -- read tests/fixtures/cue/album.cue`
3. `cargo run --bin musicctl -- write tests/fixtures/flac/simple/track1.flac --set invalid_field=value --dry-run`

Expected:
1. Clear error messages.
2. Non-zero exit code on errors.
3. No panic/crash.

### 12. Unicode and Path Edge Cases
Test ID: `TC-EDGE-012`

Steps:
1. `cargo run --bin musicctl -- scan "tests/fixtures/unicode"`
2. `cargo run --bin musicctl -- tree "tests/fixtures/unicode"`
3. `cargo run --bin musicctl -- read "tests/fixtures/unicode/José González/album/track.flac"`

Expected:
1. Unicode paths are processed correctly.
2. Output is stable and readable.
3. No encoding-related failures.

## Coverage Checklist
- [ ] `scan` core + advanced flags (`--max-depth`, `--exclude`, `--skip-metadata`)
- [ ] `tree` plain + JSON
- [ ] `read` across available fixture formats
- [ ] `write` default dry-run + apply + conflict flag validation
- [ ] `normalize` plain + JSON report behavior
- [ ] `emit` plain + JSON
- [ ] `validate` plain + JSON
- [ ] `duplicates` plain + JSON + verbose + parallel
- [ ] `cue` generate/parse/validate paths and JSON where supported
- [ ] help/version checks
- [ ] error-path behavior
- [ ] unicode/path edge cases

## Notes
1. Current normalize help text mentions `--genres`, but that flag is not currently implemented in CLI options.
2. Keep this plan synchronized with `src/presentation/cli/commands.rs` and `src/presentation/cli/commands_processor.rs` when CLI flags change.
