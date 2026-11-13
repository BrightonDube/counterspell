# grimoire-packs review queue

No task in `tasks.md` is marked `[SUPERVISED]`, so no item here is a supervised task.

## For Brighton to decide before the starter packs are reviewed

1. **Open question 1 (most important):** are the reference solution and the hidden-test seed public in git? The current code assumes yes. Changing this would move `reference/` and `[generator]` outside the repo and change GRM-6 and GRM-34.
2. **Open question 7:** the `PackRunner` trait lives in `counterspell-grimoire`, and the judge depends on grimoire. Confirm before the judge spec starts (T20).
3. **Open question 9:** the `author` byline is a placeholder. No starter pack has been written yet.

## Items to check in the code (commits to review)

- `packs/README.md` (commit d0fa37d): the plain-English rule for each code. Check that the wording matches Brighton's intent for LAY-02 (dot files) and MET-03 (licence and origin).
- `crates/grimoire/src/codes.rs` (commit 361bb0d): the severity of SBX-04, SBX-05 and SBX-08 as warnings. Check this matches the design table.
- `crates/grimoire/src/ghosts.rs` (commit 1cb6bd9): the copy of the forbidden-phrase list and its drift guard.
- `crates/grimoire/src/main.rs` (commit f75291d): `--all` skips `_` packs, and `--sandbox` exits 2 with SBX-07.

## Review of waves 0 to 2 and 4 to 5

The requesting-code-review step was not run as a subagent in this session. The review checklist was applied by hand, for each task, through the tests and the Clippy gate. An independent code review is still needed before the spec is marked done.

## Continuation run (13 Nov 2026)

No task in `tasks.md` is marked `[SUPERVISED]`, so nothing here is a supervised item. The wave-end review was done by hand, against `requesting-code-review/code-reviewer.md`, because the template forbids spawning a review subagent.

### For Brighton to check

1. **Starter packs** (`packs/count-pairs`, `packs/longest-run-length`, `packs/diagonal-total`): statements, hints, ghost explanations and reference solutions are drafts. Confirm the wording and the `author` byline before publishing (T28).
2. **Known ghosts are hand traces.** The `got` values and crash classes are checked by the model runner only. The judge spec re-runs them in Python (T20).
3. **Open question 1** (reference and hidden seed are public in git) is still assumed, not confirmed.
4. **Open question 7** (`PackRunner` lives in grimoire, the judge implements it) must be confirmed before T20.

### Review findings

- **Critical:** none.
- **Important, fixed in this run:** the generator check could make the engine panic on an invalid schema (MET-06). Fixed on the grimoire side (commit `c3b878c`).
- **Important, open:** `validate --sandbox` cannot reach SBX-06 until a runner is wired (T20). Recorded in the blockers file.
- **Important, open, another spec:** the engine's `tiny_ints` panics on min > max. See the blockers file.
- **Minor, not changed:**
  - `tests/sandbox_fake.rs` duplicates the `count-pairs` text instead of reading `packs/count-pairs`. A change to the pack needs both edits.
  - The sandbox stage uses a local `observe` helper, not `ProgramOracle`. The design suggested reuse. Both do the same thing, so this is a style point.
  - SBX-03 pointers use the index after the known-ghost filter. Only an invalid ghost kind shifts it, and static checks reject that before the sandbox runs.
  - The mutation suite is Unix only (`#![cfg(unix)]`), because it creates a symlink. The CI on Linux covers it.

## Implement pass (13 Nov 2026): wave 5 review for T19

Review was done by a subagent against `requesting-code-review/code-reviewer.md`. Its findings and what was done:

- **Important, fixed:** the exit-2 test only checked the code. Now `sandbox_reports` returns the fault, and a test asserts `SBX-06` on it.
- **Important, fixed:** the SBX-07 path had no test. Added `missing_runner_exits_two_with_sbx_07`.
- **Important, fixed:** faults printed plain text even under `--json`. `print_fault` now prints a JSON report when `--json` is set.
- **Minor, open:** with `validate --all --sandbox`, a fault on a later pack drops the earlier packs' reports without printing them. Exit 2 is still correct. Fix when the judge runner exists (T20).
- **Minor, open:** the exit-path test reads `packs/count-pairs`. If that pack changes so it no longer passes the static checks, the test fails for an unrelated reason. A small fixture under `tests/` would be steadier.
- **Minor, open:** the `--json` fault output has no test (see the blockers file).

### For Brighton to check

- Open question 7 (who owns `PackRunner`) is still the gate for T20. Nothing else in this spec needs a decision before the judge spec starts.
