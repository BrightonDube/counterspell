# Test packs (`packs/_test`)

These three packs exist to test the ghost engine. They are not catalogue problems.

## The `_` prefix

Any directory whose name starts with `_` is for tests only. The pack importer must skip it, so these packs never reach the catalogue. `validate --all packs` skips them too; `validate --all packs --include-underscore` validates each pack inside a `_` container (the container itself is not a pack).

## Layout

Each pack is a self-contained Grimoire pack in the two-tree format (grimoire-packs T29, amendment X1). A single tree that carries private-resident files is an assembled tree, so public and private files sit side by side here:

- `problem.md`: the statement, in the `## Task` / `## Input` / `## Output` / `## Examples` / `## Constraints` order.
- `meta.toml`: slug, title, difficulty, track, concepts, function, params, limits, `input_schema` and samples. No `[generator]` table — that lives in `generator.toml`.
- `hints.toml`: the four-rung ladder.
- `generator.toml`: the private generation seed and `hidden_count`.
- `hidden.toml`: the curated hidden cases — the hand-verified golden vectors, each with a `why`.
- `known_ghosts.toml`: the authored ghosts, each pointing at the `incorrect/` program that shows it.
- `reference/solution.py`: the trusted reference.
- `incorrect/*.py`: deliberately wrong programs. Each is referenced by a ghost and has an engine descriptor.

The engine-only descriptors that have no pack-format slot — per-program kind, expected exception class, smallest input, `dogfood` flag and hand-traced vectors — live in `crates/engine/tests/fixtures/<slug>.json`. A grimoire-side test (`crates/grimoire/tests/test_pack_projection.rs`) keeps that projection in sync with these files.

The packs are `pair-count`, `longest-run` and `diagonal-sum`. Each has two incorrect programs. The incorrect programs cover a wrong answer and a crash.

## What the tests check, and what they do not

Python files are never run on the host. The Rust test support in `crates/engine/tests/support/` mirrors each Python file with a model function. The tests then check that:

1. Every reference model reproduces its golden vectors and samples.
2. Every incorrect model reproduces its hand-traced vectors.
3. Each incorrect program yields a verified ghost, and that ghost matches the smallest input recorded in its descriptor. The check is a regression expectation for the current shrinker, not a claim of minimality.
4. Each reference hunted against itself reports `NotFound`.

**The limit:** the models stand in for the sandbox. A model can drift from its Python file, and the hand-traced vectors are the only control against that. The static checks in `packs.rs` only confirm the file names and function names match.

**Conformance hand-off:** the `judge-sandbox` and `run-lifecycle` specs must run these same vectors, and the same expected minimized inputs, through the real Python files in the sandbox. Until that passes, this spec's definition of done is "proven against faithful models", not "proven against Python".

## Dogfooding candidate

`pair-count/incorrect/same-index.py` is flagged `"dogfood": true` in its engine descriptor. It is the buggy implementation that the Kiro dogfooding demo (MCP-3) can point `find_ghost_input` at.

## Authoring rules

- Write the statement, vectors and known-ghost explanations in your own words. Do not copy text from other sites.
- Hand-check every vector against the Python source before you commit it.
- Keep known ghosts in their smallest form, and fill in `got` so that a different bug cannot borrow the explanation.
- After editing a pack file, keep the engine projection in sync: `crates/grimoire/tests/test_pack_projection.rs` fails when they drift.
