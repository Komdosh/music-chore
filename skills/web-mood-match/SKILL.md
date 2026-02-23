# Web Mood Match

## Purpose

Find web recommendations that match a mood and still align with library taste.

## Use When

- User asks "what should I hear for this mood?"
- User wants new music but not random picks.

## Inputs

- `path` (optional)
- `mood` (required)
- `max_results` (optional, default `10`)

## MCP Prompt

- `web-mood-match`

## Expected Output

- mood anchors inferred from library
- scored web recommendations
- immediate listening queue
