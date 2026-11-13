# Tasks: run-lifecycle

All tasks are implementation tasks for the design in `design.md` and the requirements in `requirements.md`. No task in this document is executed by the spec-generation run. Each task is sized for 30 minutes or less. `[P]` means it can run in parallel with other tasks in the same wave once its dependencies are done. `[S]` means it is sequential. `[SUPERVISED]` means Brighton reviews the diff before the next task starts (claim, lease, fence, terminal write and the SSE core are the security-critical paths).

Conventions for every Rust task:

- Follow `.kiro/steering/rust-teaching.md`, `security.md`, `api-conventions.md`, `structure.md` and (for `crates/judge/**`) `sandbox.md`. No `unwrap()` or `expect()` outside tests; no `unsafe` (the workspace forbids it).
- The workspace has Clippy pedantic on and CI runs `-D warnings`. Run `cargo clippy -p <package> --all-targets -- -D warnings` before finishing.
- Every public item gets a plain-English doc comment (`#![warn(missing_docs)]` in `common` and `judge`).
- Use runtime-checked `sqlx::query(...)` / `query_as(...)`, not the compile-time `query!` macros, so building never needs a database. Enable only the `sqlx` features that are needed (Postgres, Tokio runtime, `uuid`, `time`, `json`, `migrate`); no TLS features.
- Name every database-backed test with a `pg_` prefix and mark it `#[ignore = "needs DATABASE_URL"]`. Valkey-backed tests use `valkey_` and `VALKEY_URL`. Use `#[sqlx::test(migrations = "../../migrations")]`.
- To run database tests locally, start a throwaway Postgres 17 (for example `docker run --rm -d --name cs-pg -e POSTGRES_PASSWORD=<throwaway> -p 5432:5432 postgres:17-alpine`), export `DATABASE_URL` in your shell, and run `cargo test -p <package> -- --ignored pg_`. Never commit a URL or password, and never read or write `.env` files.
- Valkey holds counters only. Nothing in any task may use Valkey Streams, Valkey pub/sub or an in-memory channel as a queue or as the source of events.
- Do not add a Cargo package, a second server, or a dependency other than those a task names. If `cargo deny check licenses` fails after adding a dependency, stop and ask Brighton before touching `deny.toml`.
- End each Rust task with a **Rust lesson** of 3–5 lines, appended to `docs/kiro-log/rust-lessons.md` in the existing format (`# Rust lesson: run-lifecycle Txx`). The suggested topic is listed per task.
- Every new third-party crate is added to `THIRD-PARTY-NOTICES.md` in the task that adds it.

## Wave 0: shared types, state machine and schema

- [x] **T01 [S] (25 min) Prepare `counterspell-common` for shared run types**
  - Files: `crates/common/Cargo.toml`, `crates/common/src/lib.rs`, empty module files `ids.rs`, `status.rs`, `run_state.rs`, `events.rs`, `limits.rs`, `THIRD-PARTY-NOTICES.md`, `Cargo.lock`.
  - Add `serde` (with `derive`), `serde_json`, `uuid` (features `v4`, `serde`); dev-dependency `proptest` (try `default-features = false, features = ["std"]` first). Declare the modules with module docs that state what the crate must never depend on. Add `#![warn(missing_docs)]`. Keep `COMMON_VERSION`.
  - Requirements: R6.8, scope constraints.
  - Depends on: none.
  - Verify: `cargo check -p counterspell-common`; `cargo tree -p counterspell-common` shows no `sqlx`, `axum`, `tokio`, `bollard`; `cargo deny check licenses`.
  - Preserve: `common` stays free of web, database, Docker and process dependencies.
  - Rust lesson: crates, features, and why shared crates stay small.

- [x] **T02 [P] (25 min) Ids and enums**
  - Files: `crates/common/src/ids.rs`, `crates/common/src/status.rs`.
  - `RunId` and `UserId` newtypes over `Uuid` (`Display`, `FromStr`, serde transparent). Enums with `as_str`, `parse`, serde `snake_case`: `RunStatus` (nine states, `is_terminal`, `is_active`), `Verdict` (seven), `TestVerdict` (six), `FailureReason` (`cancelled`, `problem_unavailable`, `language_unavailable`, `expired`, `retries_exhausted`), `RunMode`, `JobState` (five). Export `ALL_*` constant slices. Add a fixed table `FailureReason::public_message()` of safe messages.
  - Unit tests: string round trips; `ALL_*` lists have no duplicates; serde forms equal `as_str`; invalid strings are rejected.
  - Requirements: R6.1, R6.4, R6.5, R15.5.
  - Depends on: T01.
  - Verify: `cargo test -p counterspell-common`.
  - Preserve: `Verdict` is separate from `RunStatus`; there is no verdict-like value in `RunStatus`.
  - Rust lesson: newtypes and enums.

- [x] **T03 [S] (30 min) Run state machine**
  - Files: `crates/common/src/run_state.rs`.
  - Implement the transition table from `design.md` as `can_transition(from, to) -> bool` (same-status is allowed), `status_after_event(current, &PublicEvent kind)`, `job_state_for(status)`, and `terminal_event_kind(status)`. Use exhaustive `match` with no wildcard arm so a new state fails to compile.
  - Proptest (P14): terminal states have no exit; every reachable transition appears in the table; a random walk over allowed moves never leaves the table; `job_state_for` is total and matches the mapping table; `RunStatus::is_active` equals "`job_state_for` is `claimed`".
  - Requirements: R6.2, R6.3, R6.7, R6.8, P14.
  - Depends on: T02.
  - Verify: `cargo test -p counterspell-common run_state`.
  - Preserve: this is the only Rust definition of the table; the server and judge call it and do not copy it.
  - Rust lesson: exhaustive `match` and why a missing arm is a compile error.

- [x] **T04 [P] (30 min) Public event types and bounded payloads**
  - Files: `crates/common/src/events.rs`, `crates/common/src/limits.rs`.
  - Define `PublicTest::{Hidden, Sample}`, `SampleDetail`, `PublicEvent` (all eleven kinds, with `event_type() -> &'static str` returning exactly the names in `api-conventions.md`), `DonePayload` (including optional `compile_output` capped at 8 KiB), `ErrorPayload { code, message }` built only from `FailureReason`, `BoundedJson` (object only, at most 16 KiB, built only from a `Serialize` value), the SSE envelope `{seq, attempt, payload}`, and `MAX_PAYLOAD_BYTES`. There is no constructor from raw JSON for `Test`, `Done` or `Error`.
  - Unit tests: a hidden test serializes to exactly the keys `ordinal`, `verdict`, `time_ms`, `sample`; a sample test adds `input`, `expected`, `got`; `BoundedJson` rejects arrays, scalars and oversized objects; the `event_type` set equals the eleven allowed names (copy the list into the test from the steering file); error messages come only from the fixed table. Proptest: any generated `PublicEvent` serializes to a JSON object under the cap.
  - Requirements: R15.1, R15.2, R15.5, R15.6, R11.5, P12.
  - Depends on: T02.
  - Verify: `cargo test -p counterspell-common events`.
  - Preserve: a hidden test has no field that can hold an input or expected value.
  - Rust lesson: making illegal states unrepresentable with enums.

- [x] **T05 [S] (30 min) Migration A: tables, constraints and indexes**
  - Files: `migrations/<timestamp>_run_lifecycle_tables.sql` (timestamp later than `20261112020000`).
  - Create `submissions`, `jobs`, `run_events`, `test_results` exactly as in `design.md` "Data contract", with all `CHECK` constraints, the unique and partial indexes, and `ON DELETE CASCADE` from `jobs`, `run_events` and `test_results` to `submissions`. No foreign keys to `users` or `problems` (open question 1). Idempotent where cheap; plain `CREATE TABLE` is fine because SQLx records applied migrations.
  - Requirements: R2.4, R3.4, R6.7, R11.5, R11.6, R15.4.
  - Depends on: none (can run in parallel with Wave 0 Rust tasks).
  - Verify: apply to a scratch Postgres 17 with `sqlx migrate run --source migrations`; inspect with `psql -c '\d+ submissions'` and `\d+ jobs`; re-running is a no-op.
  - Preserve: the bootstrap migration is untouched; Familiar, ghost and other columns are not added.
  - Rust lesson: none (no Rust changed). Add a one-line note in the lessons file saying schema is the contract that Rust code relies on.

- [x] **T06 [S] (30 min) Migration B: transition function and freeze trigger**
  - Files: `migrations/<timestamp>_run_lifecycle_guards.sql`.
  - Create `cs_status_transition_allowed(from text, to text) RETURNS boolean IMMUTABLE` from the transition table, and the `BEFORE UPDATE` trigger on `submissions` that (1) freezes a terminal run, (2) rejects a disallowed transition, (3) rejects a decreasing `last_event_seq`. Use the error code `check_violation` and short messages that contain no data.
  - Requirements: R6.3, R10.3, P6, P14.
  - Depends on: T05.
  - Verify: apply to a scratch Postgres; run two manual updates (an allowed one and a forbidden one) with `psql`. Automated tests come in T09.
  - Preserve: the SQL table must equal the Rust table in T03 (T09 compares all 81 pairs).
  - Rust lesson: none (no Rust changed); note that the database now enforces the same rules as the Rust types.

- [x] **T07 [P] (25 min) Migration C: roles and grants**
  - Files: `migrations/<timestamp>_run_lifecycle_roles.sql`.
  - Create `counterspell_server` and `counterspell_judge` as `NOLOGIN` roles inside a `DO` block that ignores `duplicate_object`. Grant exactly the privileges in the matrix in `design.md`, using column-level `UPDATE` grants. Revoke default `PUBLIC` privileges on the four tables. No passwords, no `LOGIN`.
  - Requirements: R19.1–R19.3.
  - Depends on: T05.
  - Verify: apply to a scratch Postgres; `\dp submissions jobs run_events test_results` shows the matrix. Automated tests come in T10.
  - Preserve: the server role cannot insert events or results or update submissions.
  - Rust lesson: none (no Rust changed); note least privilege.

## Wave 1: judge queue core

- [x] **T08 [S] (30 min) Make the judge a library and binary, add config, errors and the test harness**
  - Files: `crates/judge/Cargo.toml`, `crates/judge/src/lib.rs`, `crates/judge/src/main.rs` (thin), `crates/judge/src/config.rs`, `crates/judge/src/queue/mod.rs`, `crates/judge/src/testkit.rs`, `THIRD-PARTY-NOTICES.md`.
  - Add `sqlx`, `tokio` (`macros`, `rt-multi-thread`, `sync`, `time`, `signal`), `tracing`, `uuid`, `serde_json`, `thiserror`, and `counterspell-common`. Add features `testkit` (off by default) and `sandbox` (default on, empty for now; judge-sandbox adds `bollard` behind it). `JudgeConfig` loads the judge variables from `design.md` "Configuration" with defaults and returns a clear error for invalid values. `QueueError` has `LeaseLost`, `NotFound`, `Database(sqlx::Error)`. `ClaimToken { job_id, run_id, lease_owner, attempts }`. `testkit` provides `seed_user_and_run(pool)` (inserts a submission and job as a superuser), `age_lease(pool, run)`, `age_created(pool, run, secs)`, and an `assert_invariants(pool)` checker for P1, P3, P6, P7, P8 (the checker is reused by every later test).
  - Requirements: R7.5, R8.5, scope constraints.
  - Depends on: T01.
  - Verify: `cargo check -p counterspell-judge` and `cargo check -p counterspell-judge --features testkit`; `cargo tree -p counterspell-judge` has no `bollard`; `cargo test --workspace` still passes without a database.
  - Preserve: the judge binary still starts as a placeholder; nothing talks to Docker.
  - Rust lesson: library plus binary targets, features, and `Result` with a custom error type.

- [x] **T09 [S] (30 min) Schema, trigger and agreement tests**
  - Files: `crates/judge/tests/schema.rs`.
  - Tests: `pg_check_constraints_reject_bad_rows` (bad status, verdict without completion, terminal without `finished_at`, hidden `test_results` row with detail, payload over 16 KiB, unknown event type); `pg_unique_job_per_submission`; `pg_one_terminal_event_per_run`; `pg_terminal_row_is_frozen` (update status, verdict, counts, `finished_at`, `user_id`, `code_sha256` after finishing → error); `pg_transition_guard_rejects_illegal_move`; `pg_status_transition_agreement` (all 81 pairs: `common::run_state::can_transition` equals `cs_status_transition_allowed`); `pg_idempotency_index_is_per_user`.
  - Requirements: R6.3, R10.3, R15.4, P6, P7, P14.
  - Depends on: T03, T04, T06, T08.
  - Verify: `cargo test -p counterspell-judge --test schema -- --ignored pg_`.
  - Preserve: tests use the admin pool and do not rely on role grants.
  - Rust lesson: async tests and `#[sqlx::test]`.

- [x] **T10 [P] (25 min) Role privilege tests**
  - Files: `crates/judge/tests/roles.rs`.
  - `pg_role_privileges`: inside a transaction, `SET LOCAL ROLE counterspell_server` and assert it can insert a submission and job, read all four tables, and set `cancel_requested_at`; and that it cannot insert `run_events` or `test_results`, update `submissions`, or change `state`, `attempts` or `lease_*`. Repeat for `counterspell_judge` (can claim, update allowed columns, insert events and results; cannot delete, cannot change `user_id`, `code` or `code_sha256`, cannot insert `jobs`). Use `SAVEPOINT` so one failure does not abort the transaction.
  - Requirements: R19.2–R19.4.
  - Depends on: T07, T08.
  - Verify: `cargo test -p counterspell-judge --test roles -- --ignored pg_`.
  - Preserve: grants are not widened to make a test pass; fix the test or ask.
  - Rust lesson: transactions, savepoints and what happens on drop.

- [x] **T11 [S] [SUPERVISED] (30 min) Claim with `FOR UPDATE SKIP LOCKED`**
  - Files: `crates/judge/src/queue/claim.rs`, `crates/judge/tests/claim.rs`.
  - Implement `claim_next(pool, worker_id, lease) -> Result<Option<ClaimedRun>>` as in `design.md` Path 2: one transaction; eligible rows, cancel-first ordering, lease from the database clock, `claimed_at`, submission to `claimed` with `started_at`, load of the run fields, and the `restarted` event when `attempts > 0` (the event append calls the helper from T12; stub it behind a trait or order T11 after T12 if simpler, but keep the claim and the `restarted` event in one transaction). Add `NOTIFY` listening is **not** part of this task.
  - Tests: `pg_claim_exclusive_under_contention` (8 concurrent claimers, 50 jobs, every job claimed exactly once, none lost); `pg_claim_order_and_run_after` (FIFO, future `run_after` skipped); `pg_claim_prefers_cancelled`; `pg_duplicate_claim_attempts` (a job already claimed is not claimed again); `pg_claim_sets_submission_state`.
  - Requirements: R7.1–R7.4, R7.7, R11.4, P3.
  - Depends on: T08, T12 (or implement together with T12).
  - Verify: `cargo test -p counterspell-judge --test claim -- --ignored pg_`.
  - Preserve: all timestamps come from the database; lock order is jobs, then submissions.
  - Rust lesson: borrowing a connection pool across tasks (`Arc`, `Clone` on `PgPool`).

- [x] **T12 [S] [SUPERVISED] (30 min) Fence, append-event and `emit`**
  - Files: `crates/judge/src/queue/fence.rs`, `crates/judge/src/queue/emit.rs`, `crates/judge/tests/emit.rs`.
  - Implement the fenced transaction helper (job row `FOR UPDATE` with owner and attempts), `append_event_locked(tx, run, attempt, &PublicEvent)` (counter increment with status change from `status_after_event`, insert, `pg_notify('run_events', run_id)`), and `emit(pool, &ClaimToken, &PublicEvent)`. Ensure the events table is written only through `PublicEvent`; no function accepts raw JSON.
  - Tests: `pg_emit_assigns_gapless_seq`; `pg_emit_changes_status_with_event` (`compiling`, first `test`, `ghost_hunting`); `pg_emit_with_wrong_owner_or_stale_attempt_writes_nothing` (P4); `pg_event_seq_gapless_under_concurrency` (old and new holder and a reaper-style locked writer racing; the final set of `seq` is exactly `1..n`; P8); `pg_no_append_after_terminal`.
  - Requirements: R10.1, R11.1–R11.3, R11.7, P4, P8.
  - Depends on: T04, T06, T08.
  - Verify: `cargo test -p counterspell-judge --test emit -- --ignored pg_`.
  - Preserve: `NOTIFY` carries only the run id and is sent in the same transaction.
  - Rust lesson: `sqlx` transactions, `?`, and rollback on early return.

- [x] **T13 [S] [SUPERVISED] (30 min) Lease renewal and release for retry**
  - Files: `crates/judge/src/queue/lease.rs`, `crates/judge/tests/lease.rs`.
  - Implement `renew(pool, &ClaimToken, lease) -> Result<Renewal { cancel_requested }>` (fenced update, no row → `LeaseLost`), and `release_for_retry(pool, &ClaimToken, reason)` which, under the fence, requeues with `attempts + 1` and backoff or, when `attempts + 1 >= max_attempts`, marks the job dead and the run `internal_error`/`retries_exhausted` with its `error` event. Put the shared requeue-or-exhaust transaction body in one function used by both this task and the reaper (T15). Sanitise and truncate `last_error` to 500 characters.
  - Tests: `pg_lease_renewal_extends_and_reports_cancel`; `pg_renew_after_expiry_but_before_reap_succeeds`; `pg_renew_after_reap_fails`; `pg_lost_lease_stops_writes` (P4, after the lease is reaped, emit, renew and finish all return `LeaseLost`); `pg_reclaim_by_same_worker_gets_new_fence`; `pg_release_requeues_with_backoff`; `pg_release_exhausts_to_internal_error`.
  - Requirements: R8.1, R8.2, R8.4, R9.3, R9.4, R9.8, R9.9, P4, P5.
  - Depends on: T11, T12.
  - Verify: `cargo test -p counterspell-judge --test lease -- --ignored pg_`.
  - Preserve: `attempts` never exceeds `max_attempts`.
  - Rust lesson: `Result` error variants for expected outcomes (`LeaseLost` is not a crash).

- [x] **T14 [S] [SUPERVISED] (30 min) Terminal write**
  - Files: `crates/judge/src/queue/finish.rs`, `crates/judge/tests/finish.rs`.
  - Implement `finish(pool, &ClaimToken, FinalResult)` and `finish_failed(pool, &ClaimToken, FailureReason)` as in `design.md` Path 5. `FinalResult` holds verdict, counts, score, timings, and the list of `TestRecord` (`Sample` with detail, `Hidden` without). The `done` payload and the stored columns are built from the same value. Hidden records must have null snippets (the type has no place for them).
  - Tests: `pg_terminal_write_exactly_once` (two tasks race to finish with the same fence; exactly one succeeds, one terminal event, `test_results` once); `pg_stale_owner_finish_rejected` (P4); `pg_finish_writes_done_event_matching_columns`; `pg_finish_failed_cancelled_shape`; `pg_second_terminal_event_rejected` (direct SQL); `pg_finish_is_atomic` (fault injected after `test_results` insert leaves nothing).
  - Requirements: R10.2–R10.4, R10.7, R16.4, P6, P7, P15.
  - Depends on: T12, T13.
  - Verify: `cargo test -p counterspell-judge --test finish -- --ignored pg_`.
  - Preserve: `test_results` is written only here.
  - Rust lesson: why one transaction beats several careful statements.

- [x] **T15 [S] (30 min) Reaper and queue expiry**
  - Files: `crates/judge/src/queue/reap.rs`, `crates/judge/tests/reap.rs`.
  - Implement `reap_once(pool, cfg) -> ReapReport` covering expired leases, the maximum claim age, cancelled-and-orphaned jobs, retries exhausted, and queue TTL expiry, as in `design.md` Path 6, using `FOR UPDATE SKIP LOCKED` and one transaction per job. Add `run_reaper(pool, cfg, shutdown)` that runs at start-up and every `REAPER_INTERVAL`.
  - Tests: `pg_reaper_requeues_expired_lease`; `pg_reaper_marks_dead_after_max_attempts` (P5, P18); `pg_reaper_two_reapers_do_not_double_count` (concurrent `reap_once`; `attempts` rises by exactly 1); `pg_max_claim_age_reclaims_stuck_run`; `pg_expiry_of_stale_queue` (P17, never executed); `pg_reaper_finishes_cancelled_orphan`; `pg_reaper_does_not_touch_live_leases`.
  - Requirements: R9.1–R9.8, R9.10, P5, P17.
  - Depends on: T13, T14.
  - Verify: `cargo test -p counterspell-judge --test reap -- --ignored pg_`.
  - Preserve: no `sleep` in these tests except where the test is named `real_time`; leases are aged with SQL.
  - Rust lesson: periodic tasks with `tokio::time::interval` and shutdown signals.

- [ ] **T16 [S] (30 min) Executor trait, run context and the scripted executor**
  - Files: `crates/judge/src/executor.rs`, `crates/judge/src/testkit.rs`.
  - Define `ClaimedRun`, `RunOutcome`, `ExecError`, the `RunExecutor` trait (async fn in trait, no `dyn`), and `RunContext` (`emit`, `stop_requested`, `stopped`, `run_id`, `attempt`), exactly as sketched in `design.md` Path 7. In `testkit`, add `ScriptedExecutor` with behaviours `Pass { tests, delay }`, `Fail { verdict }`, `Hang`, `Infra(msg)`, `Panic`, `PassWithCanaries` (hidden inputs and expected outputs contain canary strings that the executor tries to place in free-text fields; only typed public constructors may carry data out) and `CrashAfter(n)` (stops after `n` events without finishing).
  - Requirements: R15.1, R18.1, R18.4.
  - Depends on: T04, T12.
  - Verify: `cargo test -p counterspell-judge executor` (unit tests with a fake context that records events); `cargo clippy -p counterspell-judge --all-targets --features testkit -- -D warnings`.
  - Preserve: the context offers no way to write a verdict, status or job state.
  - Rust lesson: traits with `async fn` and why generics are used instead of `dyn`.

- [ ] **T17 [S] [SUPERVISED] (30 min) Worker loop, heartbeat, panic containment and shutdown**
  - Files: `crates/judge/src/worker.rs`, `crates/judge/tests/worker.rs`.
  - Implement `Worker<E: RunExecutor>` per `design.md` Path 7: semaphore of `JUDGE_CONCURRENCY` slots; claim loop that waits on `NOTIFY run_jobs` or `CLAIM_POLL`; heartbeat task per run that renews every `RENEW` and also on `NOTIFY run_cancel`; stop signal on a `watch` channel; executor spawned in its own task so a panic is a `JoinError`; hard deadline from `MAX_CLAIM_AGE`; local lease-lost detection when renewals fail for a full lease period; outcome mapping to `finish`, `finish_failed`, `release_for_retry`; graceful shutdown with `SHUTDOWN_GRACE`. A job with a pending cancel is finished as `cancelled` without calling the executor.
  - Tests: `pg_worker_runs_scripted_job_to_completion`; `pg_worker_concurrency_limit_respected`; `pg_executor_infrastructure_error_retries` and `pg_executor_panic_is_contained` (P18; other slots keep working); `pg_worker_hard_deadline_releases_stuck_executor`; `pg_cancel_queued_never_invokes_executor`; `pg_cancel_running_stops_executor` (cancel set through SQL as the server would); `pg_cancel_vs_complete_race` (verdict wins once complete); `pg_graceful_shutdown_releases_job`; `pg_crash_recovery_real_time` (abort a worker task mid-run with a 2 s lease and 300 ms reaper; a second worker finishes; `restarted` is the first event of attempt 2; exactly one terminal result; P15).
  - Requirements: R7.5, R7.6, R8.1–R8.3, R9.9, R17.3, R17.4, R17.5, R18, P15, P16, P18.
  - Depends on: T11–T16.
  - Verify: `cargo test -p counterspell-judge --test worker -- --ignored pg_`.
  - Preserve: the worker never writes a verdict itself; it only calls `finish` with the executor's value.
  - Rust lesson: `tokio::spawn`, `JoinError`, `select!` and channels.

- [ ] **T18 [S] (30 min) Model-based interleaving test**
  - Files: `crates/judge/tests/model.rs`.
  - `pg_model_interleavings` with `proptest` as described in `design.md` ("Model-based test"): operations `enqueue`, `claim(w)`, `renew(w)`, `emit(w)`, `finish(w)`, `expire_lease`, `reap`, `cancel` across 2–3 workers and 1–4 runs, including deliberate stale-holder calls. After every step run `assert_invariants` (P3–P8, P14) and a fence-oracle: the model predicts which calls must report `LeaseLost`. 32 cases, a small Tokio runtime per case, tables reset per case. Commit the proptest regression file if one appears.
  - Requirements: P3, P4, P5, P6, P7, P8, P14.
  - Depends on: T11–T15.
  - Verify: `cargo test -p counterspell-judge --test model -- --ignored pg_`.
  - Preserve: lease ageing uses SQL, not sleeping.
  - Rust lesson: property-based testing with an operation model.

- [ ] **T19 [S] (25 min) Wire the judge binary and its deployment settings**
  - Files: `crates/judge/src/main.rs`, `docker-compose.yml` (judge service only), `README.md` (one short section).
  - `main` loads `JudgeConfig`, initialises `tracing` (no source code, no hidden data in logs), connects to Postgres with a small pool, starts the reaper and a `Worker` with a placeholder `NoExecutor` that returns `ExecError::Infrastructure("no executor configured")`, so any real job ends honestly as `internal_error` after its retries and is never reported as a verdict (the judge-sandbox spec replaces it; production cannot enqueue runs until auth and the catalogue are wired anyway). Handle `SIGTERM` and `SIGINT`. In Compose set the judge `restart: unless-stopped` and pass the lease and concurrency variables; keep the service without ports and without a Docker socket.
  - Requirements: R9.1, R9.9, R1.6, R19.1.
  - Depends on: T17.
  - Verify: `cargo build -p counterspell-judge`; `docker compose config` shows the judge without `ports:` or socket mounts and with `restart: unless-stopped`.
  - Preserve: the judge needs no Valkey URL and no secret other than its database credentials.
  - Rust lesson: signal handling and graceful shutdown in `main`.

## Wave 2: server API

- [ ] **T20 [S] (30 min) Turn the server into a library and binary; add state, config and the JSON error envelope**
  - Files: `crates/server/Cargo.toml`, `crates/server/src/lib.rs`, `crates/server/src/main.rs`, `crates/server/src/config.rs`, `crates/server/src/error.rs`, `THIRD-PARTY-NOTICES.md`.
  - Move router assembly into `lib.rs` (`build_router(AppState)`), keep `/api/health`, the static files and the SPA fallback working with their existing precedence (unknown `/api/*` stays a JSON `404`, never HTML). Add `AppState` (pool, limiter, resolver, problem lookup, language catalogue, config, run waker handle). `ApiError` with stable `code`, safe `message`, optional `retry_after_seconds`, `IntoResponse`; 500s are logged and return a generic body. Add dependencies `sqlx`, `redis`, `tracing`, `tracing-subscriber`, `uuid`, `sha2`, `thiserror`, `tower-http` `limit` and `set-header` features, `counterspell-common`; add feature `testkit` (off) and dev-dependency on `counterspell-judge` with `default-features = false, features = ["testkit"]`.
  - Requirements: R4.1, scope constraints, `api-conventions.md`.
  - Depends on: T01.
  - Verify: `cargo test -p counterspell-server` (existing health and static tests still pass); `cargo tree -p counterspell-server -e normal` has no `bollard` and no `counterspell-judge`.
  - Preserve: the existing skeleton behaviour and its tests.
  - Rust lesson: `IntoResponse` and turning errors into HTTP replies safely.

- [ ] **T21 [P] (25 min) Seams: user resolver, problem lookup, language catalogue, and their test versions**
  - Files: `crates/server/src/seams.rs`, `crates/server/src/testkit.rs`.
  - Traits `UserResolver` (`resolve(&HeaderMap) -> Option<UserId>`) with an axum extractor `CurrentUser`; `ProblemLookup` (`published(slug) -> Option<ProblemRef>`); `LanguageCatalog` (enabled languages and `max_source_bytes`, defaults for `python` and `javascript`). Production implementations: `Unauthenticated` and `EmptyCatalogue`. In `testkit` (feature-gated): `BearerTestResolver` (`Authorization: Bearer test-user-<n>`), `InMemoryProblems`, and the in-memory rate limiter from T23.
  - Requirements: scope constraints, seams table.
  - Depends on: T20.
  - Verify: `cargo test -p counterspell-server seams`; a unit test proves the production resolver always returns `None`.
  - Preserve: no header-based test authentication exists outside the `testkit` feature.
  - Rust lesson: traits as seams, and extractors in axum.

- [ ] **T22 [P] (30 min) Submission validation (VAL-1, VAL-2, VAL-3)**
  - Files: `crates/server/src/validate.rs`, `crates/server/Cargo.toml`, `THIRD-PARTY-NOTICES.md`.
  - Pure function `validate_source(lang, code, limits) -> Result<(), ValidationError>`: byte limit with a message that states limit and actual size; no NUL or control characters other than tab, newline, carriage return; tree-sitter parse with the Python or JavaScript grammar, reporting the first error line and column. Add `tree-sitter`, `tree-sitter-python`, `tree-sitter-javascript` (check licences; MIT expected). No keyword or API blocklist.
  - Unit tests and proptest: oversize by one byte; multi-byte characters counted in bytes not characters; NUL and `\x1b` rejected; tab, `\n`, `\r\n` accepted; valid and invalid programs in both languages; a very deeply nested expression does not panic or hang; random byte soup never panics.
  - Requirements: R4.3–R4.7, VAL-1, VAL-2, VAL-3.
  - Depends on: T20.
  - Verify: `cargo test -p counterspell-server validate`; `cargo deny check licenses`.
  - Preserve: this is a first filter only; the sandbox remains the security boundary.
  - Rust lesson: parsing untrusted input without panicking.

- [ ] **T23 [P] (30 min) Rate limiter (Valkey windows) and the in-memory test limiter**
  - Files: `crates/server/src/ratelimit.rs`.
  - Trait `RateLimiter` (`hit(key, limit, window) -> Result<Decision>`), a Valkey implementation using one atomic script (increment, set expiry on first hit, return count and remaining TTL) with namespaced keys `rl:runs:user:<id>` and `rl:runs:ip:<hash of ip>`, and an in-memory implementation (feature `testkit`). On a Valkey error, `Decision::SkippedFailOpen` plus a `tracing` warning.
  - Tests: `valkey_window_limits` (limit, reset, `retry_after_seconds >= 1`); `valkey_outage_fails_open`; `valkey_keys_hold_only_counters` (scan keys and check types and values, P13); unit tests for the in-memory limiter.
  - Requirements: R5.1, R5.2, R5.6, R5.8, P13.
  - Depends on: T20.
  - Verify: `cargo test -p counterspell-server ratelimit`; with `VALKEY_URL`: `cargo test -p counterspell-server -- --ignored valkey_`.
  - Preserve: Valkey stores counters only.
  - Rust lesson: async trait objects versus generics for a shared service.

- [ ] **T24 [S] [SUPERVISED] (30 min) Enqueue transaction**
  - Files: `crates/server/src/runs/enqueue.rs`, `crates/server/tests/enqueue.rs`.
  - Implement `enqueue(pool, NewRun) -> Result<EnqueueOutcome { Created | Replayed | Reuse | CapReached | QueueFull }>` exactly as `design.md` Path 1 step 9: advisory lock per user, authoritative replay check, active-job count from `jobs`, global depth check, inserts, `pg_notify('run_jobs')`, one transaction.
  - Tests: `pg_enqueue_is_atomic` (fault injection after the submission insert; no rows remain; P1); `pg_idempotency_sequential`; `pg_idempotency_concurrent` (10 simultaneous identical requests, one run, one job); `pg_idempotency_reuse_is_422_case`; `pg_idempotency_keys_are_per_user`; `pg_active_cap_under_concurrent_posts` (10 concurrent requests from one user with cap 3, exactly 3 created; P13); `pg_global_queue_depth_cap`.
  - Requirements: R2.1–R2.5, R3.2–R3.5, R5.3–R5.5, P1, P2, P13.
  - Depends on: T05, T20.
  - Verify: `cargo test -p counterspell-server --test enqueue -- --ignored pg_`.
  - Preserve: a replay never consumes quota or cap.
  - Rust lesson: transactions that roll back automatically when a value is dropped.

- [ ] **T25 [S] (30 min) `POST /api/runs` handler**
  - Files: `crates/server/src/runs/create.rs`, `crates/server/tests/post_runs.rs`.
  - Implement the check order from `design.md` Path 1: body limit and content type, strict JSON (`deny_unknown_fields`), `CurrentUser`, `Idempotency-Key` syntax, language and mode, `validate_source`, `ProblemLookup`, replay pre-check, rate-limit windows (user then IP; IP from `TRUSTED_PROXIES` rules), then `enqueue`. Return `202 {run_id, status}` with `Location`, and `Idempotent-Replayed: true` on replays. Map each `EnqueueOutcome` to the documented error.
  - Tests: `validation_rejects_without_side_effects` (every VAL case → correct status, no rows, no quota used); `pg_post_requires_session`; `pg_post_unknown_problem_404`; `pg_post_rate_limited_has_retry_after`; `pg_post_replay_header`; `pg_post_trusted_proxy_ip` (header ignored from untrusted peers).
  - Requirements: R3.1, R3.6, R4, R5, R2.2, R2.6.
  - Depends on: T21–T24.
  - Verify: `cargo test -p counterspell-server --test post_runs -- --ignored pg_`.
  - Preserve: errors contain no internals; rejected requests create nothing.
  - Rust lesson: extractors, ordering of checks, and early returns with `?`.

- [ ] **T26 [S] (25 min) `GET /api/runs/{id}` read model**
  - Files: `crates/server/src/runs/read.rs`, `crates/server/tests/get_run.rs`.
  - Owner-checked read of the submission and its `test_results` (public detail only), returning the JSON in `design.md` Path 10. A missing id, a malformed id and a foreign id all return the same `404 not_found`. Terminal responses are built only from final columns.
  - Tests: `pg_get_run_requires_session`; `pg_get_run_foreign_and_missing_are_identical_404`; `pg_get_run_nonterminal_shape`; `get_run_after_reload_matches_stream` (finish a scripted run; `GET` equals the `done` payload; two `GET`s are byte-identical; P12 hidden tests have no detail).
  - Requirements: R16, R15.3, P11, P12.
  - Depends on: T14, T25.
  - Verify: `cargo test -p counterspell-server --test get_run -- --ignored pg_`.
  - Preserve: absent sections stay absent, never fabricated.
  - Rust lesson: serialising structs with `serde` and optional fields.

- [ ] **T27 [S] (25 min) `POST /api/runs/{id}/cancel`**
  - Files: `crates/server/src/runs/cancel.rs`, `crates/server/tests/cancel.rs`.
  - Implement Path 8: owner-scoped `UPDATE jobs … cancel_requested_at`, `NOTIFY run_cancel` and `run_jobs`, `202` or `200`, common `404`. Require `Content-Type: application/json`. Cancel consumes no quota.
  - Tests with the real worker (T17) and the scripted executor: `pg_cancel_queued`, `pg_cancel_running`, `pg_cancel_after_done` (no change, `200`), `pg_cancel_foreign` (`404`, no flag set), `pg_cancel_twice`, `pg_cancel_never_yields_both_done_and_error`.
  - Requirements: R17, P11, P16.
  - Depends on: T17, T25.
  - Verify: `cargo test -p counterspell-server --test cancel -- --ignored pg_`.
  - Preserve: the server only sets the flag; the judge makes the state change.
  - Rust lesson: sharing a database pool between handlers and tests.

- [ ] **T28 [S] (25 min) `RunWaker`: one listener for wake-ups**
  - Files: `crates/server/src/runs/wake.rs`, `crates/server/tests/wake.rs`.
  - One `PgListener` task on channel `run_events`. It sends `RunId`s on a `tokio::sync::broadcast` channel (capacity 1024), reconnects after errors with a delay, and sends a "wake everyone" marker after each reconnect. A switch `WakerMode::Disabled` (feature `testkit`) turns it off for the poll-only tests. Streams subscribe through a small handle.
  - Tests: `pg_waker_delivers_run_id_only` (the broadcast item contains no event data); `pg_waker_reconnect_sends_wake_all`; `pg_waker_lag_is_harmless`.
  - Requirements: R12.5, R13.3, R13.4.
  - Depends on: T12, T20.
  - Verify: `cargo test -p counterspell-server --test wake -- --ignored pg_`.
  - Preserve: wake-ups are hints only; nothing in this module is read as an event.
  - Rust lesson: `broadcast` channels and what happens to slow receivers.

- [ ] **T29 [S] [SUPERVISED] (30 min) SSE endpoint core: authorize, replay, wait, close**
  - Files: `crates/server/src/runs/sse.rs`.
  - Implement Path 9 steps 1–3 and 5–6: session, cursor parsing (header or `last_event_id` query; `400` on invalid), the single owner-checked read of `last_event_seq` and `finished_at` before any event query, subscribe before the first query, replay in batches of 200 with the owner join in every query, wait loop on wake-up or `SSE_POLL_MS`, send `id`, `event` and the JSON envelope, close after the terminal event. Hold no connection while waiting.
  - Requirements: R12.1–R12.5, R13.1–R13.5, R13.6 (close part), R13.9, P9, P11.
  - Depends on: T26, T28.
  - Verify: `cargo test -p counterspell-server sse` (unit tests for cursor parsing and the owner-join query text); end-to-end tests arrive in T31.
  - Preserve: the ownership check precedes every event read; event queries always include the user.
  - Rust lesson: streams, `async` loops and `select!`.

- [ ] **T30 [S] (25 min) SSE hardening: heartbeat, headers, `204`, limits, max age**
  - Files: `crates/server/src/runs/sse.rs`, `crates/server/src/runs/streams.rs`.
  - Add the 15 s comment heartbeat, the headers (`Cache-Control: no-store`, `X-Accel-Buffering: no`, `X-Content-Type-Options: nosniff`), `204 No Content` when the cursor is at or beyond the terminal `seq`, the per-user stream limit with a drop guard (`429 too_many_streams`), and the maximum stream age.
  - Requirements: R12.6, R12.7, R13.6, R13.7, R13.8.
  - Depends on: T29.
  - Verify: `cargo test -p counterspell-server sse`; tests in T31 cover the behaviour end to end.
  - Preserve: a dropped connection frees its slot even on abrupt close.
  - Rust lesson: the `Drop` trait as a cleanup guarantee.

- [ ] **T31 [S] (30 min) SSE test client and integration tests**
  - Files: `crates/server/tests/support/mod.rs` (loopback server starter, test SSE client, run helpers), `crates/server/tests/sse.rs`.
  - Test client: a hand-written SSE parser on a hyper client (already in the tree through axum; add `hyper-util` / `http-body-util` as dev-dependencies only if licence-clean) that records `id`, `event`, `data`, comment lines and receive times. Tests: `sse_requires_session_and_ownership` (anonymous, other user, missing, malformed; P11); `sse_replay_after_drop_at_every_position` (P9); `sse_poll_only_without_notify` (P9, within 1 s); `sse_last_event_id_forms` (header, query, invalid, beyond-end `204`); `sse_heartbeat_headers_and_close`; `sse_stream_limit_per_user`; `sse_max_age_closes_and_resumes`; `sse_restarted_event_visible_after_retry` (scripted crash then retry shows `restarted`).
  - Requirements: R12, R13, R14.2, R22.3, R22.6, P9, P11.
  - Depends on: T17, T30.
  - Verify: `cargo test -p counterspell-server --test sse -- --ignored pg_`.
  - Preserve: the test client never trusts `NOTIFY`; it only reads the HTTP stream.
  - Rust lesson: testing real network code with ephemeral ports.

## Wave 3: browser client logic (no screens)

- [ ] **T32 [P] (30 min) Add vitest and the pure event reducer**
  - Files: `web/package.json`, `web/package-lock.json`, `web/src/lib/runs/types.ts`, `web/src/lib/runs/reducer.ts`, `web/src/lib/runs/reducer.test.ts`, `THIRD-PARTY-NOTICES.md`.
  - Add `vitest` as a dev dependency and an `npm test` script (`vitest run`). Implement `apply(state, event)` as in `design.md` Path 11. Tests (P10): duplicates ignored; stale `seq` ignored; a gap sets `gap`; `restarted` clears per-attempt state and keeps `lastSeq`; terminal events recorded; a seeded random generator inserts arbitrary duplicates and replays into the log and the final state equals applying the log once (200 iterations, fixed seeds).
  - Requirements: R14.1–R14.3, R14.5, P10.
  - Depends on: none.
  - Verify: `cd web && npm ci && npm run check && npm test && npm run build`.
  - Preserve: follows `.kiro/steering/frontend.md`; no UI, no GSAP, no colour decisions; server text is stored as plain strings.
  - Rust lesson: none (no Rust changed).

- [ ] **T33 [S] (30 min) Stream client**
  - Files: `web/src/lib/runs/stream.ts`, `web/src/lib/runs/stream.test.ts`, `web/src/lib/runs/api.ts`.
  - `startRun`, `getRun` and `watchRun`. `watchRun` wraps `EventSource`, feeds the reducer, closes on the terminal event, closes and reopens with `?last_event_id=` on a gap, treats a `204` or an immediate close of a terminal run as "fetch the result", then calls `GET /api/runs/{id}`. `startRun` creates one `Idempotency-Key` per user action and reuses it for retries of the same action. Test with a fake `EventSource`: drop and resume, duplicate delivery, gap recovery, terminal close prevents reconnect, a `429` is surfaced with `retry_after_seconds`.
  - Requirements: R14, R3.1.
  - Depends on: T32.
  - Verify: `cd web && npm run check && npm test && npm run build`.
  - Preserve: logic only; screens belong to the ide-workspace spec.
  - Rust lesson: none (no Rust changed).

## Wave 4: end-to-end proof, CI, deployment and records

- [ ] **T34 [S] (25 min) Canary privacy test**
  - Files: `crates/server/tests/canary.rs`.
  - `pg_canary_not_in_any_public_surface` (P12): hidden tests carry canary strings in input and expected output; a scripted hostile executor tries to leak them through every free-text field and through sample tests that are flagged hidden. Run `run` and `submit` flows including a retry. Scan every `run_events` payload, `test_results` row, `GET /api/runs/{id}` response and the raw SSE bytes. Also assert that hidden `test` payload keys are exactly `ordinal`, `verdict`, `time_ms`, `sample`.
  - Requirements: R15, P12, R22.10.
  - Depends on: T26, T31.
  - Verify: `cargo test -p counterspell-server --test canary -- --ignored pg_`.
  - Preserve: the canary list is generated per run so no real answer is ever stored in the test.
  - Rust lesson: why tests use generated markers instead of real data.

- [ ] **T35 [S] (30 min) Restart-recovery integration tests**
  - Files: `crates/server/tests/restart.rs`.
  - `restart_server_mid_stream` (drop the server, start a new router on the same database, the client resumes with `Last-Event-ID`, full log delivered, no duplicates after de-duplication); `restart_judge_fleet_mid_run` (kill all workers during a run, start new ones, the reaper recovers the run, `restarted` appears, one terminal result); `restart_with_no_judge_expires_stale_queue` (jobs wait, then expire when a judge starts); `pg_retries_exhausted_end_to_end` (executor `Hang` on every attempt with a short lease → `internal_error`, `error` event, `GET` agrees).
  - Requirements: R9, R13.9, R22.4, R22.7, P5, P15, P17.
  - Depends on: T17, T31.
  - Verify: `cargo test -p counterspell-server --test restart -- --ignored pg_`.
  - Preserve: ages leases with SQL wherever a real wait is not the point.
  - Rust lesson: stopping and restarting tasks cleanly in tests.

- [ ] **T36 [S] [SUPERVISED] (30 min) Definition-of-done test: twenty concurrent runs and the latency report**
  - Files: `crates/server/tests/dod_twenty_runs.rs`, `crates/server/tests/support/report.rs`.
  - `pg_dod_twenty_concurrent_runs` exactly as in `design.md` "Definition of done": 20 users post at once behind a barrier, three judge workers with concurrency 2 and the scripted lightweight executor, five clients drop and resume, five other users plus an anonymous client probe every run's events, `GET` and cancel. Assert no lost jobs, no duplicate terminal results, no unauthorized access, no canary, ordered de-duplicated streams, enqueue p95 ≤ 300 ms, all terminal within 60 s. Measure `enqueue_ms`, `first_event_ms`, `completion_ms`, `queue_wait_ms`, `execution_ms`, `delivery_ms` separately (count, mean, p50, p95, max). Write `target/run-lifecycle/dod-report.json` and print a Markdown table.
  - Requirements: R20, R21, R22, P1–P13.
  - Depends on: T34, T35.
  - Verify: `cargo test -p counterspell-server --test dod_twenty_runs -- --ignored pg_ --nocapture`; open the JSON report and check all six metrics are present.
  - Preserve: the report is produced by the test; nothing in it is typed by hand.
  - Rust lesson: running many tasks concurrently and collecting results (`JoinSet`).

- [ ] **T37 [P] (25 min) CI: services, database tests, release guard, web tests, report artifact**
  - Files: `.github/workflows/ci.yml`.
  - In the `rust` job add Postgres 17 and Valkey 8 service containers with health checks, `DATABASE_URL` and `VALKEY_URL` for the new steps, a step `cargo test --workspace -- --ignored pg_ valkey_`, a step that builds the release server and judge and fails if `cargo tree -e features` shows `testkit` for either, and an `actions/upload-artifact` step for `target/run-lifecycle/dod-report.json`. In the `web` job add `npm test`. Keep the existing steps unchanged. Use the CI-only throwaway database credentials in the workflow file; they are not production secrets.
  - Requirements: R22, R21, scope constraints.
  - Depends on: T36.
  - Verify: `act`-free check: read the workflow; run the same commands locally against a scratch Postgres and Valkey; push to a branch and confirm CI is green.
  - Preserve: `cargo test --workspace` without services must still pass (database tests are ignored).
  - Rust lesson: none (no Rust changed).

- [ ] **T38 [P] (25 min) Caddy, Compose and deployment notes**
  - Files: `Caddyfile`, `docker-compose.yml` (server and judge environment only), `deploy/run-lifecycle.md`.
  - Check that Caddy does not buffer or compress `text/event-stream` (set `flush_interval -1` on the reverse proxy and exclude event streams from `encode` if needed). Give the server and judge separate `DATABASE_URL` variables that refer to login users in `counterspell_server` and `counterspell_judge` respectively (variable names only, no values). Add `deploy/run-lifecycle.md` describing how to create the two login users and grant membership, the config table, the single-server-process assumption, and how to read the DoD report.
  - Requirements: R13.7, R19.1, R1.6.
  - Depends on: T19.
  - Verify: `docker compose config`; `caddy validate` if available; an SSE request through Caddy shows events arriving as they are written (manual check recorded in the task note).
  - Preserve: the judge keeps no published ports and no main-daemon socket; the server never gets a sandbox socket; no secrets are committed.
  - Rust lesson: none (no Rust changed).

- [ ] **T39 [S] (25 min) Documentation and recorded results**
  - Files: `README.md` (run-lifecycle section), `docs/kiro-log/run-lifecycle-dod.md`, `docs/KIRO_USAGE.md` (one line), `THIRD-PARTY-NOTICES.md` (verify completeness).
  - Run T36 once on the dev machine, copy the generated numbers from `dod-report.json` into `docs/kiro-log/run-lifecycle-dod.md` (enqueue and completion latency kept in separate tables, with the machine description and the date), and list any property that was not covered. Add a README section: how to run database tests locally, the `pg_` convention and the lifecycle diagram. Confirm every new crate is in the notices.
  - Requirements: R20.3, R21.
  - Depends on: T36, T37, T38.
  - Verify: the numbers in the log equal the JSON file; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; `cargo test --workspace`; `cargo test --workspace -- --ignored pg_ valkey_`; `cd web && npm run check && npm test && npm run build`; `cargo deny check licenses`.
  - Preserve: no claim in the docs goes beyond what was measured.
  - Rust lesson: none (no Rust changed).

## Execution order

- **Wave 0:** T01 first. T02, T05 in parallel after it (T05 needs nothing). T03 and T04 after T02 (parallel). T06 after T05. T07 after T05.
- **Wave 1:** T08 after T01. T09 after T03, T04, T06, T08. T10 after T07, T08. T12 after T04, T06, T08. T11 together with or after T12 (the `restarted` event uses the append helper). T13 after T11 and T12. T14 after T12 and T13. T15 after T13 and T14. T16 after T04 and T12. T17 after T11–T16. T18 after T11–T15 (parallel with T16, T17). T19 after T17.
- **Wave 2:** T20 after T01 (can start during Wave 1). T21, T22, T23 after T20, in parallel. T24 after T05 and T20. T25 after T21–T24. T26 after T14 and T25. T27 after T17 and T25. T28 after T12 and T20. T29 after T26 and T28. T30 after T29. T31 after T17 and T30.
- **Wave 3:** T32 has no Rust dependencies and can run at any time. T33 after T32.
- **Wave 4:** T34 after T26 and T31. T35 after T17 and T31. T36 after T34 and T35. T37 after T36. T38 after T19. T39 last.

## Completion checklist

- [ ] No new Cargo package was added; `cargo metadata --no-deps` still lists exactly the six canonical packages.
- [ ] `counterspell-common` has no database, web, Docker or Tokio dependency; the server's normal dependency tree has no `bollard` and no `counterspell-judge`.
- [ ] Postgres is the only store for submissions, jobs and run events; Valkey holds counters only; nothing uses Valkey Streams or pub/sub.
- [ ] Submission and job are created in one transaction; `jobs.submission_id` is unique; `Idempotency-Key` replay returns the original run and creates nothing.
- [ ] Claims use `FOR UPDATE SKIP LOCKED` with renewable leases, and every judge write passes the (job, owner, attempts) fence.
- [ ] The reaper requeues expired leases with `attempts + 1`, ends exhausted runs as `internal_error`, expires stale queued jobs, and reclaims stuck runs after the maximum claim age.
- [ ] The terminal write is one fenced transaction protected by three barriers (fence, frozen-row trigger, unique terminal event).
- [ ] `seq` is gap-free and equals commit order; a retried attempt starts with `restarted`.
- [ ] SSE checks ownership before any event query, replays `seq > Last-Event-ID` from Postgres, uses `NOTIFY` only as a hint, works with notifications disabled, and closes after the terminal event.
- [ ] The client reducer de-duplicates by `seq`, detects gaps, and clears per-attempt state on `restarted`.
- [ ] No hidden input, expected output, or hidden-test program output appears in any public surface (canary test passes).
- [ ] `GET /api/runs/{id}` returns one stable final result after a reload.
- [ ] VAL-1 to VAL-3 and the rate limits, active-job cap and queue-depth cap are enforced server-side; rejected requests create nothing.
- [ ] The server database role cannot forge events, results or verdicts (role test passes).
- [ ] The 18 correctness properties P1–P18 each have a passing test, and `pg_model_interleavings` passes with its regression file committed.
- [ ] `pg_dod_twenty_concurrent_runs` passes; `docs/kiro-log/run-lifecycle-dod.md` records enqueue and completion latencies as separate tables, copied from the generated report.
- [ ] fmt, Clippy (`-D warnings`), workspace tests, database tests, web checks and `cargo deny check licenses` are green in CI.
