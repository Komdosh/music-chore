# Web Perfect Match

## Purpose

Find the closest possible web recommendations from the user's library fingerprint.

## Use When

- User wants "best possible fit" recommendations from outside local library.
- User may provide mood or genre constraints.

## Inputs

- `path` (optional)
- `mood` (optional)
- `genre` (optional)
- `max_results` (optional, default `10`)

## MCP Prompt

- `web-perfect-match`

## Fit Rules

- Build local fingerprint first.
- Score recommendations with explicit evidence.
- Claim 100% fit only if all criteria are fully satisfied; otherwise report real score.

## Expected Output

- fit matrix
- ranked recommendations
- immediate play order
