# pack-authoring-pipeline: supervised task review queue

Each supervised task (`[SUPERVISED]`) is implemented test-first and committed. Brighton reviews the diff before the task counts as approved. Each entry gives the task, the commit, and what to check.

## PA04: originality marker guard (ORG-01, ORG-02)

- Commit: see `git log --grep "PA04"`.
- Files: `crates/grimoire/src/pipeline/originality.rs`, `crates/grimoire/tests/pipeline_originality.rs`, fixtures under `crates/grimoire/tests/fixtures/pipeline/copied-marker/` and `no-originality-record/`.
- Check:
  - The marker list is data passed in at run time. Code never contains real markers. Tests use the synthetic `SYNTH-MARKER-ALPHA`.
  - Reports carry a code and a `file line n` pointer only, never the matched text.
  - The five S1 section headings are an assumed choice (see the pipeline blockers log). Confirm or rename them before the first real pack.
  - A blank marker is refused, because it would match every line.

## PA05: hint leak check (HNT-03, GRM-27)

- Commit: see `git log --grep "PA05"`.
- Files: `crates/grimoire/src/pipeline/hint_leak.rs`, `crates/grimoire/tests/pipeline_hint_leak.rs`, fixtures `hint-leaks-expected/` and `hint-leaks-ghost-got/`. `Diagnostic` moved to `pipeline/mod.rs` and is shared with PA04.
- Check:
  - Matching is by whole token, so `41` does not match inside `2041`. Short values such as `3` can still flag an ordinary rung. A reviewer decides whether that flag is a real leak.
  - The fenced-block rule reports `GRM-27` (the existing code), not `HNT-03`.
  - The report names a rung and a value index only.
