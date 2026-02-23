pub(crate) fn listen_now_prompt(path: String, minutes: u32, mood: &str, novelty: &str) -> String {
    format!(
        r#"Help me decide what to listen to right now from my library at "{path}".
Available time: {minutes} minutes.
Mood/activity: "{mood}".
Novelty preference: "{novelty}".

Steps:
1. Use `scan_directory` with `json_output: true` to get all tracks.
2. Use `get_library_tree` with `json_output: true` to understand artist/album context.
3. Use `emit_library_metadata` with `json_output: true` to compare genres, years, and metadata quality.

Decision framework:
- Build three options: **Comfort Pick** (familiar), **Balanced Pick**, **Discovery Pick** (less obvious).
- Match option length to {minutes} minutes (allow +/- 10 minutes).
- Favor complete album stretches when possible.
- Use only tracks from this local library.

Output format:
- **Start Now**: one final choice with 2-3 reasons.
- **Runner-up Options**: two alternatives.
- **Queue**: 5-10 tracks in listening order.
- **Why This Works Today**: one concise paragraph tied to mood and time."#
    )
}

pub(crate) fn web_perfect_match_prompt(
    path: String,
    mood: Option<&str>,
    genre: Option<&str>,
    max_results: u32,
) -> String {
    let mood_clause = mood.unwrap_or("none");
    let genre_clause = genre.unwrap_or("none");
    format!(
        r#"Find web music recommendations that match my local library at "{path}" as closely as possible.
Mood filter: "{mood_clause}".
Genre filter: "{genre_clause}".
Max results: {max_results}.

Steps:
1. Use `scan_directory` with `json_output: true`.
2. Use `get_library_tree` with `json_output: true`.
3. Use `emit_library_metadata` with `json_output: true`.
4. Use web search/knowledge to find candidates outside my library.

Strict matching protocol:
- Derive a taste fingerprint (artists, subgenres, decades, intensity, instrumentation hints).
- Score candidates 0-100 with weighted criteria:
  - genre/subgenre overlap (35)
  - artist-neighborhood similarity (25)
  - era compatibility (15)
  - mood compatibility (15)
  - collection-pattern compatibility (10)
- A "100% fit" claim is allowed only if all weighted criteria are fully satisfied.
- If no candidate reaches 100, return best available matches and state highest score honestly.

Output:
- **Fit Matrix** table for top candidates.
- **Top Picks** (up to {max_results}) with score and evidence.
- **Play Order**: first 5 to try.
- **No-Match Notes**: what is missing for true 100% matches."#
    )
}

pub(crate) fn library_health_check_prompt(path: String) -> String {
    format!(
        r#"Perform a comprehensive health check on my music library at "{path}".

Run these tools in sequence:
1. `scan_directory` with `json_output: true` — get full track inventory.
2. `validate_library` with `json_output: true` — identify metadata issues.
3. `find_duplicates` with `json_output: true` — detect duplicate files.
4. `get_library_tree` — check organizational structure.

Compile a **Library Health Report** with:
- Overall health score (0-100).
- Top metadata issues by count.
- Duplicate summary and estimated reclaimable space.
- Top 5 fixes by impact.
- Exact `musicctl` commands to execute next."#
    )
}

pub(crate) fn metadata_cleanup_guide_prompt(path: String) -> String {
    format!(
        r#"Guide me through cleaning up metadata in my music library at "{path}".

Steps:
1. Use `validate_library` with `json_output: true` to find all issues.
2. Use `normalize` with `json_output: true` to preview normalization changes.
3. Use `scan_directory` with `json_output: true` for full context.

Output:
- Issue inventory by type and count.
- Quick wins that can be fixed automatically.
- Manual fixes requiring judgment.
- Ordered action plan with verification checkpoints.
- Exact command for apply step: `musicctl normalize "{path}" --apply`."#
    )
}

pub(crate) fn duplicate_resolution_prompt(path: String) -> String {
    format!(
        r#"Help me resolve duplicate files in my music library at "{path}".

Steps:
1. Use `find_duplicates` with `json_output: true`.
2. Use `scan_directory` with `json_output: true` for metadata on affected files.
3. Use `read_file_metadata` for close calls.

Output:
- Duplicate groups and estimated space savings.
- Keep/remove recommendation per group with reasoning.
- Confidence levels (high/medium/low).
- Caution list for likely intentional variants.
- Final cleanup action list."#
    )
}

pub(crate) fn cue_sheet_assistant_prompt(path: String) -> String {
    format!(
        r#"Help me manage CUE sheets in my music library at "{path}".

Steps:
1. Use `scan_directory` with `json_output: true`.
2. Validate existing CUE files with `cue_file` operation `validate`.
3. For albums without CUE, run `cue_file` operation `generate` with `dry_run: true`.

Output:
- Existing CUE files with validation status.
- Missing CUE opportunities.
- Fix recommendations and exact commands."#
    )
}
