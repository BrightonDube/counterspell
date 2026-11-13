# Run-lifecycle review queue (SUPERVISED tasks)

Brighton reviews each entry before the next task that depends on it starts.

## T12: fence, append-event and `emit` (commit: see `git log --oneline -1`)

- Files: `crates/judge/src/queue/fence.rs`, `crates/judge/src/queue/emit.rs`, `crates/judge/tests/emit.rs`, plus `queue/mod.rs` (`EventRejected`, token constructor) and `testkit.rs` (`force_claim`, `token_with`).
- What to check:
  - `lock_claim` locks the job row with `FOR UPDATE` and matches (job, owner, attempts). Every judge write must start with it.
  - `append_event_locked` locks the submission row, checks the status rule with `status_after_event`, then bumps `last_event_seq`. The `NOTIFY` payload is only the run id.
  - `emit` is the only public write path to `run_events`. It accepts a `PublicEvent`, never raw JSON.
  - The dev-dependency on the crate itself (`counterspell-judge` with `testkit`) is a deliberate choice so `cargo test` sees the test kit. Confirm that is acceptable.
- Gap to revisit: the concurrency test covers eight writers with one valid fence. It does not yet race a reaper-style locked writer; that belongs to T15 and the model test in T18.

## T11: claim with `FOR UPDATE SKIP LOCKED` (commit: see `git log --oneline -1`)

- Files: `crates/judge/src/queue/claim.rs`, `crates/judge/tests/claim.rs`; `ClaimToken::new` is no longer test-only.
- What to check:
  - One transaction: the CTE locks one ready job, then the submission is set to `claimed` and, when `attempts > 0`, a `restarted` event is appended through `append_event_locked`.
  - Order is cancel-first (`NULLS LAST`), then `run_after`, then `id`. Future `run_after` is skipped.
  - All times come from the database (`now()`, `make_interval`). The lease length is passed in as seconds.
- Ordering note: T11 was implemented after T12 because T11's `restarted` event uses T12's append helper. T11's own text allows this order.

## T13: lease renewal and release for retry (commit: see `git log --oneline -1`)

- Files: `crates/judge/src/queue/lease.rs`, `crates/judge/tests/lease.rs`, plus `emit.rs` changes.
- What to check:
  - `renew` matches (job, owner, attempts) and does not look at the clock, so renewal after expiry but before a reap still counts.
  - `requeue_or_exhaust_locked` is the shared body for release and the reaper (T15). It requeues with a backoff, or marks the job `dead` and emits `error` with `retries_exhausted`.
  - `append_event_locked` now sets `finished_at` and `failure_reason` for an error event, because the table's CHECK requires them in the same update.
  - `emit` now refuses `done` and `error` (test `pg_emit_refuses_terminal_events`). Only the terminal write (T14) and the retry path may end a run.
  - `sanitize_last_error` strips control characters and caps the text at 500 characters.
- Gap: `pg_lost_lease_stops_writes` covers emit, renew and release. The finish write is checked in T14.

## T14: terminal write (commit: see `git log --oneline -1`)

- Files: `crates/judge/src/queue/finish.rs`, `crates/judge/tests/finish.rs`; `emit.rs` split into `read_status_locked` and `store_event_row` helpers.
- What to check:
  - `finish` does the fence, the status rule, the result columns, the test rows, the `done` event and the job update in one transaction. The `done` payload and the columns come from the same `FinalResult`.
  - `TestRecord::Hidden` has no field for input, expected or program output, so none can be stored.
  - `finish_failed` accepts only `cancelled`, `problem_unavailable` and `language_unavailable`. Retry exhaustion and expiry have their own paths.
  - `QueueError::InvalidResult` refuses counts that do not fit an `int` column.
- Note: the atomicity test forces a duplicate ordinal to fail after the submission update. It does not inject a fault inside the code path.
