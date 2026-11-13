# Run-lifecycle blockers

Written by the implementation run on 13 Nov 2026. Each entry names the task, the reason it is not finished, and what would unblock it.

## Resolved: write-guard false positive (13 Nov 2026)

`scripts/kiro-hooks/guard-secrets.sh` now anchors `sk-`, `gsk_` and `re_` API-key patterns at a token boundary, so identifiers such as `failure_reason_public_messages_are_distinct` are no longer blocked. The hook prints which pattern class matched. T02 onward can continue. The entries below are kept as history of the earlier session.

## Environment: PreToolUse hook blocks some test writes (T02, T03)

- **What happened:** the file-write tool was blocked by a PreToolUse hook several times. The hook returned exit code 2 with no reason. The blocks were not tied to the test-integrity guard, which only counts removed tests and added skip markers, and that guard passes when run by hand with a bare path. Blocks came and went with content. The `FailureReason` tests were blocked every time they were tried, while the enum and its message table went through. The blocks were not bypassed (no shell writes were used to get around them).
- **Tasks affected:** T02 (the `FailureReason` round-trip and distinct-message tests are not written). T03 (`status_after_event`, `terminal_event_kind` and the proptests are not written, because they depend on the event types in T04).
- **Unblock:** Brighton to check which PreToolUse hook is rejecting writes in this workspace (`.kiro/hooks/` and any global hooks), and whether the rejection can be logged with a reason. Then the missing tests can be added as their own commit.

## T02: `FailureReason` tests missing

- The enum and `public_message()` exist in `crates/common/src/status.rs`. There are no tests for them yet, so T02 stays unticked.

## T03: state machine partly done

- Done: `can_transition`, `job_state_for`, five unit tests (`crates/common/src/run_state.rs`).
- Not done: `status_after_event` and `terminal_event_kind` (they need `PublicEvent`, which is T04), and the P14 proptests. T03 stays unticked.

## Not started: T04 onward

- T04 to T39 were not started in this run. The list is long, and the hook problem above was blocking test writes, so the tasks were not attempted.

## Update: status at the end of this run

- Done and verified: T01, T05, T06, T07.
- Partly done, left unticked: T02 (no `FailureReason` tests), T03 (no `status_after_event`, `terminal_event_kind` or proptests).
- Not started: T04, T08 to T39.
- Database checks run against `postgres:17-alpine` in a throwaway container. The container was stopped. The password was random and never written to a file.
- T10 (role privilege tests) is not written, so the grants are checked only with `\dp`.
- No wave-end code review was run, because Wave 0 is not complete.
- The `docs/kiro-log/2026-11-13T0938-22-run-lifecycle-implement-a1.log` file was left untracked.

## Session 3 (14 Nov run): write guard blocks all file edits

- **What happened:** every file write tool (`fs_write`, `fs_append`, `str_replace`) was refused by a PreToolUse hook with exit code 2 and no reason. The first `str_replace` on `crates/common/src/status.rs` removed a block and the second was refused, so the file was restored from git (`git checkout`, which discarded only that session's own edit).
- **Cause (confirmed):** `scripts/kiro-hooks/guard-secrets.sh write` (matcher `write`, which also matches `fs_write`) returns "probable API key detected". Its regex is `re_[A-Za-z0-9_-]{20,}` (case-insensitive), so ordinary Rust text such as long snake_case test names is flagged. A six-line Rust test sample reproduced the block with the script alone. The test-integrity guard passes the same edits.
- **What was not done:** I did not reword code to slip past the scanner, and I did not write files through the shell, because that would bypass the guard. Both are the kind of workaround the guard exists to prevent.
- **Unblock:** Brighton to anchor the key patterns so that they match only a real token boundary, for example `(^|[^A-Za-z0-9_])re_[A-Za-z0-9_-]{20,}` (and the same for `sk-` and `gsk_`), and to make the hook print its reason. After that, T02 onward can continue.
- **Tasks not done because of this:** T02 (tests), T03 (`status_after_event`, `terminal_event_kind` and the proptests), T04 onward. Every later task needs file writes.
- **Environment:** a throwaway `postgres:17-alpine` container (`cs-rl-pg`) is running on 127.0.0.1:55470 for database checks. Port 5544 was already in use by another service. Stop it with `docker stop cs-rl-pg` when finished.

## Status update: session of 13 Nov 2026 (afternoon)

- Done and verified in this session: T08 (judge library, config, queue types, testkit; 5 config unit tests), T09 (7 schema and trigger database tests, passed three times), T10 (2 role privilege tests).
- Bug fixed in T07's migration: `counterspell_server` and `counterspell_judge` role creation now also catches `unique_violation`. Two test databases migrating in parallel raced on the cluster-wide role catalogue, and the loser got `23505`. Found by T09's test run, reproduced, fixed. Migration C has not been deployed anywhere, so the file was edited in place.
- Config note: `config.rs` and its tests were written in one step, so the red phase was not observed for T08's config tests. The tests do pass.
- Stale entries above this section: T02 and T03 are now complete in the code and the tests exist. The "hook blocks writes" entries were resolved earlier in this log and do not apply now.
- Toolchain note: the cached `libsqlite3-sys-0.30.1.crate` was truncated, which broke `cargo fetch`. Only that cached archive was deleted; cargo re-downloaded it cleanly. Nothing in the project was changed for this.
- Not started: T11 (claim, SUPERVISED) onward. Those are the security-critical queue, worker, server and SSE tasks. They need a full session of their own, and this session stopped after T10 to keep the work verifiable.
- Database for tests: `postgres:17-alpine` container `cs-rl-pg` on 127.0.0.1:55470, password typed in the shell only. Stop it with `docker stop cs-rl-pg` when finished.
