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

## T10 generated-input driver properties for every published pack

- Commit: `390bd2e` (plus `821b982` fixing the C19 signature shape it exposed).
- Fixtures: `packs/{count-pairs,diagonal-total,longest-run-length}/fixtures/{python,javascript}/known_correct.{py,js}` — test-only programs, never reference rows or expected-output sources.
- `crates/grimoire/src/pack.rs`: the allow-list now descends `fixtures/<lang>/` and accepts `known_correct.<ext>` files (64 KiB cap).
- `crates/judge/src/catalogue/load.rs`: `signature_descriptor` is `pub` — the stored signature is the driver-protocol wire shape (`protocol_version`/`parameters`/`returns` with `{"kind": …}` descriptors), which the judge writes verbatim to `/box/signature.json`.
- Proving test: `tests/sbx_pack_props.rs` — `sbx_published_packs_fixtures_agree_with_reference` enumerates every `status = "published"` pack (guard: at least the three starter packs), builds a `SandboxOracle` per language fixture against `reference/solution.py`, draws `hidden_count` inputs from `CandidateStream` seeded by `meta.generator.seed` (samples seeded first), and asserts per input: both sides `Returned`, answer survives a canonical JSON re-encode, and `pack.comparator.equal(reference, fixture)`; then re-evaluates input 0 for determinism. Failure messages carry slug, lang, seed, index and input.
- Verification: `SANDBOX_DOCKER_SOCKET=/run/user/1000/docker.sock cargo test -p counterspell-judge --features sandbox --test sbx_pack_props -- --ignored` -> 1 passed; `drain_dead` left the daemon with zero managed containers.
- Gates: `cargo fmt --all -- --check` ok; `cargo clippy --workspace --all-targets --features sandbox` clean; all 20 grimoire suites green (allow-list change).

## T11 hostile protocol and cleanup behavior

- Commit: `9783902`.
- Gap analysis: Python forgery already covered — `hostile/forged_lines.py` + `fake_verdict.py` write straight to fd 1 (S14/S15). The uncovered vector was the JavaScript equivalent: `fs.writeSync(1, …)` bypasses the driver's `process.stdout.write` sink onto the protocol pipe.
- Proving tests: `tests/sbx_driver_hostile.rs` — five real-daemon cases through `execute_run` on `add(a, b)`, each ending with `sbx::drain_dead` so no managed container survives:
  - `sbx_js_fd1_forged_answer_is_never_accepted` — forged `{"t":0,"out":0}` + wrong return -> not Accepted.
  - `sbx_js_fd1_wrong_call_id_is_never_accepted` — forged `{"t":99,…}` -> protocol refusal, not Accepted.
  - `sbx_js_fd1_verdict_document_is_never_accepted` — bare `{"verdict":"accepted"}` line + verdict-shaped `out` value -> not Accepted.
  - `sbx_js_infinite_loop_is_a_time_limit` — `for(;;){}` -> `FinalVerdict::TimeLimit` at the external deadline.
  - `sbx_js_fd1_flood_is_never_accepted` — 4 MiB direct write -> limit verdict, not Accepted.
- Shared helper fix: `sbx/mod.rs` `drain_dead` tolerates a 404 between `list_managed` and `inspect` (container already removed = the clean outcome).
- Verification: `SANDBOX_DOCKER_SOCKET=/run/user/1000/docker.sock cargo test -p counterspell-judge --features sandbox --test sbx_driver_hostile -- --ignored` -> 5 passed, 0 failed.
- Note: no keyword blocklist was added; verdict safety is structural (answers are compared values only).

## T12 core freeze-safe gate

- Gate run 17 Nov 2026 on the local rootless daemon (`SANDBOX_DOCKER_SOCKET=/run/user/1000/docker.sock`, WSL host). Result: **PASS**.
- Workspace checks: `cargo fmt --all -- --check` ok; `cargo clippy --workspace --all-targets --features sandbox` clean; `cargo test -p counterspell-judge --features sandbox` all suites green, 0 failed; `cargo test -p counterspell-grimoire` 20 suites green.
- Launch judging: `sbx_functional` 7, `sbx_python_driver` 16, `sbx_javascript_driver` 20, `sbx_javascript_run` 5, `sbx_driver_parity` 2 — all green.
- Generated-input properties: `sbx_pack_props` green (every published pack, both languages).
- Bench-mode timing/events: `sbx_bench` 3 green; pg `pg_run_timings_*` 2 + `pg_replay_returns_bench_events` 1 green.
- Hostile/cleanup: `sbx_driver_hostile` 5, `sbx_forgery` 2, `sbx_output` 5, `sbx_limits` 9, `sbx_oracle` 1 green; filesystem/network/infra/logs/preflight/bollard-create/bollard-session/sweeper 19 green.
- Recovery + concurrency: `sbx_recovery` 3/3 and `sbx_concurrency` 2/2 pass under `--test-threads=1`. Under parallel threads they flake on the shared test daemon (s19's global container count sees s18's containers; removal lag exceeds the 10 s `cleared_within` bound under load). This is test-side contention, not a product defect — kills always land and containers are always reaped (verified: daemon empty afterwards).
- Preserve held: `languages/languages.toml` still has `java.enabled = false`, `cpp.enabled = false`; no stretch work entered the launch path.
- Java/C++ stretch tasks (T13–T16) may proceed on this gate's strength.

## Verified (spec-driver `verify all`, 17 Nov 2026)

Clean detached worktree at `335968e` (plus the one-line `cargo fmt` fix to `crates/grimoire/src/lib.rs`), rootless daemon `/run/user/1000/docker.sock`, Postgres from the dev env.

| Task | Result |
|---|---|
| T09 | ticked. `pg_run_timings_insert_and_read_back` and `pg_run_timings_attempt_fences_retries` pass; `pg_replay_returns_bench_events` passes; `familiar::payload` tests (public-field-only `bench_start`/`bench_point`) pass. |
| T10 | ticked. `sbx_published_packs_fixtures_agree_with_reference` passes on the real daemon. |
| T11 | ticked. `sbx_driver_hostile` 5/5 pass (fd-1 forgery, wrong call id, verdict document, infinite loop, flood). |
| T12 | NOT ticked. The core gate passed on 17 Nov, but it requires the workspace checks green, and they are red now (see `counterspell-mcp-power-blockers.md` and `problem-catalogue-blockers.md`: 12 `counterspell-mcp` tests and one catalogue-cli pg test fail after the pack-tree split; `cargo fmt --check` failed at `335968e`). Re-run T12 once those are fixed. |

Environment notes for whoever re-runs this: `/tmp` is a 3.9 GB tmpfs and filled up during the build (set `TMPDIR` and `CARGO_TARGET_DIR` to the real disk). `COUNTERSPELL_PACKS_PRIVATE_ROOT` in the dev env points at the empty scaffold in `~/code/counterspell-packs-private`; the populated checkout is the sibling `Personal Projects/counterspell-packs-private`.
