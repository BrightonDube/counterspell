---
name: author-problem
description: Author an original Counterspell problem pack (statement, metadata, schema, reference, incorrect programs, hints, known ghosts) and check it with the Counterspell MCP tools. Use when a user asks to write, add or review a problem pack.
---

# Author a Counterspell problem

## Rules for content

- Write every statement, sample, hint and program yourself. Do not copy text from another site or a third-party problem set.
- Put each pack in `packs/<slug>/` with the Grimoire layout: `problem.md`, `meta.toml`, `hints.toml`, `reference/solution.py`, and optionally `known_ghosts.toml` and `incorrect/*.py`.
- Keep hidden test data out of prompts, logs, chat replies and diagnostics. Show samples only.
- Use `packs/_<name>/` for private or demo packs. Public import skips underscore-prefixed directories.

## Steps

1. Draft `problem.md` and `meta.toml`: the function name, typed parameters, return type, comparator (`exact` or `unordered`), limits, the input schema, the generator seed and count, and the samples.
2. Write `reference/solution.py`. It is the trusted reference, so keep it simple and correct.
3. Add `hints.toml` with the authored hint ladder.
4. Run **`validate_pack`** on the pack path. It is static only and never runs code. Fix every error it reports, then run it again.
5. Only when a pack needs a behaviour check, ask the user for explicit approval before any code-running tool. Code-running tools are `run_reference`, `find_ghost_input` and `shrink_input`. They run only in the separate rootless sandbox and fail with `sandbox_not_configured` when the sandbox is not set up. Never run `reference/solution.py` or an incorrect program directly on the host.
6. After approval, call **`run_reference`** on a few inputs to confirm the reference returns the expected samples.
7. To add a known ghost, call **`find_ghost_input`** on an incorrect program. Record the minimized counterexample it returns in `known_ghosts.toml`, with an authored explanation. Describe it as the smallest failing input found within the budget. Never call it the smallest possible input.
8. Run **`validate_pack`** again before you report the pack as ready.

## Tools this skill uses

| Tool | Runs code? | Auto-approval |
|---|---|---|
| `validate_pack` | No | Yes |
| `explain_verdict` | No | Yes |
| `fit_complexity` | No | Yes |
| `run_reference` | Yes, in the sandbox | No: ask first |
| `find_ghost_input` | Yes, in the sandbox | No: ask first |
| `shrink_input` | Yes, in the sandbox | No: ask first |

## Reporting

State which checks passed and which code-running calls were made. If the sandbox was unavailable, say so. Do not claim a pack is verified without those results.
