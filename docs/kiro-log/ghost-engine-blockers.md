# ghost-engine blockers (run of 12 Nov 2026)

## Not started: T20 to T26
- T20 (test support with model oracles), T21 (properties), T22 (pack definition-of-done tests), T23 (purity guard), T24 (wording and serialisation tests), T25 (crate and pack docs), T26 (full gate) are unchecked.
- Reason: the run ran out of working room after T19. Each is a sizeable Rust change that needs its own red/green cycle and gate. Nothing was half-written.
- Unblock: rerun the spec task by task starting at T20. `packs/_test/` already has the ghost.json, golden.json, reference and incorrect files that T20 loads.

## T17 to T19: hand-traced vectors need Brighton's review
- Python files were never run on the host (steering rule). The vectors in ghost.json and golden.json were derived by hand.
- Brighton should check each vector against the Python source, and check the known-ghost explanations for wording.
- The "smallest input" entries are regression expectations for the current shrinker, not proven minima.

## Environment issues (not fixed here, need Brighton)
- `.kiro/hooks/guard-secrets.sh` (PreToolUse on write) rejected writes that contained `re_` followed by 20+ identifier characters, case-insensitively, for example `are_` in a test name. This is a false positive. Test names were shortened instead. The guard was not changed.
- `scripts/kiro-autopush.sh` is running (pid 1430880). It pushes `brighton/kiro-setup` every 10 minutes, so commits made by this run are pushed. This conflicts with "do not push". I did not stop it, because it is not this run's process. Please stop it or confirm it should stay.
