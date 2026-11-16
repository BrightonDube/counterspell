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
