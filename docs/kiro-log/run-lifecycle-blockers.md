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

## Run of 13 Nov 2026 (attempt 2): run-lifecycle T11 to T17

Done and committed this run, each with its database tests passing against a throwaway `postgres:17-alpine`: T11 claim, T12 fence and emit, T13 lease and release, T14 terminal write, T15 reaper, T16 executor seam, and T17 worker (partly, see below). The gate (fmt, clippy with `-D warnings`, workspace tests, judge `pg_` tests) was green at each commit.

### T17 is not ticked: parts still missing

- `pg_cancel_vs_complete_race` is not written. A run that has already produced a full verdict must keep it even if a cancel arrives. The fence should give that behaviour, but no test shows it yet.
- `pg_crash_recovery_real_time` is not written, and it needs a design change first. The worker's heartbeat and executor run in their own spawned tasks. Aborting the task that called `process_next` does not stop them, so the lease would keep renewing and a crash could not be simulated. The fix is to hold the heartbeat and executor handles in the worker's own task (or to let the test drop the whole runtime), then write the test.
- The claim loop polls every `CLAIM_POLL` only. The design also wants `NOTIFY run_jobs` (wake on enqueue) and `NOTIFY run_cancel` (wake the heartbeat on cancel). Not wired. Polling is correct, just slower.
- Unblock: Brighton, decide whether the crash test may restructure `Worker::execute` so the heartbeat sits in the same task as the executor. Then the remaining work is about two hours.

### Deviations recorded for review

- T11 was built after T12 because the claim's `restarted` event uses T12's append helper. T11's text allows this.
- The executor's input type is `RunInput`, not `ClaimedRun`, to avoid a name clash with the queue's `ClaimedRun`.
- `ClaimedRun` gained `mode` (T16 needs it) and `cancel_requested` (T17 needs it). Both are additive.
- `judge` lists itself under `[dev-dependencies]` with the `testkit` feature. That is what lets a plain `cargo test -p counterspell-judge` see the test kit. Cargo.lock changed only by that line.

### Not started (this run stopped at the T17 gap)

- T18 model test, T19 judge binary wiring, T20 to T39 (server API, SSE, web client, CI, DoD load test, docs). These need the next session.

## Attempt 3 (14 Nov 2026): T17 and T19

- **T17 done.** The heartbeat now runs inside the run's task, and the executor task is aborted when that task is dropped. This was needed for the crash test. The design text says "heartbeat task per run", so Brighton should confirm the change (see the review queue). The `NOTIFY run_jobs` and `NOTIFY run_cancel` wake-ups are not wired; the claim loop polls only. Polling is correct but slower.
- **T19 done with two gaps (for Brighton):**
  1. Logging is not initialised. T19 asks for `tracing` setup, but `tracing-subscriber` is not in the judge's named dependencies, so I did not add it. Until it is approved, judge logs are dropped. Unblock: approve `tracing-subscriber` for `counterspell-judge` (the server's T20 already names it).
  2. The Compose judge connects as the `counterspell` user, with no password (the Postgres service uses trust auth). The least-privilege role `counterspell_judge` is `NOLOGIN` and has no password, so it cannot log in yet. Unblock: a deploy step that creates a login for `counterspell_judge` with a secret from the environment, then point `DATABASE_URL` at it.
- **Not run:** `docker compose build judge`. Only `docker compose config` was checked.
- **T18 (model test) not started in this pass.** It is the next unchecked task after T19.

## T18: model-based interleavings (done, attempt 3)

- `pg_model_interleavings` runs 32 cases of up to 39 calls across 3 workers and 3 runs. It checks the fence's answer for each call, the job state and attempts against the model after each call, and the global invariants. It passed on the first run.
- Not measured: branch coverage. I did not instrument the test to count how often each branch (for example a `finish` refused because the run never compiled) is reached. Worth checking before relying on it as the only queue safety net.
- Proptest regression file: none was produced, because no case failed.

## Session 22 (attempt 4, 13 Nov 2026): T20 to T24 done, T25 onward not started

Done and committed, each with its gate green (fmt, clippy with `-D warnings` on the workspace, `cargo test --workspace --include-ignored` against a throwaway Postgres 17 and Valkey 8): T20 (library, config, error envelope, router), T21 (seams and testkit seams), T22 (validation with tree-sitter), T23 (Valkey limiter), T24 (enqueue, SUPERVISED).

Deviations and open points for Brighton:
- T20: `AppState` holds the pool, config, resolver, problem lookup, languages and limiter. It does not hold the `RunWaker` handle yet; that arrives with T28.
- T21: the in-memory limiter the task mentions is `MemoryLimiter` in T23 (feature `testkit`), so it is not duplicated.
- T22: the first shell-script test line used `rm -rf /tmp/work`. That is a valid Python expression, so the parser was right to accept it. The test now uses a shell `for` loop.
- T23: `valkey_outage_fails_open` is not marked `#[ignore]`. It needs no Valkey server, only an unreachable port. The other two `valkey_` tests are ignored and need `VALKEY_URL`.
- T24: a unique-index violation on `(user_id, idempotency_key)` is returned as a database error, not as a replay (design Path 1 asks for a replay). The advisory lock makes it unreachable for one user. The review queue asks Brighton to decide.
- Test dependencies: the HTTP tests send raw HTTP/1.1 over a TCP socket, so no `tower` or `reqwest` was added.
- Not started: T25 (`POST /api/runs`) and T26 to T39, including the SUPERVISED T29 (SSE core). Each needs its own session.
- A PreToolUse hook blocked one bash command (a log write combined with a git commit) without giving a reason. It was not bypassed. The log entry was written with the file tool instead. The commit of this log has not been made yet.
- Containers `cs-valkey-rl` and `cs-pg-rl2` were stopped at the end of this session.

## Attempt 5 (13 Nov 2026): T25 and T26 done, T27 partly done

- **T25 (`POST /api/runs`) done** in commit `e1ef27d`. Its 9 `pg_` tests and 2 non-database tests pass. Choices for review:
  - The per-IP key uses a new `client_ip` module. `TRUSTED_PROXIES` is parsed in config, with the design's default. Only IPv4 proxies are supported; an IPv6 peer is never trusted.
  - The replay pre-check (`existing_replay`) is a separate function, so a replayed request never reaches the rate limiter.
  - `ConnectInfo` is required, so `main` and the test server both serve with `into_make_service_with_connect_info`.
  - `queue_full` and `queue_cap` send `retry_after_seconds: 1`, which is the minimum the requirements allow.
- **T26 (`GET /api/runs/{id}`) done** in commit `c2b4e50`. The times are formatted in SQL with `to_char`, so the sqlx `time` feature is not needed. Sample details go through a three-field whitelist.
- **T27 (`POST /api/runs/{id}/cancel`) not ticked.** The endpoint and four `pg_` tests are committed and pass, including the queued case through the real judge claim. Two named tests are missing:
  - `pg_cancel_running`: needs a live claim with the heartbeat seeing the flag. The heartbeat's renewal interval has to be set short for the test, and the JudgeConfig variable for that is not in the spec. Unblock: confirm the test variable name, or allow a `real_time` test.
  - `pg_cancel_never_yields_both_done_and_error`: needs a worker that finishes while cancel is set. Same blocker.
- **Blocked on nothing else.** T28 onward was not started in this attempt.
