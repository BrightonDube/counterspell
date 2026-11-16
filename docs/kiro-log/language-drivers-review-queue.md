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
