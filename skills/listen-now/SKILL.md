# Listen Now

## Purpose

Resolve "what should I play right now?" quickly.

## Use When

- User has limited time.
- User provides a mood/activity.
- User wants confident recommendation, not endless options.

## Inputs

- `path` (optional)
- `available_minutes` (optional, default `45`)
- `mood` (optional, default `any`)
- `novelty_preference` (optional, default `balanced`)

## MCP Prompt

- `listen-now`

## Expected Output

- One primary pick (`Start Now`)
- Two fallback options
- A short ordered queue
