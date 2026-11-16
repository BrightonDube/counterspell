---
name: grimoire-pack-authoring
description: Use when drafting a Counterspell Grimoire problem pack (statement, metadata, hints, reference, hidden cases, known ghosts) for the pack authoring pipeline.
---

# Grimoire pack authoring

Write one original problem as a pack. A pack has a public part (`problem.md`, `meta.toml`, `hints.toml`, `originality.md`) and a private part (`hidden.toml`, `generator.toml`, `reference/solution.py`, `incorrect/*.py`, `known_ghosts.toml`). Follow the stage prompts in `.kiro/pipeline/prompts/` one stage at a time. The author checklist is `.kiro/pipeline/checklists/author.md`.

## Rules

1. **Original wording only.** Write the statement, the title, the scenario and the examples yourself. Do not open a third-party problem. Use general patterns, never their wording.
2. **Supported types.** Parameters and results use `int`, `str`, `list[int]` or `list[list[int]]`. Do not add another type in this pipeline.
3. **Statement.** Use a title line and the section order that the validator expects (`STM-01`). Keep it short. The statement must not be too long or contain raw HTML (`STM-02`). Say "integer" wherever negative numbers are allowed.
4. **Hidden cases.** Write at least eight cases by hand. Derive every expected value on paper. Do not run the reference to get it. Each case has a `why` note.
5. **Ghosts.** Write at least two incorrect programs, each a realistic beginner mistake. Hand-trace the `got` value of each ghost. Never describe a ghost as the smallest possible input.
6. **Hints.** Write exactly four rungs in order (`HNT-01`). Rung 3 is pseudo-code of three lines at most. A code fence longer than three lines is an error (`HNT-02`). No rung may quote a sample, hidden or ghost value.
7. **Metadata.** Keep `origin = "original"`, `licence = "MIT"` and `pack_version = 1` for a new pack. A missing or wrong field is `MET-02`. Do not add `reference_complexity` or `[bench]` in this pipeline.
8. **Never run pack code on the host.** The validator reads files only. Code runs only through the judge sandbox, when the operator asks for it.

## Checks

- Run the static validator on the assembled tree (see stage S9). Read each code in its diagnostic and fix the cause.
- Layout errors (`LAY-05`) mean a private file is in the public tree, or the reverse. Move it to the right tree. Never copy hidden data into a public file.
- A reference that disagrees with a sample is `SBX-01`. Fix the reference or the sample, then re-check. Do not change the sample to match a wrong answer.

## Output

Leave the public files in `packs/<slug>/` and the private files in the private tree. Keep `status = "draft"`. Never set `published` and never write `approval.toml` (PIP-1).
