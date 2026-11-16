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

## PA06: ghost coverage, approval hash and publish checks (GHC-01 to GHC-03, PUB-01, APR-01)

- Commit: see `git log --grep "PA06"`.
- Files: `crates/grimoire/src/pipeline/ghost_cover.rs`, `crates/grimoire/src/pipeline/approval.rs`, `crates/grimoire/tests/pipeline_ghost_approval.rs`, fixtures under `crates/grimoire/tests/fixtures/pipeline/` (`one-ghost`, `ghost-not-reproducing`, `publish-without-approval`, `approval-hash-mismatch`).
- Check:
  - The manifest hash is not computed in Rust. It arrives as text from `sha256sum` in the assembly script (PA07). The one-byte change test is in PA07, as the blockers log records.
  - Ghost `program` values keep the `incorrect/` prefix used in `known_ghosts.toml`. GHC-02 points at that program path.
  - `approval.toml` is parsed with `deny_unknown_fields`, so an approval cannot carry a hidden field.

## PA07: assembly script for the two trees (`scripts/pack-assemble.sh`)

- Commit: see `git log --grep "PA07"`.
- Files: `scripts/pack-assemble.sh`, `scripts/tests/pack-assemble-test.sh` (17 checks, temporary directories, synthetic canary `CANARY-HIDDEN-pa7`). The real private repository was not used.
- Shell choices: `set -euo pipefail`; every path is quoted; the slug must match `^[a-z0-9][a-z0-9-]*$` (blocks `../`); symbolic links in either tree are refused; a private file name or folder in the public tree, or the same relative path in both trees, gives `LAY-05` (exit 3); a missing private tree gives `PIP-34` (exit 4). The assembled copy is removed and rebuilt, and only under `PACK_ASSEMBLY_ROOT`.
- Hashing: `sha256sum` over the sorted manifest. The one-byte change test (PA06 blockers note) is case 4 of the test.
- Check: the script prints only the content hash on stdout. Diagnostics name codes and file names, never contents.
- Note: the `packs/` layout uses the public and private roots as the repository roots (`<root>/packs/<slug>`). Confirm this before PA08 and the importer.

## PA12: reviewer agent definition and review skill

- Commit: see `git log --grep "PA12"`.
- Files: `.kiro/agents/pack-reviewer.json`, `.kiro/skills/pack-review/SKILL.md`, test `crates/grimoire/tests/pipeline_agents.rs` (3 checks).
- Format: confirmed with the Kiro configuration reference through `introspect`. The `custom-agent-creator` agent described Markdown agents and could not confirm JSON, so its answer was not used. A JSON agent takes `prompt` for the system text, and `tools` takes tags. `tools` is `["read"]` only, so there is no write, shell or code-running tool.
- Check:
  - The `validate_pack` MCP tool is not used yet, because counterspell-mcp is not built. The reviewer reads the S9 report that the orchestrator passes in. When the MCP tool exists, add it to `tools` in a new review.
  - The agent prompt names the checklist and says it has no author notes.
  - The agent is read-only, and it cannot run any code.

## PA15: review digest and routing script (`scripts/pack-review-digest.sh`)

- Commit: see `git log --grep "PA15"`.
- Files: `scripts/pack-review-digest.sh`, `scripts/tests/pack-review-digest-test.sh` (7 checks).
- Subcommands: `prepare` copies the assembled pack, the checklist and the reports into `target/pack-review/<slug>/input`, which holds hidden files and stays under the git-ignored `target/`. It prints only that path. `publish-summary` runs the `SUM-01` scan and writes to `docs/pack-reviews/` only when the scan is clean.
- Check: the scan looks for fixed tokens (`hidden.toml`, `generator.toml`, `known_ghosts.toml`, `approval.toml`, `retired-values.toml`, `reference/`, `incorrect/`, `/case/`, `CANARY-`). It reports the code `SUM-01` and never the matched line. This is a first filter. A reviewer still reads each summary.
- PA14 check: the template and the README pass `publish-summary`.
