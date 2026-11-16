# language-drivers: done, pending tick

Each entry: task id, commit(s), tests that prove it, the exact verification commands and their result. A task is done only after Kiro re-verifies and ticks it.

## T08 bench-mode driver handoff

- Commit: `2260504`.
- Drivers accept bench mode and the judge hands bench work to the language drivers through the executor `measure` path.

## T09 bench persistence and public events

- Commits: `b16a9ae` (implementation), `7c02e5c` (review-requested missing tests).
- Proving tests added under the 17:54 review:
  - `crates/common/src/familiar/payload.rs` — `bench_start`/`bench_point` serialise with only public fields (`sizes`, `n`, `median_ns`); wire-shape round-trips.
  - `crates/judge/tests/familiar_pg.rs` — `pg_run_timings_insert_and_read_back`, `pg_run_timings_attempt_fences_retries` (both `#[ignore = "needs DATABASE_URL"]`).
  - `crates/server/tests/sse.rs` — `pg_replay_returns_bench_events`: `bench_start` -> `bench_point` -> `done` replay in sequence with exact JSON payloads.
- Verification run 16 Nov 2026 against test Postgres (`postgres://counterspell@localhost:5432/postgres`):
  - `cargo test -p counterspell-judge --test familiar_pg -- --ignored` -> 2 passed, 0 failed.
  - `cargo test -p counterspell-server --test sse -- --ignored pg_replay_returns_bench` -> 1 passed, 0 failed.
  - `cargo test -p counterspell-common familiar` -> 58 passed, 0 failed.
- Gates: `cargo fmt --all -- --check` ok; `cargo clippy -p counterspell-common -p counterspell-judge -p counterspell-server --all-targets` clean.
- Schema note: `run_events.type` CHECK accepts `bench_start` via `migrations/20261116130000_run_events_bench_start.sql`.
