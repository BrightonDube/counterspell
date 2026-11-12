# Test packs (`packs/_test`)

These three packs exist to test the ghost engine. They are not catalogue problems.

## The `_` prefix

Any directory whose name starts with `_` is for tests only. The pack importer and validator must skip it, so these packs never reach the catalogue.

## Layout

Each pack directory holds:

- `problem.md`: the original statement.
- `ghost.json`: engine data. It has the schema, comparator, concept tags, samples, authored known ghosts, and the list of incorrect programs with their hand-traced vectors.
- `golden.json`: hand-verified reference vectors.
- `reference/solution.py`: the trusted reference.
- `incorrect/*.py`: deliberately wrong programs. Each one is listed in `ghost.json`.

The packs are `pair-count`, `longest-run` and `diagonal-sum`. Each has two incorrect programs. The incorrect programs cover a wrong answer and a crash.

## What the tests check, and what they do not

Python files are never run on the host. The Rust test support in `crates/engine/tests/support/` mirrors each Python file with a model function. The tests then check that:

1. Every reference model reproduces its golden vectors and samples.
2. Every incorrect model reproduces its hand-traced vectors.
3. Each incorrect program yields a verified ghost, and that ghost matches the smallest input recorded in `ghost.json`. The check is a regression expectation for the current shrinker, not a claim of minimality.
4. Each reference hunted against itself reports `NotFound`.

**The limit:** the models stand in for the sandbox. A model can drift from its Python file, and the hand-traced vectors are the only control against that. The static checks in `packs.rs` only confirm the file names and function names match.

**Conformance hand-off:** the `judge-sandbox` and `run-lifecycle` specs must run these same vectors, and the same expected minimized inputs, through the real Python files in the sandbox. Until that passes, this spec's definition of done is "proven against faithful models", not "proven against Python".

## Dogfooding candidate

`pair-count/incorrect/same-index.py` is flagged `"dogfood": true` in `ghost.json`. It is the buggy implementation that the Kiro dogfooding demo (MCP-3) can point `find_ghost_input` at.

## Authoring rules

- Write the statement, vectors and known-ghost explanations in your own words. Do not copy text from other sites.
- Hand-check every vector against the Python source before you commit it.
- Keep known ghosts in their smallest form, and fill in `got` so that a different bug cannot borrow the explanation.
