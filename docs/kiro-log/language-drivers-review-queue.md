# language-drivers review queue (SUPERVISED tasks)

Each entry is a supervised change that Brighton should check before the work builds on it. Changes are written test-first where the harness allows.

## T08 generic bench-mode process handoff
- **Files:** `crates/judge/src/sandbox/bench.rs`, `crates/judge/src/sandbox/run.rs` (keep-session split), `crates/judge/tests/model_bench.rs`, `crates/judge/tests/sbx_bench.rs`.
- **Check:**
  - The normal driver is ended by dropping its channel (stdin closes, the process exits itself) — no new `DriverChannel` primitive was added, which resolves the T08 blocker noted in `language-drivers-blockers.md` without touching the sandbox boundary.
  - A clean driver opens with `ExecSpec::bench` (`run` argv + `bench_args`) in the same container, then the bench driver is put back into `Opened` so the normal close path stays valid.
  - Worker clock only: 2 untimed warm-ups, 5 timed calls per size, per-call deadline and whole-bench budget both enforced with `Instant`. The driver emits no timing.
  - Call ids are sequential across warm-ups, timed reps and sizes (encoded per call, not per size).
  - A spent budget mid-call is `BenchOutcome::Incomplete` with finished points kept — never `Failed`. A driver crash/hang is `Failed`. A daemon failure is `Err(ExecError::Infrastructure)` — never a verdict. Empty timings cannot panic `median_ns`.
  - Raw per-call samples are reduced to `median_ns` + `spread` per `BenchPoint` (the existing `familiar::assess` type); raw samples are not stored. Brighton should confirm that matches the "records raw samples" verify wording — the persisted `run_timings` rows in T09 may want them.
- **Evidence:** `model_bench` 6/6 (fake backend; sequential ids, per-size input reuse, stop, hang, crash, infra mapping). `sbx_bench` 2/2 on the rootless daemon with the real pinned `node:22-slim` image: six Submit sizes complete with real medians (~0.5ms/call) and a 1ms budget reports `Incomplete([])` with the container reaped.
- **Not done here:** `execute_run` does not yet hand a passing run to the collector (the keep-session split in `run.rs` is the seam); T09 wires persistence and events.

## T09 persist and publish bench measurements
- **Files:** `crates/judge/src/sandbox/bench.rs` (events + timing rows through `RunContext`), `crates/judge/src/sandbox/sandbox_executor.rs` (`measure` wiring, `MeasureCall`), `crates/judge/src/familiar/mod.rs` (`measure_run` optional collector + `MeasuredColumns`), `crates/judge/src/familiar/store.rs` (`TimingRow`, `insert_run_timings`), `crates/judge/src/executor.rs` (`record_timings`, `pool()`, `Recorded`), `crates/judge/src/sandbox/problem.rs` (`BenchDecl` + fields), `crates/judge/src/sandbox/testkit.rs` (fixture `bench` block), `crates/common/src/events.rs` + `run_state.rs` + `familiar/payload.rs` (`BenchStart` event, payloads), `migrations/20261116130000_run_events_bench_start.sql` (`run_events.type` CHECK), `web/src/lib/{runs/types,reducer}.ts`, `web/src/lib/familiar/types.ts` (`bench_start` parse/reduce), tests `model_sandbox_executor.rs`, `model_bench.rs`, `sbx_bench.rs`, `familiar_measure.rs`, `two-sum.json` bench block.
- **Check:**
  - `measure` runs for every completed run, including a precheck-refused source (familiar verdict, never a bench). An accepted run with a declared bench gets timed points in the same kept container, then `measure` closes the session whatever it returns.
  - `run_timings` rows are the raw samples (per the T08 open question): one row per timed call, `repeat` 1..=5, fenced by `attempt`, recorded only when the bench loop ends normally — a misbehaving driver's samples are never stored.
  - `bench_start`/`bench_point`/`complexity`/`familiar` go through `ctx.emit` — same fenced `run_events` path, `BoundedJson` payloads (sizes, n, median_ns only; no inputs, expected values, source or answers).
  - A partial bench declaration (sizes without schema/reference) is `ExecError::Infrastructure`, never a verdict. `verdict != Accepted` or no plan means the collector is never built.
  - `bench_start` joins the public contract end to end: `run_events.type` CHECK migration, `run_state` status-neutral arm, web `RUN_EVENT_NAMES` + `benchSizes` state.
  - `measured` columns ride `FinalResult.measured` into the existing terminal write (finish.rs already binds them).
- **Evidence:** `model_sandbox_executor` 14/14 (event order, timings count/shape, partial-decl infrastructure, no-bench familiar only). `model_bench` 6/6, `familiar_measure` 10/10 unchanged. `sbx_bench` 3/3 on the rootless daemon — full `execute` on `two-sum` JS: `compiling`, 3 tests, `bench_start`, 6 `bench_point`, `complexity`, `familiar`, 30 timing rows recorded, container reaped. All sbx suites re-run green (functional 7, limits 9, output 5, network 2, filesystem 4, infra 4, oracle 20, logs 1, drivers 19, recovery 2, concurrency 3). `cargo clippy --workspace --all-targets -D warnings` clean; `svelte-check` 0 errors; web vitest 141/141.
- **Not done here:** the production problem loader is still `EmptyProblemTests`; `problems.bench`/`input_schema`/`reference_complexity` columns are read once the catalogue loader lands (problem-catalogue). The queue `for_queue` context path is written but only exercised once run-lifecycle wires the worker.
