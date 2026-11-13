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
