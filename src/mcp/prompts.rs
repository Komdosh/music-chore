pub(crate) fn top_tracks_analysis_prompt(path: String) -> String {
    format!(
        r#"I want you to analyze my music library at "{path}" and predict my top 10 favorite tracks.

Here's how to approach this:

1. First, use the `scan_directory` tool with `json_output: true` to get all tracks.
2. Then use `get_library_tree` to understand the library hierarchy.
3. Analyze the results looking for these signals of a "loved" track:
   - **Artist frequency**: Artists with many albums/tracks likely indicate strong preference.
   - **Metadata completeness**: Well-tagged tracks suggest the owner cares about them.
   - **Genre patterns**: Identify the dominant genres — tracks in these genres are more likely favorites.
   - **Album completeness**: Complete albums (vs. scattered singles) suggest intentional collection.

Present your findings as:
- A ranked list of 10 tracks with reasoning for each pick.
- A brief "taste profile" summary explaining my overall music personality.
- 3 honorable mentions that almost made the list."#
    )
}

pub(crate) fn genre_breakdown_prompt(path: String) -> String {
    format!(
        r#"Analyze the genre distribution across my music library at "{path}".

Steps:
1. Use `scan_directory` with `json_output: true` to get all tracks and their metadata.
2. Use `emit_library_metadata` for structured genre data.

Provide:
- **Genre Distribution**: A percentage breakdown of all genres in my library, sorted by prevalence.
- **Genre Clusters**: Group related genres (e.g., "Rock → Alternative Rock → Indie Rock") and show how they connect.
- **Listening Identity**: Give me a creative 2-3 word "listener archetype" name (e.g., "Melancholic Indie Explorer" or "Rhythmic Jazz Purist").
- **Genre Gaps**: Identify genres that are notably absent given my existing taste profile.
- **Genre Evolution Hints**: Based on my collection, suggest which genres I might naturally gravitate toward next.
- **Cross-Genre Bridges**: Identify tracks or artists in my library that bridge between my top genres."#
    )
}

pub(crate) fn decade_analysis_prompt(path: String) -> String {
    format!(
        r#"Perform a temporal analysis of my music library at "{path}".

Steps:
1. Use `scan_directory` with `json_output: true` to extract track metadata including year/date tags.
2. Use `get_library_tree` to see the full library structure.

Analyze and present:
- **Decade Distribution**: A breakdown showing how many tracks fall in each decade (1950s–2020s).
- **Peak Decade**: Which decade dominates and what that says about my influences.
- **Nostalgia Index**: Am I primarily a "retrospective listener" or "contemporary explorer"?
- **Decade-Genre Map**: Show how genres shift across decades in my collection.
- **Time Travel Playlist**: Pick one standout track per decade that best represents my taste in that era.
- **Missing Eras**: If any decades are underrepresented, suggest what I'm missing out on."#
    )
}

pub(crate) fn collection_story_prompt(path: String) -> String {
    format!(
        r#"Tell me the story of my music collection at "{path}".

Steps:
1. Use `scan_directory` with `json_output: true` to get all tracks.
2. Use `get_library_tree` for the hierarchical view.
3. Use `emit_library_metadata` for the full structured metadata.

Write a compelling narrative that covers:
- **The Opening Chapter**: How does the collection begin? What's the foundational artist/genre?
- **Themes & Motifs**: What recurring patterns or themes emerge (genres, moods, eras)?
- **Diversity Score**: Rate the collection's diversity on a 1-10 scale with explanation.
- **Emotional Landscape**: Map the emotional range — is this a collection of joy, melancholy, energy, contemplation?
- **The Collector's Portrait**: Paint a picture of who owns this library. What can you infer about their personality, life stage, and experiences?
- **The Missing Chapter**: What's conspicuously absent that would complete the story?

Write this in an engaging, literary style — as if reviewing a curated exhibition."#
    )
}

pub(crate) fn artist_deep_dive_prompt(path: String, artist_name: Option<String>) -> String {
    let artist_clause = match artist_name {
        Some(name) => format!("Focus specifically on the artist \"{name}\"."),
        None => "Identify the most prominent artist in my library and focus on them.".into(),
    };

    format!(
        r#"Perform a deep dive into an artist from my music library at "{path}".

{artist_clause}

Steps:
1. Use `scan_directory` with `json_output: true` to get all tracks.
2. Use `get_library_tree` to understand the library hierarchy.

Provide:
- **Discography Coverage**: Which albums/tracks do I have? What am I missing?
- **Collection Completeness**: Rate as percentage with specific gaps identified.
- **Standout Tracks**: Which tracks by this artist are likely my favorites and why?
- **Genre/Style Evolution**: How does this artist's style evolve across the albums I own?
- **Library Context**: How does this artist connect to my other artists and genres?
- **Recommendations**: Suggest 3-5 missing albums/tracks I should add.
- **Similar Artists in Library**: Which other artists in my collection share DNA with this one?"#
    )
}

pub(crate) fn instrument_to_learn_prompt(path: String, level: &str) -> String {
    format!(
        r#"Based on my music library at "{path}", recommend what instrument I should learn.
My experience level: {level}.

Steps:
1. Use `scan_directory` with `json_output: true` to analyze my music.
2. Use `emit_library_metadata` for detailed metadata.

Provide:
- **Top 3 Instrument Recommendations**: Ranked by how well they match my music taste.
- For each instrument:
  - Why it matches my library's genres and artists.
  - 5 songs from my library I could learn (ordered from easiest to hardest).
  - Estimated time to play my first song at my experience level.
  - Recommended learning resources specific to my genres.
- **Genre-Instrument Map**: Show which instruments dominate each of my top genres.
- **The "Fun Factor" Pick**: Which instrument would let me jam along to the most tracks?
- **The "Surprise" Pick**: An unconventional instrument that would give my favorites a fresh twist."#
    )
}

pub(crate) fn similar_artists_discovery_prompt(path: String) -> String {
    format!(
        r#"Find new artists that closely match my library at "{path}".

Steps:
1. Use `scan_directory` with `json_output: true`.
2. Use `get_library_tree` with `json_output: true`.
3. Use `emit_library_metadata` with `json_output: true`.
4. Use web knowledge/search to identify artists NOT already in my library.

Method:
- Build a taste fingerprint from my dominant artists, genres, eras, and metadata quality signals.
- Score each candidate from 0-100 using explicit evidence.
- Require at least 2 concrete overlap signals per candidate.

Output:
- **Fingerprint Summary** (short).
- **Top 10 Artist Matches** with:
  - Fit score (0-100)
  - Why it matches (2-4 concrete signals)
  - One starter album
  - One starter track
  - Closest in-library reference artist
- **High-Confidence First 3**: best immediate picks to try now.
- **Rejected Candidates**: 3 artists you considered and why they failed fit."#
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

pub(crate) fn web_genre_scout_prompt(path: String, genre: &str, max_results: u32) -> String {
    format!(
        r#"Scout the web for music in genre "{genre}" that fits my library at "{path}".
Max results: {max_results}.

Steps:
1. Use `scan_directory` with `json_output: true`.
2. Use `emit_library_metadata` with `json_output: true`.
3. Build a genre-specific profile from my existing collection.
4. Use web search/knowledge for external candidates not present in my library.

Selection rules:
- Prefer candidates with strong overlap to my in-library genre patterns.
- Penalize candidates that are genre-adjacent but historically poor fit to my profile.
- Provide only high-confidence picks.

Output:
- **Genre Fit Summary**.
- **Top {max_results} Web Picks** with fit score and concrete rationale.
- **Start Here**: 3-track/album starter sequence."#
    )
}

pub(crate) fn web_mood_match_prompt(path: String, mood: &str, max_results: u32) -> String {
    format!(
        r#"Find web music that matches mood "{mood}" and still fits my library profile at "{path}".
Max results: {max_results}.

Steps:
1. Use `scan_directory` with `json_output: true`.
2. Use `get_library_tree` with `json_output: true`.
3. Use `emit_library_metadata` with `json_output: true`.
4. Infer mood anchors from my library.
5. Use web search/knowledge for candidates not already in my library.

Evaluation:
- Mood alignment first.
- Then verify compatibility with my dominant artists/genres/eras.
- Reject recommendations that match mood but conflict with established taste fingerprint.

Output:
- **Mood Anchors** from my library.
- **Best Web Matches** (up to {max_results}) with fit score and reasons.
- **Tonight Queue**: 5-item sequence for immediate listening."#
    )
}

pub(crate) fn mood_playlist_prompt(path: String, mood: &str, max: u32) -> String {
    format!(
        r#"Create a playlist from my music library at "{path}" for this mood/activity: "{mood}".
Maximum tracks: {max}.

Steps:
1. Use `scan_directory` with `json_output: true` to get all tracks with metadata.
2. Use `emit_library_metadata` for detailed track information.

Build the playlist considering:
- **Track Selection**: Pick tracks whose genre, tempo (inferred from genre/style), and mood align with "{mood}".
- **Flow & Sequencing**: Order tracks for natural energy flow — consider openers, builders, peaks, and cool-downs.
- **Variety**: Mix artists and albums while maintaining mood coherence.

Present:
- **Playlist Name**: A creative, evocative name for this playlist.
- **Track List**: Numbered list with Artist – Title – Album for each track.
- **Playlist Arc**: Brief description of the emotional journey the playlist creates.
- **Transition Notes**: For key transitions, explain why one track flows into the next.
- **Mood Match Score**: Rate how well my library serves this mood (1-10) and identify any gaps."#
    )
}

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

pub(crate) fn quick_pick_prompt(path: String) -> String {
    format!(
        r#"I am indecisive. Give me one immediate listening pick from my library at "{path}".

Steps:
1. Use `scan_directory` with `json_output: true`.
2. Use `emit_library_metadata` with `json_output: true`.

Rules:
- Return exactly 5 candidate tracks from different artists when possible.
- Use deterministic tie-breaking (artist frequency, metadata completeness, and genre fit).
- Then select exactly one track as **Play This First**.

Output:
- 5 candidates with one-line reasoning each.
- **Play This First** winner.
- 3-track follow-up sequence."#
    )
}

pub(crate) fn album_tonight_prompt(path: String, minutes: u32) -> String {
    format!(
        r#"Pick one album from my library at "{path}" for tonight.
Time available: {minutes} minutes.

Steps:
1. Use `scan_directory` with `json_output: true`.
2. Use `get_library_tree` with `json_output: true`.
3. Use `emit_library_metadata` with `json_output: true`.

Selection rules:
- Prefer complete albums.
- Target total runtime close to {minutes} minutes (allow +/- 15 minutes).
- If runtime is unavailable, estimate from track count and sequence coherence.
- Use only local library content.

Output:
- **Album Tonight**: artist + album + why this is the best fit.
- **Listening Plan**: start track and optional stop point if time runs short.
- 2 backup album choices."#
    )
}

pub(crate) fn rediscovery_rotation_prompt(path: String, max_tracks: u32) -> String {
    format!(
        r#"Create a rediscovery rotation from my library at "{path}".
Maximum tracks: {max_tracks}.

Steps:
1. Use `scan_directory` with `json_output: true`.
2. Use `get_library_tree` with `json_output: true`.
3. Use `emit_library_metadata` with `json_output: true`.

Look for:
- Deep cuts from frequently represented artists.
- Tracks from underrepresented artists/albums.
- Genre outliers that still connect to my main taste.

Output:
- **Rediscovery Rotation** of up to {max_tracks} tracks in sequence.
- One-line reason for each track.
- **If You Like This, Next Rotation**: 5 additional tracks."#
    )
}

pub(crate) fn decision_duel_prompt(
    path: String,
    option_a: &str,
    option_b: &str,
    max_tracks_per_option: u32,
) -> String {
    format!(
        r#"Help me choose between two listening directions from my library at "{path}".
Option A: "{option_a}".
Option B: "{option_b}".
Max tracks per option: {max_tracks_per_option}.

Steps:
1. Use `scan_directory` with `json_output: true`.
2. Use `emit_library_metadata` with `json_output: true`.

For each option:
- Suggest up to {max_tracks_per_option} tracks.
- Explain why each track matches the option.

Then:
- Compare Option A vs Option B for coherence, variety, and momentum.
- Choose a winner as **Tonight's Direction**.
- Provide a 6-track mini-queue for the winner."#
    )
}

pub(crate) fn hidden_gems_prompt(path: String) -> String {
    format!(
        r#"Search my music library at "{path}" for hidden gems — tracks and albums I might be overlooking.

Steps:
1. Use `scan_directory` with `json_output: true` to scan the full library.
2. Use `get_library_tree` for structure analysis.
3. Use `emit_library_metadata` for metadata details.

Look for:
- **Deep Cuts**: Non-single tracks from well-known artists that are critically acclaimed.
- **Solo Artists**: Tracks by artists with very few entries (1-2 tracks) — they were added for a reason.
- **Genre Outliers**: Tracks in genres that are rare in my collection — they stand out as intentional picks.
- **Album Closers/Openers**: Often the most carefully crafted tracks on an album.

Present:
- **10 Hidden Gems** with:
  - Track name, artist, album.
  - Why this qualifies as a hidden gem.
  - What makes it special or noteworthy.
  - When to listen to it (mood/setting recommendation).
- **The One You Forgot**: One track that's probably been sitting in your library unplayed.
- **Rediscovery Playlist**: Arrange all gems in an optimal listening order."#
    )
}

pub(crate) fn album_marathon_prompt(path: String, hours: u32, theme: Option<String>) -> String {
    let theme_clause = match theme {
        Some(t) => format!("Theme/approach: \"{t}\"."),
        None => "Choose the most interesting sequencing approach for my collection.".into(),
    };

    format!(
        r#"Design a {hours}-hour album listening marathon from my library at "{path}".
{theme_clause}

Steps:
1. Use `scan_directory` with `json_output: true` to get all tracks.
2. Use `get_library_tree` to see album completeness.
3. Use `emit_library_metadata` for full album details.

Create a marathon plan with:
- **Album Selection**: Choose complete albums (or near-complete) that fit within {hours} hours.
- **Sequencing Strategy**: Explain the logic behind the album order (chronological, genre flow, energy arc, etc.).
- **Schedule**: Approximate timestamps for when each album starts/ends.
- **Intermissions**: Suggest 1-2 break points with palate-cleansing single tracks.
- **Listening Notes**: For each album, provide a 1-2 sentence "what to listen for" guide.
- **The Finale**: Which album closes the marathon and why it's the perfect ending.
- **Snack Pairings**: Fun bonus — suggest a drink/snack that matches each album's vibe!"#
    )
}

pub(crate) fn concert_setlist_prompt(path: String, minutes: u32, vibe: Option<String>) -> String {
    let vibe_clause = match vibe {
        Some(v) => format!("Vibe/genre focus: \"{v}\"."),
        None => "Pull from all genres in my library for maximum variety.".into(),
    };

    format!(
        r#"Build a dream concert setlist from my library at "{path}".
Target duration: {minutes} minutes.
{vibe_clause}

Steps:
1. Use `scan_directory` with `json_output: true` for the full track catalog.
2. Use `emit_library_metadata` for detailed metadata.

Structure the setlist like a real concert:
- **Opener** (1-2 tracks): High energy to grab attention.
- **Early Set** (3-4 tracks): Establish the vibe, crowd-pleasers.
- **Deep Cut Segment** (2-3 tracks): Reward the dedicated fans.
- **Acoustic/Chill Break** (1-2 tracks): Emotional breather.
- **Build-Up** (3-4 tracks): Rising energy toward the climax.
- **Main Set Closer** (1 track): The biggest anthem in my collection.
- **Encore** (2-3 tracks): One intimate track, then go out with a bang.

For each track include:
- Position, artist, title.
- Why it fits this slot.
- Crowd energy level (1-10).
- Imagined crowd reaction.

End with a "setlist poster" — formatted like a real concert poster with the show name, venue vibe, and lineup."#
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

📊 **Overall Health Score**: X/100 with letter grade.

🏗️ **Structure Assessment**:
  - Folder hierarchy consistency.
  - Naming convention adherence.
  - Orphaned or misplaced files.

🏷️ **Metadata Quality**:
  - Percentage of tracks with complete metadata.
  - Most common missing fields.
  - Inconsistencies (e.g., same artist spelled differently).

🔁 **Duplicate Report**:
  - Number of duplicate groups found.
  - Total wasted space.
  - Recommended deletions.

⚠️ **Critical Issues** (fix immediately):
  - List top 5 most urgent problems.

🔧 **Improvement Plan** (prioritized):
  - Step-by-step actions ordered by impact.
  - Estimated effort for each step.
  - Which `musicctl` commands to run.

Present the report in a clean, professional format."#
    )
}

pub(crate) fn metadata_cleanup_guide_prompt(path: String) -> String {
    format!(
        r#"Guide me through cleaning up metadata in my music library at "{path}".

Steps:
1. Use `validate_library` with `json_output: true` to find all issues.
2. Use `normalize` with `json_output: true` to preview normalization changes.
3. Use `scan_directory` with `json_output: true` for the full picture.

Create a **Metadata Cleanup Guide**:

📋 **Issue Inventory**:
  - Categorize all found issues by type (missing title, bad genre, inconsistent artist name, etc.).
  - Count how many tracks are affected by each issue type.

🎯 **Quick Wins** (automated fixes):
  - Issues that `normalize` can fix automatically.
  - Show before/after previews for each normalization.
  - Provide the exact command to apply: `musicctl normalize "{path}" --apply`.

✏️ **Manual Fixes Required**:
  - Issues that need human judgment (e.g., ambiguous artist names).
  - For each, suggest the most likely correct value.

📐 **Consistency Check**:
  - Artist name variations (e.g., "The Beatles" vs "Beatles").
  - Genre standardization opportunities.
  - Title formatting inconsistencies.

🚀 **Action Plan**:
  - Ordered steps to resolve everything.
  - Start with automated fixes, then manual ones.
  - Checkpoint after each step to verify."#
    )
}

pub(crate) fn duplicate_resolution_prompt(path: String) -> String {
    format!(
        r#"Help me resolve duplicate files in my music library at "{path}".

Steps:
1. Use `find_duplicates` with `json_output: true` to identify all duplicate groups.
2. Use `scan_directory` with `json_output: true` for full metadata on affected files.
3. For key duplicates, use `read_file_metadata` on individual files for detailed comparison.

Provide a **Duplicate Resolution Report**:

📊 **Summary**:
  - Total duplicate groups found.
  - Total redundant files.
  - Estimated space savings.

🔍 **For each duplicate group**:
  - List all copies with their paths.
  - Compare: format, bitrate/quality, metadata completeness, file size.
  - **Recommendation**: Which copy to KEEP and why (prefer: higher quality → better metadata → better file path).
  - Mark as: ✅ Keep, ❌ Remove.

⚡ **Batch Actions**:
  - Group deletions by confidence level (high/medium/low).
  - High confidence: exact checksums, clear quality winner.
  - Low confidence: different versions, remasters, etc.

⚠️ **Caution List**: Duplicates that might actually be intentional (e.g., album version vs. compilation, remastered vs. original).

📝 **Cleanup Script**: Provide a summary of recommended file removals."#
    )
}

pub(crate) fn reorganization_plan_prompt(path: String) -> String {
    format!(
        r#"Analyze my music library at "{path}" and suggest an optimal reorganization strategy.

Steps:
1. Use `get_library_tree` to see the current folder structure.
2. Use `scan_directory` with `json_output: true` for all tracks.
3. Use `validate_library` with `json_output: true` for structural issues.

Create a **Reorganization Plan**:

📁 **Current Structure Analysis**:
  - Describe the existing folder hierarchy pattern.
  - Identify inconsistencies and deviations.
  - Rate current organization (1-10).

🏗️ **Recommended Structure**:
  - Propose an `Artist/Album/Track` hierarchy.
  - Naming convention: `Artist Name/[Year] Album Name/## - Track Title.ext`.
  - Handle edge cases: compilations, soundtracks, singles, multi-disc albums.

🔀 **Migration Plan**:
  - List specific files/folders that need to move.
  - Show before → after paths for each.
  - Group moves by priority (worst offenders first).

📏 **Naming Standards**:
  - Capitalization rules.
  - Character handling (special chars, unicode).
  - Track number formatting (01, 02 vs 1, 2).

🛡️ **Safety Steps**:
  - Backup recommendations before restructuring.
  - How to verify nothing was lost after reorganization.
  - Metadata preservation during moves."#
    )
}

pub(crate) fn format_quality_audit_prompt(path: String, suggest_upgrades: bool) -> String {
    let upgrade_clause = if suggest_upgrades {
        "Include specific upgrade recommendations for lossy files."
    } else {
        "Focus on the audit only, skip upgrade recommendations."
    };

    format!(
        r#"Audit the audio quality and formats in my music library at "{path}".
{upgrade_clause}

Steps:
1. Use `scan_directory` with `json_output: true` to get all tracks with format details.
2. Use `emit_library_metadata` for structured metadata including format info.
3. Use `get_library_tree` for the full library view.

Create a **Format & Quality Audit Report**:

📊 **Format Distribution**:
  - Breakdown: FLAC, MP3, WAV, DSF, WavPack, and any others.
  - Percentage and file count for each format.

🎧 **Quality Tiers**:
  - 🥇 Lossless Hi-Res (24-bit FLAC, DSF, etc.)
  - 🥈 Lossless Standard (16-bit FLAC, WavPack)
  - 🥉 High-Quality Lossy (MP3 320kbps, etc.)
  - ⚠️ Low-Quality Lossy (MP3 <192kbps)

💾 **Storage Analysis**:
  - Total library size estimate by format.
  - Space savings if lossy were used vs. space cost of going full lossless.

🔄 **Upgrade Recommendations** (if requested):
  - Priority list of albums/tracks to upgrade from lossy to lossless.
  - Prioritize: favorite artists first, then albums with mixed formats.

🏆 **Audiophile Score**: Rate my library's overall quality (1-10) with explanation."#
    )
}

pub(crate) fn year_in_review_prompt(path: String, year: u32) -> String {
    format!(
        r#"Create a "{year} Year in Review" for my music library at "{path}".

Steps:
1. Use `scan_directory` with `json_output: true` to get all tracks.
2. Use `get_library_tree` for the library overview.
3. Use `emit_library_metadata` for detailed track data.

Generate a **{year} Music Collection Review**:

📈 **Collection Stats**:
  - Total tracks, albums, and artists in the library.
  - Tracks from {year} specifically.
  - Library growth indicators.

🏆 **Top of {year}**:
  - Top 5 artists by track count.
  - Top 5 albums (most complete).
  - Genre of the year.

🎵 **Highlights**:
  - Most interesting additions from {year}.
  - Genre diversification — any new genres added?
  - The oldest and newest recordings in the collection.

📊 **By the Numbers**:
  - Average tracks per album.
  - Most prolific artist.
  - Format breakdown for {year} additions.

🔮 **{} Preview**:
  - Based on trends, predict what genres/artists I might add next.
  - Suggest 5 albums from {} that would complement my collection.

Present this like a premium music streaming service's annual wrap-up!"#,
        year + 1,
        year + 1
    )
}

pub(crate) fn cue_sheet_assistant_prompt(path: String) -> String {
    format!(
        r#"Help me manage CUE sheets in my music library at "{path}".

Steps:
1. Use `scan_directory` with `json_output: true` to find all music files.
2. For any existing .cue files found, use `cue_file` with operation "validate" to check them.
3. For directories with album tracks but no .cue file, use `cue_file` with operation "generate" and `dry_run: true` to preview.

Provide a **CUE Sheet Management Report**:

📋 **Existing CUE Files**:
  - List all found CUE files and their validation status.
  - For invalid CUE files: describe the specific issues.

🆕 **Missing CUE Files**:
  - Albums that would benefit from a CUE sheet.
  - Preview the generated CUE content.

🔧 **Fix Recommendations**:
  - For each issue, provide the fix.
  - Suggest exact commands to regenerate problematic CUE files.

📖 **CUE Sheet Best Practices** specific to my library's formats."#
    )
}
