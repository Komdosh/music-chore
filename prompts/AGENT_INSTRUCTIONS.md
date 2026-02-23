# Agent Instructions (music-chore)

Use this file for session workflow. Use `prompts/APP_SPEC.md` for product/architecture truth.

## Session Workflow

1. **Orient**
   - `pwd`
   - `ls -la`
   - `cat prompts/APP_SPEC.md`
   - `cat agent-progress.txt || true`
   - `git log --oneline -20`

2. **Baseline Verification (before feature work)**
   - Run targeted checks for core behavior:
     - `cargo run --bin musicctl -- scan tests/fixtures/flac/simple --json`
     - `cargo run --bin musicctl -- tree tests/fixtures/flac/nested --json`
     - `cargo run --bin musicctl -- read tests/fixtures/flac/simple/track1.flac`
   - If regressions appear, fix them first.

3. **Implement one focused change**
   - Keep scope tight and testable.
   - Respect architecture boundaries from APP_SPEC.

4. **Verify end-to-end**
   - Run relevant CLI commands with fixtures.
   - Run targeted tests first, then broader tests as needed (`cargo test`).

5. **Update tracking/docs**
   - Update `agent-progress.txt` when substantive work is done.
   - Update docs if behavior changed.

6. **Finish cleanly**
   - Ensure changes are intentional (`git status`).
   - Commit atomic, reviewable changes.

## Quality Bar

- Deterministic output.
- No silent failures.
- No hidden mutation in read-only flows.
- Public API changes must be tested.
- Prefer clarity over cleverness.
