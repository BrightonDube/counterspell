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
