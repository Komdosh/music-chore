# Music Discovery Skills

This folder contains practical skills for the core pain point: deciding what to listen to.

Each skill maps directly to an MCP expert prompt and uses local library data only.

## Available Skills

- `listen-now`: decide what to play right now based on time, mood, and novelty.
- `quick-pick`: give one immediate starter track when indecision is high.
- `album-tonight`: choose one album session that fits available time.
- `rediscovery-rotation`: surface overlooked tracks for library rediscovery.
- `decision-duel`: compare two listening directions and choose one winner.
- `web-perfect-match`: find highest-fit recommendations on the web.
- `web-genre-scout`: find web recommendations for a target genre.
- `web-mood-match`: find web recommendations for a target mood/activity.

## Usage Pattern

1. Call the matching MCP prompt.
2. Let the model run `scan_directory`, `get_library_tree`, and `emit_library_metadata` as needed.
3. Return a concrete queue, not just analysis.
