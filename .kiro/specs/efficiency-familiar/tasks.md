# Tasks: efficiency-familiar

All tasks are implementation tasks for the design in `design.md`. No task in this document is executed by the spec-generation run. Each task is sized for 30 minutes or less. `[P]` means it can run in parallel with other tasks in the same wave once its dependencies are done. `[S]` means it is sequential. `[SUPERVISED]` means Brighton reviews the diff before the task is marked done (database, terminal write, trust boundary). `[BLOCKED?]` means the task needs another spec's implementation; T01 records whether it is available. `[STRETCH]` tasks do not start before the core is freeze-safe.

Build order follows the brief: (1) the measured layer, (2) the mascot, (3) the speech bubble, (4) the stretch live guess.

Conventions for every Rust task:

- Follow `.kiro/steering/ghost-engine.md` (for the engine), `sandbox.md` (for the judge), `security.md`, `api-conventions.md` and `rust-teaching.md`. No `unwrap()` or `expect()` outside tests; no `unsafe`.
- The workspace has Clippy pedantic on and CI uses `-D warnings`. Run `cargo clippy -p <package> --all-targets -- -D warnings` before finishing. Prefer `u64::try_from` over `as`. Where a float cast is provably safe, allow the single lint on that line with a comment.
- Every public item gets a plain-English doc comment.
- End the task with a **Rust lesson** of 3 to 5 lines appended to `docs/kiro-log/rust-lessons.md` in the existing format (`# Rust lesson: efficiency-familiar Txx`). The suggested topic is listed per task.
- Database tests are `#[ignore]`d and named `pg_*`, and run with `-- --ignored pg_` (the run-lifecycle convention).
- Do not add a crate, a package or a Rust dependency. If `cargo deny check licenses` fails for anything, stop and ask Brighton before editing `deny.toml`.
- Do not read or write `.env`, keys or credentials. Do not run any user, reference or fixture code on the host.

Conventions for every web task:

- Follow `.kiro/steering/frontend.md`: Svelte 5 runes, semantic palette tokens only (no raw hex in components), GSAP only inside the Familiar, no ambient IDE motion, plain-text output, no colour-only status.
- Before creating a file under `web/src/lib/familiar/`, check whether `ide-workspace` already created it. If so, extend it; do not create a second copy.
- Test with vitest (`cd web && npm test`) and `npm run check`. Fixtures are labelled and live only in test files.

## Wave 0: prerequisites and shared types

- [ ] **T01 [S] (20 min) Audit prerequisites and record what exists**
  - Files: `docs/kiro-log/efficiency-familiar-prereqs.md` (new). No source changes.
  - Check and record, as a table of present / missing / blocked tasks: run-lifecycle migrations, `FinalResult`, `PublicEvent::{Complexity, Familiar}`, `RunContext::emit`, read model and shared JSON error envelope; `Verdict` and `RunMode` in `counterspell-common`; `language-drivers` bench code and its `run_timings` use; `hints::ai::generate(Kind::Familiar, …)` and the `hints` table; `.kiro/specs/judge-sandbox/` and the sandbox runner and image digest; `PackRunner::bench`; `web/static/familiar/` and any `assets/` art; `vitest` in `web/package.json`; existing `web/src/lib/familiar/**` from `ide-workspace`.
  - For every missing item, list the tasks below that it blocks.
  - Requirements: scope constraints.
  - Depends on: none.
  - Verify: the file names every seam row from the design's seam table with a status.
  - Preserve: nothing is created outside the log file. Do not generate art.

- [ ] **T02 [P] (25 min) Class, confidence, mood and effect types in `counterspell-common`**
  - Files: `crates/common/src/familiar/{mod.rs,class.rs}`, `crates/common/src/lib.rs` (module line only).
  - Implement `ClassId` (seven ids exactly as GRM-16: `1`, `log n`, `n`, `n log n`, `n^2`, `n^3`, `2^n`) with `rank()`, `id()`, `display_name()` ("constant" … "exponential"), `FromStr`, and serde as the id string; `Confidence`; `Mood` (newtype checked 1 to 5, `name()`: furious … laughing); `Effect` (`none`, `crack_burst`, `laugh_burst`).
  - Tests: ranks 0 to 6 in order; every id round-trips; unknown ids are rejected; `Mood::try_from` accepts only 1 to 5; serde forms are stable.
  - Requirements: R3.2, R5, R6.
  - Depends on: T01.
  - Verify: `cargo test -p counterspell-common familiar`.
  - Preserve: `counterspell-common` gains no dependency and no async, DB or Docker code.
  - Rust lesson: enums with data-free variants, `FromStr`, newtypes that enforce a range.

- [ ] **T03 [S] (30 min) `MeasuredFacts` and the `decide` function**
  - Files: `crates/common/src/familiar/{facts.rs,mood.rs}`.
  - Implement `MeasuredFacts`, `BenchFacts`, `NoMeasurement`, `MoodReason` (every reason code in R5.1) and `decide` exactly as the R5.1 table, including gap calculation, the `<= 2.0` ratio rule, effects, and Run versus Submit.
  - Tests: one example per table row; property P8 (totality and table), P9 (monotone in gap), P10 (effects); an exhaustive test over the finite combinations (verdicts × modes × no-measurement × classes × confidences × ratio buckets).
  - Add the P11 source-scan test: `mood.rs` and `facts.rs` must not contain the identifiers `hint`, `ai`, `guess`, `speech`, `prompt`, `prefs` (word-bounded, case-insensitive).
  - Requirements: R1.1 to R1.4, R5.
  - Depends on: T02, `Verdict` and `RunMode` from run-lifecycle T02.
  - Verify: `cargo test -p counterspell-common familiar::mood`.
  - Preserve: `decide` is pure, takes only `&MeasuredFacts`, and has no other input path.
  - Rust lesson: exhaustive `match`, why the compiler checks you covered every case, and small pure functions.

- [ ] **T04 [P] (30 min) Payload and column types**
  - Files: `crates/common/src/familiar/payload.rs`.
  - Implement `ComplexityPayload`, `FamiliarPayload`, `ComplexitySection`, `FamiliarSection`, `FitSummary`, `BenchStatus` (`measured`, `not_specified`, `host_busy`, `failed`, `incomplete`) and `MeasuredColumns` (the five stored values). All `Serialize`/`Deserialize` with `deny_unknown_fields` on inputs. Add `to_bounded_json` helpers that use run-lifecycle's `BoundedJson`.
  - Tests: JSON shapes match the design examples; a maximal `FitSummary` (32 points) stays under 4 KiB or is truncated by documented rules (points capped at 8 in storage); payloads stay under 16 KiB; unknown fields are rejected.
  - Requirements: R7.6 to R7.10.
  - Depends on: T02, run-lifecycle T04 (`BoundedJson`).
  - Verify: `cargo test -p counterspell-common familiar::payload`.
  - Preserve: no field can hold source, hidden input, expected output or stderr.
  - Rust lesson: `serde` derive, `deny_unknown_fields`, and why wire types are separate from internal types.

- [ ] **T05 [S] (30 min) Wording: headline, badge, template**
  - Files: `crates/common/src/familiar/words.rs`.
  - Implement `headline`, `badge` and `template_bubble` as pure functions of (decision, observed class, confidence, plausible classes, reference, ratio, mode, verdict) following the design's wording table. Add `FORBIDDEN_WORDS`. "Quick check: " prefix for Run in measured cases.
  - Tests: P12 as an exhaustive enumeration over all (mood, reason, class, confidence, mode) combinations: headline ≤ 160 characters, badge ≤ 40, plain text (no `<`, `>`, backticks), no forbidden word, the top mood uses exactly "Matches reference complexity" or "Scales better than the reference", failure text never mentions complexity.
  - Requirements: R6, R11.10.
  - Depends on: T03.
  - Verify: `cargo test -p counterspell-common familiar::words`.
  - Preserve: observation wording ("looks quadratic, medium confidence"); never "optimal".
  - Rust lesson: `format!`, string building without allocation surprises, and testing text by enumeration.

## Wave 1: the fit and bench inputs (engine)

- [ ] **T06 [S] (25 min) Engine complexity module skeleton and validation**
  - Files: `crates/engine/src/complexity/mod.rs`, `crates/engine/src/lib.rs` (module line, docs), `crates/engine/Cargo.toml` is **not** changed.
  - Add `Class` (seven classes, rank, id), `Point`, `Mode`, `Confidence`, `Reason`, `FitConfig` with the design's constants and `Default`, `FitError`, `FitReport`, `Comparison`, `FIT_VERSION = "fit-1"`. Implement input validation only. Until T09, `fit` returns the temporary variant `FitError::NotImplemented` after validation succeeds; T09 removes that variant.
  - Tests: every R3.1 rejection (2 points, 33 points, `n < 2`, not increasing, zero median, negative, NaN and infinite spread).
  - Requirements: R3.1, R3.2, R3.11, R3.12.
  - Depends on: T01.
  - Verify: `cargo test -p counterspell-engine complexity`; the existing engine purity test still passes (no new dependency, no process, file, network or clock use).
  - Preserve: the engine keeps only `serde` and `serde_json`; it does not depend on `counterspell-common`.
  - Rust lesson: `Result` and custom error enums, and why a crate's dependency list is part of its design.

- [ ] **T07 [S] (30 min) Model fitting: basis, weighted non-negative least squares, residuals**
  - Files: `crates/engine/src/complexity/models.rs`.
  - Implement the six polynomial/logarithmic models as `t ≈ a + c·g(n)` with `a, c ≥ 0` (normal equations, clamp and refit), normalising the basis and the timings first, the log-space `2^n` model, and the RMS log residual `E_m` with the large constant for non-positive predictions.
  - Tests: exact data from each model with an offset is recovered with `E ≈ 0` for its own model and clearly larger for the others; flat data collapses every model to the constant; decreasing data does not produce negative parameters; huge `n` (up to `u64::MAX`) gives finite residuals and no overflow for `2^n`.
  - Requirements: R3.3 (c), R3.10.
  - Depends on: T06.
  - Verify: `cargo test -p counterspell-engine complexity::models`.
  - Preserve: all arithmetic is deterministic `f64`; no random numbers.
  - Rust lesson: `f64` pitfalls (NaN, infinity), `is_finite`, and clamping.

- [ ] **T08 [S] (30 min) Noise, slope view and local exponents**
  - Files: `crates/engine/src/complexity/{noise.rs,views.rs}`.
  - Implement per-point and combined noise (`σ_i`, `σ̄`), the least-squares log-log slope over the top `max(3, ceil(k/2))` points with `se_b = max(regression SE, σ̄/sqrt(Sxx))`, and the local exponents with their median over the same segment.
  - Tests: exact power laws give the right slope and exponents; known spread maps to the expected `σ_i`; unknown spread uses `DEFAULT_NOISE`; `k = 3` uses the noise-implied SE.
  - Requirements: R3.3 (a)(b).
  - Depends on: T06.
  - Verify: `cargo test -p counterspell-engine complexity::views`.
  - Preserve: functions are pure and take plain slices.
  - Rust lesson: slices, iterators and why `windows(2)` suits consecutive pairs.

- [ ] **T09 [S] (30 min) Plausible set, agreement, stability, confidence, `fit` and `compare`**
  - Files: `crates/engine/src/complexity/{decide.rs,mod.rs}`.
  - Implement steps 4 to 10 of the design's algorithm: plausible set, agreement per model (including the `2^n` rule), drop-one stability for `k ≥ 5`, the `Low`/`Medium`/`High` rules, the run-mode cap, `class` only for `Medium` and `High`, `reasons`, and `compare` (gap and `reference_plausible`). Make `fit` return a `FitReport` with serde.
  - Tests: golden cases for clean `n`, `n log n`, `n^2`, `n^3`, flat, a single spiked point, a short range, a class outside the set (`n^1.5`), `k = 3` for `Mode::Quick`; each asserts class, confidence and reasons.
  - Requirements: R3.3 to R3.7, R3.11.
  - Depends on: T07, T08.
  - Verify: `cargo test -p counterspell-engine complexity`.
  - Preserve: a class is never reported at `Low`; `High` needs all the listed conditions.
  - Rust lesson: borrowing slices, `Option` for "maybe a class", and sorting with `partial_cmp`.

- [ ] **T10 [P] (30 min) Synthetic timing corpus and strategies**
  - Files: `crates/engine/tests/support/complexity.rs`, `crates/engine/tests/support/mod.rs` (add a module line).
  - Build the series generator from the design: true class ∈ {`n`, `n log n`, `n^2`, `n^3`}; `k` ∈ 3..=8; size factor 2 or 4; `n_0` ∈ 2^6..2^10; fixed overhead `ρ` ∈ [0, 3] of the compute at `n_0`; lognormal multiplicative noise `s` ∈ [0, 0.25]; spike probability up to 0.1 with multiplier 1.5 to 3; five repeats per point, median and `(max − min)/median` as `Point`. A seeded variant for the calibration report. Offer a "spread unknown" form.
  - Tests: the generator itself obeys its parameter ranges; noise-free output is exactly `a + c·g(n)`.
  - Requirements: R3.8, R3.9, R15.2.
  - Depends on: T06.
  - Verify: `cargo test -p counterspell-engine --test properties complexity_corpus` (or the module name used).
  - Preserve: test support only; nothing here ships.
  - Rust lesson: `proptest` strategies and why a generator needs its own tests.

- [ ] **T11 [S] (30 min) Fit properties P1 to P7**
  - Files: `crates/engine/tests/properties_complexity.rs`.
  - Implement P1 (soundness), P2 (usefulness), P3 (scale invariance with powers of two), P4 (totality on arbitrary and extreme inputs), P5 (determinism), P6 (class only with confidence; `plausible` contains `leaning`), P7 (`Quick` never `High`). Run them. If P1 or P2 fail, **record the failing seeds and stop**; T12 fixes constants.
  - Requirements: R3.4 to R3.10.
  - Depends on: T09, T10.
  - Verify: properties compile and run; the run output is saved to `docs/kiro-log/efficiency-familiar-calibration.md` as the "before" section.
  - Preserve: regression files are committed.
  - Rust lesson: property tests versus example tests, and reading a shrunk counterexample.

- [ ] **T12 [S] [SUPERVISED] (30 min) Calibrate and freeze the constants as `fit-1`**
  - Files: `crates/engine/src/complexity/mod.rs` (constants only), `docs/kiro-log/efficiency-familiar-calibration.md`.
  - Adjust only constants in `FitConfig`, and only toward less confidence, until P1 and P2 pass with at least 4,000 cases. Do not change the algorithm. Record the final constants, the corpus parameters, and for each true class the share of `Low`, `Medium`-correct, `Medium`-adjacent and `High`-correct results, and the number of wrong-at-`High` results (must be zero). If P2 cannot pass without breaking P1, stop and report to Brighton instead of weakening either property.
  - Requirements: R3.8, R3.9, R15.2.
  - Depends on: T11.
  - Verify: `cargo test -p counterspell-engine --test properties_complexity` passes; the report contains the table and the zero.
  - Preserve: `FIT_VERSION` stays `fit-1` only if constants are frozen here; any later change bumps it.
  - Rust lesson: constants as configuration, and why tuning tests is different from tuning to the tests.

- [ ] **T13 [P] (30 min) Sized bench inputs and the bench seed**
  - Files: `crates/engine/src/complexity/bench_input.rs`.
  - Implement `sized_input` and `bench_seed` as in the design, using the engine's existing PRNG (`SplitMix64`) and `derive_seed`-style FNV-1a with length-prefixed fields plus `GENERATOR_VERSION`. Support `int_array`, `str` and `matrix` size parameters inside a tuple root; honour range, alphabet, `sorted`, `unique`; return `BenchInputError` when `unique` is impossible or the size parameter is not a supported kind.
  - Tests: P15 (exact size; valid by `Schema::contains` apart from the size parameter's length bound; deterministic; independent of anything but its arguments; error cases); pinned `bench_seed` values.
  - Requirements: R2.2 to R2.4.
  - Depends on: T06, ghost-engine `Schema`, `SplitMix64`.
  - Verify: `cargo test -p counterspell-engine complexity::bench_input`.
  - Preserve: the engine stays pure and synchronous.
  - Rust lesson: deterministic random numbers and why seeds are derived from data, not time.

- [ ] **T14 [P] (20 min) Ratio vs baseline**
  - Files: `crates/judge/src/familiar/{mod.rs,ratio.rs}`.
  - Implement the geometric-mean ratio over common sizes, absent for fewer than two common sizes or any zero median. Add the usability filter type for baselines (problem version, image digest) as a pure function over rows.
  - Tests: P14; stale version and wrong digest are rejected; order independence.
  - Requirements: R4.3, R4.4.
  - Depends on: run-lifecycle T08 (judge as library).
  - Verify: `cargo test -p counterspell-judge familiar::ratio`.
  - Preserve: no database access in this module.
  - Rust lesson: `iter().map().sum()`, `ln`/`exp`, and avoiding division by zero.

## Wave 2: persistence, judge wiring and read model

- [ ] **T15 [S] [SUPERVISED] (30 min) Migration: columns, constraints, freeze trigger, tables and grants**
  - Files: `migrations/<timestamp>_efficiency_familiar.sql` (timestamp later than every existing migration).
  - Add the five `submissions` columns and the `submissions_familiar_ck` constraint from the design; the separate `cs_familiar_frozen` trigger; `run_timings` and `problem_baselines` (`IF NOT EXISTS`); the grants in the design's privilege table (column-level `UPDATE` for the judge role, none for the server role). No `DELETE` or `TRUNCATE` grants. No passwords or secrets.
  - Requirements: R7.1 to R7.5, R12.2.
  - Depends on: run-lifecycle T05 to T07 (tables, guards, roles).
  - Verify: `sqlx migrate run` twice on a clean database succeeds; `docker compose config` still valid.
  - Preserve: the bootstrap migration and run-lifecycle migrations are untouched; the freeze function of run-lifecycle is not replaced.

- [ ] **T16 [S] (30 min) Database tests for constraints, freeze and roles (P16)**
  - Files: `crates/judge/tests/familiar_pg.rs`.
  - `pg_familiar_constraints`: a non-accepted verdict with mood above 1 is rejected; a class with `low` confidence is rejected; a mood on a non-completed run is rejected; mood without reason is rejected; an oversize `fit` is rejected; a valid row is accepted.
  - `pg_familiar_frozen`: after `finished_at` is set, changing any of the five columns fails, and the terminal `UPDATE` that sets them together with `finished_at` succeeds.
  - `pg_familiar_roles`: as `counterspell_server`, updating or inserting any new column or table fails; as `counterspell_judge`, the permitted writes succeed and `DELETE` fails.
  - Requirements: R7.2, R7.3, R7.5, R12.2.
  - Depends on: T15.
  - Verify: `cargo test -p counterspell-judge --test familiar_pg -- --ignored pg_`.
  - Preserve: tests use an admin pool for setup and `SET LOCAL ROLE` for role checks.
  - Rust lesson: `sqlx` queries and transactions that roll back after a test.

- [ ] **T17 [S] (30 min) Pure `assess` and the engine-to-common mapping**
  - Files: `crates/judge/src/familiar/assess.rs`.
  - Implement `AssessInput`, `Assessment`, `BenchState` and `assess` as in the design. One exhaustive `match` converts engine `Class`/`Confidence` to common `ClassId`/`Confidence`. The complexity payload, the familiar payload and the stored columns are built from the same values. Non-completed runs yield no mood.
  - Tests: P13 (one value, two uses) as PBT; a table of scenarios from the failure matrix (no bench, host busy, failed, incomplete, low, each gap, no baseline, slower constant, matches, better); engine and common agree on ids and ranks for all seven classes.
  - Requirements: R1, R4, R5, R7.4, R7.6.
  - Depends on: T03, T04, T05, T09, T14.
  - Verify: `cargo test -p counterspell-judge familiar::assess`.
  - Preserve: pure; no clock, no I/O.
  - Rust lesson: conversion functions between types, `From`, and exhaustive `match` for safety.

- [ ] **T18 [P] (25 min) Load guard and bench gate**
  - Files: `crates/judge/src/familiar/{load.rs,gate.rs}`.
  - `LoadSource` trait with `ProcLoadAvg` (reads the first field of `/proc/loadavg`, returns an error on parse failure) and a test double; `quiet(before, after, limit)` is a pure function; `BenchGate` is a one-permit `tokio::sync::Semaphore` wrapper controlled by `BENCH_SERIALISE` (default true), with `BENCH_MAX_LOAD` (default 3.0 [E]).
  - Tests: quiet/busy cases at the boundary; unreadable load is treated as unknown and does not discard the bench (logged); two tasks cannot hold the gate together; a waiter proceeds after release; the gate does not hold a database connection.
  - Requirements: R2.5.
  - Depends on: run-lifecycle T08.
  - Verify: `cargo test -p counterspell-judge familiar::load familiar::gate`.
  - Preserve: reading `/proc/loadavg` is the only file access.
  - Rust lesson: traits as seams for testing, and `Semaphore` permits dropping automatically.

- [ ] **T19 [S] [SUPERVISED] (30 min) Store the measured columns in the terminal write**
  - Files: wherever run-lifecycle T14 defined `FinalResult` and `finish` (`crates/judge/src/queue/finish.rs` expected).
  - Add `measured: Option<MeasuredColumns>` to `FinalResult`. In the existing fenced transaction, set the five columns in the same `UPDATE` that sets `finished_at`. Failed, expired and internal-error paths store none.
  - Tests: a completed run stores exactly the columns `assess` produced; a worker that lost its lease writes nothing; a violating value (mood 5 with a wrong verdict) fails the transaction and is reported as an infrastructure error; the existing terminal-write tests still pass.
  - Requirements: R1.5, R7.4.
  - Depends on: T15, T17, run-lifecycle T14.
  - Verify: `cargo test -p counterspell-judge --test terminal -- --ignored pg_` (the run-lifecycle terminal tests plus the new cases).
  - Preserve: the three barriers (fence, frozen row, one terminal event) remain intact.
  - Rust lesson: `Option` fields in a struct, and why a database transaction keeps related writes together.

- [ ] **T20 [S] (30 min) `measure_run`: collector trait, events and the failure matrix**
  - Files: `crates/judge/src/familiar/mod.rs`, `crates/judge/src/executor.rs` or the existing call site (one small call).
  - Define `BenchPlan`, `BenchPoint`, `BenchOutcome` and `BenchCollector` as in the design. Implement `measure_run(ctx, run, collector, baselines, problem)`: failure path emits `familiar` right after the last test and before `ghost_hunting`; pass path acquires the gate, reads load, calls the collector, reads load, assesses, then emits `complexity` (only when fitted) and `familiar`. Maps `Failed` and `Incomplete` and `host_busy` to missing measurements, not errors; maps `ExecError` through unchanged. Provide a scripted collector in `testkit`.
  - Tests with the fake executor: every row of the failure matrix; event order (`bench_start`… `complexity`, `familiar`, then `done`); a `restarted` attempt produces a fresh result; infrastructure error is retried and never yields a mood; hidden-data canary absent from every payload.
  - Requirements: R1, R2, R7.6, R7.7.
  - Depends on: T17, T18, T19, run-lifecycle T16.
  - Verify: `cargo test -p counterspell-judge familiar` and `-- --ignored pg_` for the database-backed cases.
  - Preserve: the executor still cannot write a verdict or status; `language-drivers` bench code is not rewritten here, only called through the trait.
  - Rust lesson: async traits with `impl Future`, generics instead of `dyn`, and passing a context object.

- [ ] **T21 [S] (30 min) Baseline store**
  - Files: `crates/judge/src/familiar/store.rs`.
  - `insert_baseline(rows)` (insert-only, `ON CONFLICT DO NOTHING`, returns how many were new) and `find_baseline(slug, version, lang, digest)` returning sorted `(n, median_ns)`; all SQL for this module lives here. The reader returns nothing for a stale version or a different digest.
  - Tests (`pg_`): insert, re-insert (no overwrite), a new version creates a second set, a digest mismatch finds nothing, the judge role cannot update or delete.
  - Requirements: R4.3, R8.2, R8.3.
  - Depends on: T15, T14.
  - Verify: `cargo test -p counterspell-judge --test familiar_pg -- --ignored pg_baseline`.
  - Preserve: SQL appears only in `store.rs`.
  - Rust lesson: parameterised queries and why string-built SQL is never used.

- [ ] **T22 [S] [BLOCKED? judge-sandbox, language-drivers] (30 min) `counterspell-judge baseline` subcommand and the admission test**
  - Files: `crates/judge/src/familiar/baseline_cmd.rs`, `crates/judge/src/main.rs` (subcommand dispatch only).
  - `baseline --problem <slug> --lang <lang>`: load the pack's known-correct fixture for that language, run bench mode through the normal sandbox path at the pack's sizes (Submit budget), fit it, and **refuse to store** unless the fit is the declared reference class at `medium` or better; print why when refusing. Otherwise insert rows with problem version, image digest, engine and fit versions. Exit code 2 for infrastructure problems.
  - Tests: with a scripted collector and fake store: accepts a matching fixture, refuses a mismatching or `Low` one, is idempotent; no code runs outside the collector.
  - Requirements: R8, R13.2.
  - Depends on: T20, T21, judge-sandbox runner, language-drivers T08.
  - Verify: unit tests above; one real run on the VPS configuration is done in T40.
  - Preserve: all execution goes through the sandbox runner; nothing is executed on the host.
  - Rust lesson: command-line subcommands, exit codes and `Result` at a program's edge.

- [ ] **T23 [S] (30 min) Read model sections and stream parity test (P17)**
  - Files: `crates/server/src/runs/read.rs`, `crates/server/tests/familiar_read.rs`.
  - Build the `complexity` and `familiar` sections of `GET /api/runs/{id}` from stored columns only; absent when null; no `effect`. Add the `pg_` integration test: run the scripted executor and collector, collect the SSE `complexity`/`familiar` payloads, fetch the read model after, and assert equality (minus `effect`) and that a reload from a fresh client shows the same.
  - Requirements: R7.4, R7.8, R7.9.
  - Depends on: T04, T19, T20, run-lifecycle T26, T29.
  - Verify: `cargo test -p counterspell-server --test familiar_read -- --ignored pg_`.
  - Preserve: the owner predicate is in the query; the server role performs no writes.
  - Rust lesson: building a response from stored data, and `Option` with `skip_serializing_if`.

- [ ] **T24 [P] (30 min) Grimoire: SBX-05 through the fit and the BEN-01 static check**
  - Files: `crates/grimoire/src/sandbox.rs`, `crates/grimoire/src/codes.rs`, `crates/grimoire/src/limits.rs`, focused tests.
  - When the runner's `bench` returns `(n, median_ns)` pairs, call `engine::complexity::fit` (`Mode::Full`, spread unknown) and emit SBX-05 when the class differs from `reference_complexity` or is not confirmed (`Low`), with the plausible classes in the message. Stop emitting SBX-08 once the runner implements `bench`. Add `BEN-01` (error): for every declared size, `engine::complexity::sized_input` succeeds with the pack's schema. Add a warning when sizes span less than 32×.
  - Tests with a fake runner: matching class passes; a quadratic reference declared as linear warns; `Low` warns; infrastructure error stays exit 2 (SBX-06); an infeasible `unique` bench size gives BEN-01.
  - Requirements: R13.1, R13.4.
  - Depends on: T09, T13, grimoire-packs T19.
  - Verify: `cargo test -p counterspell-grimoire`.
  - Preserve: the grimoire crate still executes nothing and does not depend on the judge.
  - Rust lesson: trait objects versus generics for a fake runner in tests.

- [ ] **T25 [S] [BLOCKED? judge-sandbox, language-drivers] (30 min) Implement `PackRunner::bench` in the judge's runner**
  - Files: `crates/judge/src/` (the module that implements `PackRunner`, expected from judge-sandbox), tests.
  - Implement `bench(sized_inputs, limit)` by delegating to the same `BenchCollector` code path the production run uses (so the admission test measures what users are measured by), returning `(n, median_ns)` pairs.
  - Tests: scripted collector returns the points unchanged; a timeout gives `RunnerError`/incomplete as designed; no host execution.
  - Requirements: R13.1, R13.2.
  - Depends on: T20, grimoire-packs `PackRunner`, judge-sandbox runner, language-drivers T08.
  - Verify: `cargo test -p counterspell-judge`; real run in T40.
  - Preserve: one measurement path for users, baselines and pack validation.
  - Rust lesson: implementing a trait from another crate and keeping one code path.

## Wave 3: the mascot (web) and preferences

- [ ] **T26 [S] (30 min) Web prerequisites: asset contract, placeholder, GSAP and notices**
  - Files: `web/src/lib/familiar/mascot/contract.ts`, `contract.test.ts`, `web/static/familiar/` (audit; placeholder only if the art is absent), `web/package.json`, `web/package-lock.json`, `THIRD-PARTY-NOTICES.md`, `docs/kiro-log/efficiency-familiar-prereqs.md` (append).
  - Audit `web/static/familiar/`. Implement `validate(svgText)` for the id contract in the design. If a conforming SVG exists, validate it and record its size. If not, add a **clearly labelled placeholder** (`jack-o-lantern.placeholder.svg`, text "PLACEHOLDER ART" in the file) that satisfies the contract, so the code can be built and tested; record in the log that the real art is still needed. Add `gsap` as a dependency (npm only, not vendored) and its notice; add `vitest` if run-lifecycle T32 has not.
  - Requirements: R9.1, R9.11, R9.12.
  - Depends on: T01.
  - Verify: `cd web && npm ci && npm test && npm run check && npm run build`; the contract test names any missing id; SVG size within the 120 KB budget or the overrun is recorded.
  - Preserve: do not draw or generate final art; the placeholder never ships as the default once real art exists.

- [ ] **T27 [P] (25 min) Mood styling and the effect planner (P22)**
  - Files: `web/src/lib/familiar/mascot/{mood-style.ts,plan.ts}`, tests.
  - `mood-style.ts`: mood to name, layer ids, icon name, text token, and the glow recipe (token names and mix percentages, no hex). `plan.ts`: `planEffect` exactly as designed.
  - Tests: exhaustive table for `planEffect` over all combinations; every mood has a style; no hex strings in either file (source scan); passing moods never select the danger text token.
  - Requirements: R9.2, R9.5 to R9.9, R9.13.
  - Depends on: T26.
  - Verify: `cd web && npm test -- mascot`.
  - Preserve: both modules are pure and import nothing from GSAP.

- [ ] **T28 [P] (25 min) Seeded particle and ember plans (P24)**
  - Files: `web/src/lib/familiar/mascot/particles.ts`, tests.
  - Pure functions that build the 40-particle crack plan (position, velocity, size, colour token name) and the 12-ember + 3-spark ring from a seed, with target durations (about 900 ms and 700 ms).
  - Tests: counts exact; values finite and bounded; same seed gives the same plan; durations within 5% of target.
  - Requirements: R9.3, R9.4.
  - Depends on: T26.
  - Verify: `cd web && npm test -- particles`.
  - Preserve: no GSAP, no DOM.

- [ ] **T29 [S] (30 min) `Mascot.svelte`: static rendering of five moods**
  - Files: `web/src/lib/familiar/mascot/Mascot.svelte`, tests.
  - Load the SVG lazily, show the layers for the current mood, set the glow from the resolved palette tokens, expose the `aria-live="polite"` text and the badge, support the hidden-dot form, and render everything without GSAP. Props: `mood`, `reason`, `headline`, `badge`, `effect`, `live`, `muted`, `hidden`, `reducedMotion`. Missing art shows the badge text only.
  - Tests (component): each mood shows the right layer; headline and badge appear as text; an HTML-looking headline renders as text (P25); the aria-live text matches the required form; hidden state is a keyboard-operable button with a 44 px target; no `onerror` or `innerHTML` in the component.
  - Requirements: R9.1, R9.2, R9.7 to R9.10, R12.5.
  - Depends on: T27, T26.
  - Verify: `cd web && npm test -- Mascot && npm run check`.
  - Preserve: the mascot never covers editor text and fetches only its own static assets.

- [ ] **T30 [S] (30 min) GSAP timelines: crossfade, laugh burst, crack-and-burst, dispose**
  - Files: `web/src/lib/familiar/mascot/timelines.ts`, `Mascot.svelte` (wiring only), tests.
  - Dynamic `import('gsap')` only when `planEffect` returns `play` and motion is allowed. Implement the 300 ms crossfade, the crack draw-on followed by the particle burst, and the laugh shake with embers and sparks, using only transform and opacity and the plans from T28. Register every timeline with a disposer and call it on unmount, route change and change of `reducedMotion`, `muted` or `hidden`.
  - Tests: with GSAP mocked, `import('gsap')` is never called under reduced motion or muted; a played effect creates the expected number of `fam-fx` children and removes them on dispose; a second effect inside 2 s applies the mood without playing; disposing twice is safe.
  - Requirements: R9.3 to R9.6, R9.11, R9.12.
  - Depends on: T28, T29.
  - Verify: `cd web && npm test -- timelines`; manual check in `npm run dev` with the placeholder art recorded in the log (screen recording optional).
  - Preserve: no looping animation; effects play once.

- [ ] **T31 [S] (30 min) Mute and hide behaviour and the component contract**
  - Files: `web/src/lib/familiar/mascot/Mascot.svelte`, `web/src/lib/familiar/mascot/CONTRACT.md` (short), tests.
  - Muted: static face, no timelines, headline visible. Hidden: 16 px dot in the status bar, text equivalent still announced. Document the props that `ide-workspace`'s `Familiar.svelte` passes, and the events it must pass back (toggle mute, toggle hide).
  - Tests: muted never loads GSAP; hidden removes the face from the tab order but keeps the dot focusable and the live text; reduced motion plus a `crack_burst` shows the badge text "Wrong answer" or "Timeout" and no particles.
  - Requirements: R9.7 to R9.10.
  - Depends on: T30.
  - Verify: `cd web && npm test -- Mascot && npm run check`.
  - Preserve: the menu UI belongs to `ide-workspace`; do not duplicate it.

- [ ] **T32 [P] (30 min) Preferences endpoints**
  - Files: `crates/server/src/familiar/{mod.rs,prefs.rs}`, `crates/server/tests/familiar_prefs.rs`.
  - `prefs.rs` (pure): `validate_patch`, `merge`, defaults. `GET` and `PATCH /api/me/familiar` use `AuthUser`, the origin guard, a 1 KiB body limit, JSON-only, the window limiter, and only touch the session user's `users.familiar_prefs`.
  - Tests: unknown field, wrong type, bad enum, oversized body and wrong content type are rejected; defaults are filled; a valid patch merges and persists; user A cannot change user B; the demo user can change prefs; unauthenticated gets 401.
  - Requirements: R10.
  - Depends on: auth T-series (`AuthUser`, origin guard), run-lifecycle limiter.
  - Verify: `cargo test -p counterspell-server familiar::prefs` and `--test familiar_prefs -- --ignored pg_`.
  - Preserve: the server role writes only `familiar_prefs`.
  - Rust lesson: validating untrusted JSON into a typed struct with `serde`.

- [ ] **T33 [S] (30 min) Browser state rules and preference sync (P23)**
  - Files: `web/src/lib/familiar/familiar-state.ts`, `web/src/lib/api/familiar.ts` (extend or create), tests.
  - Ensure the reducer changes mood only on a `familiar` event (`live = true`) or on hydration from the read model (`live = false`, effect dropped), clears on `restarted`, and has no input from editor changes or the guess. Preferences: load from `GET /api/me/familiar`, apply locally first, `PATCH` in the background, mark "not synced" on failure.
  - Tests: edits, language switches and unrelated events leave the state unchanged; duplicates by `seq` ignored; reload hydration shows a static face; a failed PATCH keeps the local choice.
  - Requirements: R1.7, R1.8, R9.6, R10.5.
  - Depends on: T23, T32, `ide-workspace` T-series if present.
  - Verify: `cd web && npm test -- familiar-state`.
  - Preserve: do not duplicate `ide-workspace`'s reducer; extend it.

- [ ] **T34 [P] (25 min) Explainer view-model for "How does it judge me?"**
  - Files: `web/src/lib/familiar/explain.ts`, tests.
  - Pure function from the read-model `complexity` section to what the popover shows: points on log-log axes, the slope with its error, the plausible classes, the per-model residuals sorted, the reference, the ratio vs baseline, and fixed sentences for the honest limits. No new data is invented; missing sections give an explicit "not measured" state.
  - Tests: sample sections give stable view-models; `Low` confidence shows "unsure" and the plausible list; text contains no forbidden word.
  - Requirements: R6, R7.8 (data for FAM-5).
  - Depends on: T23.
  - Verify: `cd web && npm test -- explain`.
  - Preserve: the popover component itself belongs to `ide-workspace`.

## Wave 4: the speech bubble

- [ ] **T35 [P] (30 min) Familiar prompt builder (P19)**
  - Files: `crates/server/src/familiar/prompt.rs`.
  - Build the `Prompt` for `Kind::Familiar` from: the code (≤ 8 KB), the stored measured block, the reference class, the verdict summary and the public ghost fields. Use the hints data markers and instruction skeleton; add the "never name an unmeasured complexity", "unsure when low", "never optimal" lines. Return a `NotApplicable`/`CodeTooLarge` variant instead of a prompt when needed.
  - Tests: P19 (canary in a hidden expected output or in the statement never appears; size bound under 16 KiB for any code up to the limit); the prompt includes no identity fields; an injection string in the code stays inside the data markers.
  - Requirements: R11.4, R11.5.
  - Depends on: T04, hints `guard.rs` conventions.
  - Verify: `cargo test -p counterspell-server familiar::prompt`.
  - Preserve: pure function; no I/O.
  - Rust lesson: borrowing strings into a larger value, and `Cow` or `&str` versus `String`.

- [ ] **T36 [P] (30 min) Claim guard (P18)**
  - Files: `crates/server/src/familiar/claims.rs`.
  - Implement `check_claims(text, allowed: &[ClassId]) -> Verdict { Ok, UnmeasuredClass(id|any), Forbidden }` with the fixed pattern lists from the design, case-insensitive, word-bounded, including `O(…)` forms and the superscript forms.
  - Tests: P18 with generated sentences; allowed classes pass; any other class is rejected; each forbidden phrase is rejected; "linear search" with `n` not allowed is rejected (strict by design).
  - Requirements: R11.7.
  - Depends on: T02.
  - Verify: `cargo test -p counterspell-server familiar::claims`.
  - Preserve: false positives fall back to the template and are acceptable.
  - Rust lesson: string matching without a regex crate, and the cost of being strict.

- [ ] **T37 [S] (30 min) Speech orchestration, one-attempt marker and template fallback (P20)**
  - Files: `crates/server/src/familiar/{attempt.rs,speech.rs,store.rs}`.
  - `attempt.rs`: Valkey `SET NX EX 86400` marker for the run. `speech.rs`: stored hint row first; speech `off`; code size; marker; `hints::ai::generate(Kind::Familiar, …)`; claim guard; insert the `hints` row (`source = 'familiar'`, `ON CONFLICT DO NOTHING`); otherwise the template with the right `reason`. `store.rs` reads the stored result with the owner predicate.
  - Tests with the scripted transport and Valkey: P20 (N concurrent calls, one provider call, identical bodies); cap reached, both providers failing, speech off, oversized code, guard rejection and marker already set each return a template with the right reason; the stored row is returned without quota use; Valkey down returns a template.
  - Requirements: R11.3, R11.6 to R11.10.
  - Depends on: T35, T36, T05, hints `ai::generate` and `hints` table.
  - Verify: `cargo test -p counterspell-server familiar::speech` and `-- --ignored pg_`.
  - Preserve: this module has no write access to mood columns and cannot call `decide`.
  - Rust lesson: `async` functions, early returns with `?`, and why one place owns each side effect.

- [ ] **T38 [S] (30 min) `POST /api/runs/{id}/familiar-hint` route (P21)**
  - Files: `crates/server/src/familiar/mod.rs`, `crates/server/tests/familiar_hint.rs`.
  - Handler: `AuthUser`, window limit, parse the run id, owner-checked read, `404`/`409`/`422` rules from R11, then `speech`. Response has no mood field. Errors use the shared envelope.
  - Tests: P21 (non-owner, unknown and malformed id give byte-identical `404`); `409` for an unfinished run of the caller; `422` for a run without a mood (cancelled, internal error); success shapes for `ai` and `template`; the response never contains a mood; the label is `"AI hint"` only for `ai`.
  - Requirements: R11.1, R11.2, R11.8, R11.11, R12.1.
  - Depends on: T37, run-lifecycle limiter.
  - Verify: `cargo test -p counterspell-server --test familiar_hint -- --ignored pg_`.
  - Preserve: ownership predicate in the query; no new write privileges.
  - Rust lesson: Axum extractors and returning typed JSON errors.

- [ ] **T39 [S] (30 min) Speech bubble component and the once-per-run request**
  - Files: `web/src/lib/familiar/bubble/{SpeechBubble.svelte,request.ts}`, tests.
  - The component shows the body as text, the "AI hint" label for `source: "ai"`, a small "From your measured results" caption for templates, and a one-time notice that the code was sent to a third-party provider. `request.ts` calls the endpoint at most once per run id (in-memory map survives remounts and SSE reconnects) and respects `speech: "off"`. Extend `ide-workspace`'s existing call if present.
  - Tests: second call for the same run returns the same promise; reconnect and remount do not repeat the request; failure shows a safe message and leaves mood and verdict untouched; HTML in the body renders as text (P25); no label for templates.
  - Requirements: R11.8, R11.9, R11.13, R12.5.
  - Depends on: T38, T33.
  - Verify: `cd web && npm test -- bubble && npm run check`.
  - Preserve: the bubble never changes verdict or mood; typing never triggers a request.

## Wave 5: integration, proof and records

- [ ] **T40 [S] [SUPERVISED] [BLOCKED? judge-sandbox, language-drivers, grimoire-packs] (30 min) Enable bench on launch problems through the admission test**
  - Files: `packs/<slug>/pack.toml` (`[bench]`, `reference_complexity`, `complexity_note`) for each problem that passes; `docs/kiro-log/efficiency-familiar-admission.md`.
  - For each launch problem with a meaningful growth curve, run `validate --sandbox` and `counterspell-judge baseline` for Python and JavaScript on the VPS configuration. Enable bench only where the reference passes at `medium` or better in both languages; record the baselines. Target at least three problems, including a pairs-style one with a linear reference. Record the ones that fail and why.
  - Requirements: R13.2, R13.3, R15.4.
  - Depends on: T22, T24, T25, language-drivers T08 and T09.
  - Verify: the log lists every problem with its admission result; baselines exist for each enabled problem and language; if fewer than three pass, say so in the log and tell Brighton.
  - Preserve: original problems only; no copied statements.

- [ ] **T41 [S] [SUPERVISED] (30 min) Real-stack proof: linear versus quadratic, 30 runs each**
  - Files: `crates/judge/tests/familiar_proof.rs` (ignored, runs against the real sandbox), `docs/kiro-log/efficiency-familiar-proof.md`.
  - Submit a known-correct linear solution and a deliberately quadratic one at least 30 times each through the real server and judge with `JUDGE_CONCURRENCY=2`, alternating, with some overlap. Record the distribution of confidence and mood, host load readings, discarded benches, and latency added by the bench.
  - Requirements: R15.3.
  - Depends on: T40, T20, T23.
  - Verify: the quadratic solution never reaches mood 5; no result is `high` with the wrong class; the report states the numbers. If either fails, stop and report to Brighton with the raw rows; do not loosen the criteria.
  - Preserve: run only against a local or staging stack that you own.

- [ ] **T42 [S] (30 min) End-to-end pass in the IDE**
  - Files: `docs/kiro-log/efficiency-familiar-e2e.md`.
  - With the real stack, walk: a wrong submit (crack-and-burst, Furious headline, then ghost), a slow correct submit (Angry or Uneasy, headline, popover data), a good submit (Laughing, burst once, "Matches reference complexity"), a reload (static face, no effect), reduced motion, mute, hide, a speech bubble (AI), and a forced template (cap reached). Record event ids and run ids.
  - Requirements: R15.4, R9, R11.
  - Depends on: T39, T41, `ide-workspace` core tasks.
  - Verify: every item has a recorded pass or a bug filed in the log.
  - Preserve: no fabricated results; fixtures are not used for this pass.

- [ ] **T43 [P] (30 min) Privacy and security tests across surfaces**
  - Files: `crates/judge/tests/familiar_privacy.rs`, `crates/server/tests/familiar_privacy.rs`, `web/src/lib/familiar/privacy.test.ts`.
  - Seed a canary in a hidden test's expected output and input, in stderr, and in the problem statement. Assert it never appears in any `complexity` or `familiar` event, the `fit` column, the read model, the speech prompt, the bubble response or logs. Confirm the server role cannot write the new columns or tables (extends P16), and that web components never use `innerHTML`.
  - Requirements: R7.9, R11.4, R12.
  - Depends on: T20, T23, T38, T39.
  - Verify: tests pass in CI.
  - Preserve: canaries are test-only strings.

- [ ] **T44 [P] (25 min) Documentation and notices**
  - Files: `docs/efficiency-familiar.md` (new), `THIRD-PARTY-NOTICES.md`, `README.md` (one line), `docs/KIRO_USAGE.md` (entry).
  - Explain in plain words how the Familiar judges, the five moods, the honest limits and the confidence rules; how to regenerate baselines; the constants and `fit-1`; GSAP's notice (and `web-tree-sitter` if the stretch ships).
  - Requirements: R6, R15.
  - Depends on: T42.
  - Verify: the doc contains no "optimal" claim and states the limits from the design.
  - Preserve: the licence statements match `docs/licence-rules.md`.

- [ ] **T45 [S] (30 min) Full verification gate**
  - Files: none planned; fixes only if a check fails.
  - Run: `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; `cargo test --workspace`; the `pg_` suites; `cargo deny check licenses`; `cd web && npm ci && npm run check && npm test && npm run build`. Confirm `cargo metadata --no-deps` lists exactly six packages and the engine manifest still has only `serde` and `serde_json`.
  - Requirements: R15.5.
  - Depends on: T43, T44.
  - Verify: every command passes.
  - Preserve: no stretch work started.
  - Rust lesson: why the CI gates are cheap insurance.

## Wave 6: stretch, live typing guess (only after the core is freeze-safe)

- [ ] **T46 [STRETCH] [S] (25 min) Decide grammar sources and size budget**
  - Files: `docs/kiro-log/efficiency-familiar-guess.md`.
  - Determine where the `web-tree-sitter` runtime and the Python and JavaScript grammar `.wasm` files come from (npm package or build step), their licences (must be MIT or compatible) and sizes. Set a total budget (for example 600 KB gzip [E]).
  - Requirements: R14.1, R14.6.
  - Depends on: T45 and Brighton's go-ahead.
  - Verify: sources, licences and sizes recorded; if the budget cannot be met, record that the stretch is dropped.
  - Preserve: nothing is added to the build yet.

- [ ] **T47 [STRETCH] [P] (30 min) Pure heuristics**
  - Files: `web/src/lib/familiar/guess/heuristics.ts`, tests.
  - Implement the signals from the design over a small syntax-tree interface (so tests use plain objects): loop depth over input-dependent iterables, linear search or `remove`/`insert(0)` in a loop, sort then a pass, halving loops, multi-self-call recursion without memo. Return `{ guess_class, confidence: "low" | "med", reasons[] }`.
  - Tests: golden snippets for the common shapes (double loop, hash map single pass, sort then scan, binary search, naive Fibonacci); reasons are non-empty whenever a class is returned; never above `med`; deterministic.
  - Requirements: R14.2.
  - Depends on: T46.
  - Verify: `cd web && npm test -- heuristics`.
  - Preserve: no import from the mood path.

- [ ] **T48 [STRETCH] [S] (30 min) Worker, labelled bubble, switch and boundary test (P26)**
  - Files: `web/src/lib/familiar/guess/{worker.ts,GuessBubble.svelte}`, `web/src/lib/familiar/guess/boundary.test.ts`.
  - Load the parser lazily in a Web Worker, debounce about 700 ms, show the fixed label "Guess from reading your code. Run it for the real verdict.", and add an on/off switch (default off until chosen). The boundary test scans sources: `guess/**` imports nothing from `familiar-state`, `mascot`, `$lib/runs`, nor any network API; nothing in the mood path imports `guess/`.
  - Requirements: R14.
  - Depends on: T47, T33.
  - Verify: `cd web && npm test -- guess && npm run check && npm run build`; the mood stays unchanged while typing (state test).
  - Preserve: the guess never changes the mood, is never sent to the server, and loads nothing until enabled.

## Dependency summary and parallel waves

- **Wave 0:** T01 first. T02 after T01; T03 after T02; T04 after T02; T05 after T03.
- **Wave 1 (engine):** T06 after T01. T07 and T08 in parallel after T06. T09 after T07 and T08. T10 after T06 (parallel with T07 to T09). T11 after T09 and T10. T12 after T11. T13 after T06. T14 after T01 and run-lifecycle T08.
- **Wave 2:** T15 after run-lifecycle T05 to T07. T16 after T15. T17 after T03, T04, T05, T09, T14. T18 after run-lifecycle T08. T19 after T15, T17 and run-lifecycle T14. T20 after T17, T18, T19 and run-lifecycle T16. T21 after T15 and T14. T22 after T20, T21 and judge-sandbox. T23 after T04, T19, T20. T24 after T09, T13 and grimoire-packs T19. T25 after T20 and judge-sandbox.
- **Wave 3 (web and prefs):** T26 after T01. T27 and T28 after T26 (parallel). T29 after T26 and T27. T30 after T28 and T29. T31 after T30. T32 after auth and the run-lifecycle limiter (parallel with the web tasks). T33 after T23 and T32. T34 after T23.
- **Wave 4 (speech):** T35 and T36 after T04 and T02 (parallel; they can start during Wave 2). T37 after T35, T36, T05 and hints. T38 after T37. T39 after T38 and T33.
- **Wave 5:** T40 after T22, T24, T25 and language-drivers T08 and T09. T41 after T40. T42 after T39 and T41. T43 after T20, T23, T38, T39. T44 after T42. T45 after T43 and T44.
- **Wave 6 (stretch):** T46 to T48 only after T45 and Brighton's go-ahead.

Critical path for the demo: T01 → T02 → T03 → T06 → T09 → T11 → T12 → T17 → T19 → T20 → T23 → T29 → T30 → T33. The speech bubble (Wave 4) and the real bench (T22, T25, T40) can lag without blocking the pass or fail reactions, which need only T03, T05, T15, T17, T19, T20 and the mascot.

## Completion checklist

- [ ] No new Cargo package or Rust dependency; `cargo metadata --no-deps` lists exactly the six canonical packages; the engine manifest still has only `serde` and `serde_json`.
- [ ] The mood is computed once, by `decide`, from `MeasuredFacts` only; a source-scan test proves no AI, hint, guess, speech, prompt or prefs identifier is reachable from it.
- [ ] A non-Accepted verdict always gives mood 1; `internal_error`, `failed` and `expired` runs have no mood and no reaction.
- [ ] The fit passes P1 to P7 on the synthetic corpus, with zero wrong-at-`High` results and a committed calibration report; constants are frozen as `fit-1`.
- [ ] Results are worded as observations; the top mood says "Matches reference complexity"; no string says "optimal" (exhaustive test).
- [ ] The migration, constraints, freeze trigger and grants are in place; the server role cannot write any mood, class, timing or baseline.
- [ ] The five columns, the `complexity` event, the `familiar` event and the read-model sections come from one value and agree after a reload.
- [ ] Bench is gated (one at a time), guarded by load, and discarded or incomplete benches give Pleased with an honest headline, never a complexity claim.
- [ ] Baselines are same-language, same-version, same-digest, insert-only, and created only after the admission test.
- [ ] The mascot plays effects only for live events, honours reduced motion, mute and hide, loads GSAP lazily and disposes every timeline, and uses palette tokens only.
- [ ] The speech bubble is requested once per run, grounded only on the allowed inputs, passes the claim guard, is labelled "AI hint", falls back to a template, and cannot change verdict or mood.
- [ ] Preferences endpoints validate input and act only on the session user.
- [ ] The real-stack proof shows the quadratic solution never reaching mood 5 and no wrong class at `high`.
- [ ] At least three launch problems have bench enabled, or the shortfall is reported to Brighton.
- [ ] fmt, Clippy (`-D warnings`), workspace tests, `pg_` tests, web check, web tests and `cargo deny check licenses` are green.
- [ ] If the stretch ships: the guess is labelled, switchable, lazy, never changes the mood, and the boundary test passes.
