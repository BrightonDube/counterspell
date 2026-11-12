# Tasks: ghost-engine

All tasks are implementation tasks for the design in `design.md`. No task in this document is executed by the spec-generation run. Each task is sized for 30 minutes or less. `[P]` means it can run in parallel with other tasks in the same wave once its dependencies are done. `[S]` means it is sequential.

Conventions for every Rust task:

- Follow `.kiro/steering/ghost-engine.md`, `rust-teaching.md` and `security.md`. No `unwrap()` or `expect()` outside tests; no `unsafe`.
- The workspace has Clippy pedantic on and CI runs `-D warnings`. Run `cargo clippy -p counterspell-engine --all-targets -- -D warnings` before finishing.
- Every public item gets a plain-English doc comment (`#![warn(missing_docs)]`).
- End the task with a **Rust lesson** of 3–5 lines, appended to `docs/kiro-log/rust-lessons.md` in the existing format (`# Rust lesson: ghost-engine Txx`). The suggested topic is listed per task.
- Do not run any Python file on the host. Do not add a crate, a package or a dependency other than those named in T01.

## Wave 0: foundations and leaf modules

- [x] **T01 [S] (25 min) Add dependencies and the module skeleton**
  - Files: `crates/engine/Cargo.toml`, `crates/engine/src/lib.rs`, empty module files listed in the design, `THIRD-PARTY-NOTICES.md`, `Cargo.lock`.
  - Add `serde` (with `derive`) and `serde_json`; add `proptest` as a dev-dependency (try `default-features = false, features = ["std"]` first). Declare the modules from the design's layout. Add `GENERATOR_VERSION = "gen-1"`, `GHOST_LABEL`, keep `ENGINE_VERSION`. Add `#![warn(missing_docs)]` and module docs that state the dependency-direction table.
  - Requirements: R1.2, R18.1, R18.2.
  - Depends on: none.
  - Verify: `cargo check -p counterspell-engine`; `cargo deny check licenses` passes (if it fails, stop and ask Brighton before touching `deny.toml`); notices list the new crates.
  - Preserve: the engine keeps its package name and location; no other crate is edited.
  - Rust lesson: crates, features and why dependencies are kept few.

- [x] **T02 [P] (25 min) Seeded random numbers and seed derivation**
  - Files: `crates/engine/src/rng.rs`.
  - Implement SplitMix64, `gen_range(lo, hi)` for `i64` bounds using `u128` arithmetic with rejection sampling, `gen_bool(percent)`, and `derive_seed(slug, code_sha256, language)` as FNV-1a 64 over length-prefixed fields plus `ENGINE_VERSION`. No `DefaultHasher`.
  - Unit tests: pinned first outputs for three seeds; pinned `derive_seed` value; length-prefix collision case (`("ab","c")` vs `("a","bc")`); full-range `gen_range` stays in bounds; `u64 → i64 → u64` seed round trip.
  - Requirements: R3.1–R3.3, R10 (seed storage).
  - Depends on: T01.
  - Verify: `cargo test -p counterspell-engine rng`.
  - Preserve: pinned values change only together with `GENERATOR_VERSION`.
  - Rust lesson: integer types, `as` casts versus `try_from`, wrapping arithmetic.

- [x] **T03 [S] (30 min) Schema type and definition checks**
  - Files: `crates/engine/src/schema.rs`.
  - Implement `Schema` with the serde form in the design, `deny_unknown_fields`, and `check_definition` (all rules in R2.3 and the design). Provide a clear `SchemaError` with `Display`. No `thiserror`.
  - Unit tests: valid examples for the three packs' schemas parse; each rule has a rejecting test; nested tuple, empty tuple, float bound, unknown field and unknown `type` are rejected.
  - Requirements: R2.1–R2.3, R2.8.
  - Depends on: T01.
  - Verify: `cargo test -p counterspell-engine schema`.
  - Preserve: no graph, tree, float or custom-checker kinds.
  - Rust lesson: `enum` with data, `serde` attributes, implementing `Display` by hand.

- [x] **T04 [S] (30 min) The validator `contains` and property P2**
  - Files: `crates/engine/src/schema.rs` (validator section), `Violation`.
  - Implement `Schema::contains(&self, &Value) -> Result<(), Violation>` as an independent walk (JSON pointer path, short reason). Use `as_i64`; reject floats and numbers above `i64::MAX`.
  - Tests: example tests per rule; proptest P2 (`arb_valid_value` then one controlled break); proptest P13 for `contains` on arbitrary JSON (no panic). Put `arb_schema`/`arb_value` in a `#[cfg(test)] mod strategies` inside the crate for now (T20 re-uses the idea in `tests/support`).
  - Requirements: R2.4–R2.7, R15.2, R15.11.
  - Depends on: T03.
  - Verify: `cargo test -p counterspell-engine schema`.
  - Preserve: `contains` shares no code with `generate.rs`.
  - Rust lesson: borrowing `&Value`, `match` on `serde_json::Value`, early return with `?`.

- [x] **T05 [P] (25 min) Size metric and property P3**
  - Files: `crates/engine/src/size.rs`.
  - Implement `SizeMetric`, `size_of`, `compare_size` as in the design (`nodes`, `magnitude` as `u128` serialised as a decimal string, canonical text tie-break). Floats and big numbers get magnitude 0 so the function is total.
  - Tests: the examples in the design (`0` → 1, `[0]` → 2, `([0],0)` → 4, the six table sizes); proptest P3 (irreflexive, exactly-one-of, transitive).
  - Requirements: R6, R15.3.
  - Depends on: T01.
  - Verify: `cargo test -p counterspell-engine size`.
  - Preserve: no randomness, no clock, no dependency on schema.
  - Rust lesson: `Ordering`, `Ord` versus a comparison function, and why `u128` is used.

- [x] **T06 [P] (20 min) Comparators and property P12**
  - Files: `crates/engine/src/compare.rs`.
  - Implement `Comparator::{Exact, Unordered}` with serde names `exact` and `unordered`, and `Comparator::equal(&self, expected, got) -> bool`.
  - Tests: `1` vs `1.0`, `true` vs `1`; permutation of top-level arrays; changed element detected; non-array falls back to exact; proptest P12.
  - Requirements: R4.7, R15.11.
  - Depends on: T01.
  - Verify: `cargo test -p counterspell-engine compare`.
  - Preserve: no float tolerance and no custom checkers.
  - Rust lesson: `derive(PartialEq)` and what `==` means on `serde_json::Value`.

- [x] **T07 [S] (30 min) Failure classification and sanitising**
  - Files: `crates/engine/src/classify.rs`.
  - Implement `ProgramResult`, `Observation`, `FailureKind` (serde `wrong`/`crash`/`timeout`), `FailurePredicate`, `Classification`, `classify`, and `sanitise_class`/`sanitise_text` (R4.8, R17.2). Provide `ProgramResult::sanitised()` used when a result enters a ghost.
  - Tests: every row of the classification table; garbled output is a wrong answer; reference crash, timeout or garble gives `ReferenceFailed`; hostile class strings (spaces, unicode, 10 KB) become `UnknownError`; messages are truncated on a character boundary; proptest P13 for `classify`.
  - Requirements: R4, R17.1–R17.2.
  - Depends on: T06.
  - Verify: `cargo test -p counterspell-engine classify`.
  - Preserve: no module from the generator or shrinker is imported; user text never decides a verdict beyond the comparison.
  - Rust lesson: `enum` variants with named fields, `Option`, and untrusted-input handling.

## Wave 1: generation and shrinking

- [x] **T08 [S] (30 min) Edge-biased value generation**
  - Files: `crates/engine/src/generate.rs` (value builders, `GeneratorConfig`).
  - Implement `generate_value(schema, rng, size_cap, config)` for every schema kind with the edge pools and collection shapes in the design, honouring `sorted` and `unique`, and a `units(value)` helper for the 256-unit cap.
  - Tests: proptest P1 for values only (valid for any schema, seed and cap); equal seeds give equal values; edge values (empty, min, max, zero, duplicates) all appear over 500 draws for a sample schema.
  - Requirements: R3.3, R3.4, R3.6, R3.8.
  - Depends on: T02, T04.
  - Verify: `cargo test -p counterspell-engine generate`.
  - Preserve: the generator does not call `contains` to repair its own output; invalid output must be a visible test failure.
  - Rust lesson: passing `&mut` state (the RNG) and why determinism needs it explicit.

- [x] **T09 [S] (30 min) Tiny enumeration and the candidate stream**
  - Files: `crates/engine/src/generate.rs` (enumeration, `CandidateStream`, `Candidate`, `Origin`).
  - Implement tiny pools per kind, the lazy odometer product with `enumeration_cap`, the stage order (known ghosts, samples, enumeration, random with growing cap), de-duplication, invalid-seed skipping with a counter, and the timeout-target size growth.
  - Tests: stage order; duplicates never yielded; invalid sample is skipped and counted; stream is lazy (taking 5 items does not build 500); proptest P1 for the whole stream; equal seeds give equal streams.
  - Requirements: R3.4–R3.8, R13.5.
  - Depends on: T08.
  - Verify: `cargo test -p counterspell-engine generate`.
  - Preserve: `generate` imports only `schema` and `rng`.
  - Rust lesson: implementing `Iterator`, laziness, and `HashSet` for de-duplication.

- [x] **T10 [S] (30 min) Shrink proposals, part 1: whole-value and scalar moves**
  - Files: `crates/engine/src/shrink.rs` (proposal generators).
  - Implement integer targets (nearest in-range value to 0), whole-value moves (all to target, all halved), scalar moves (jump, halve, step), and character moves toward the first alphabet character. Work on a path-based walk so tuple items shrink independently.
  - Tests: the `([5], 10)` coupled example reaches `([0], 0)` through a whole-value move; targets respect ranges that exclude 0; proptest: every proposal is either invalid (filtered later) or has the same shape as its source.
  - Requirements: R7.2.
  - Depends on: T04, T05.
  - Verify: `cargo test -p counterspell-engine shrink`.
  - Preserve: `shrink` imports only `schema` and `size`.
  - Rust lesson: recursion over a tree, cloning versus borrowing, `Vec` building.

- [x] **T11 [S] (30 min) Shrink proposals, part 2: structural removal**
  - Files: `crates/engine/src/shrink.rs`.
  - Implement ddmin-style chunk removal (sizes `⌈n/2⌉` down to 1) for arrays, strings, matrix rows and matrix columns, respecting minimum lengths and arity. Order the three proposal passes as in the design and expose them as one lazy iterator, `proposals(schema, value)`.
  - Tests: chunk sizes and order on an 8-element array; column removal removes the same index from every row; minimum lengths are never violated; tuples keep their arity.
  - Requirements: R7.2.
  - Depends on: T10.
  - Verify: `cargo test -p counterspell-engine shrink`.
  - Preserve: proposals are candidates only; legality and size checks happen in the loop.
  - Rust lesson: slices, `chunks`, and building iterators with `chain`.

- [x] **T12 [S] (30 min) The shrink loop, trace recorder and properties P4–P7, P11**
  - Files: `crates/engine/src/shrink.rs`.
  - Implement `Probe`, `shrink`, `ShrinkOutcome`, `ShrinkStop`, `ShrinkStep`, `ShrinkReport` and the head-10/tail-10 recorder with `omitted_steps`. Enforce validity, strict size decrease and the tried-set before every probe call.
  - Tests: the loop in the design on hand-made predicates; budget stop (`EvalBudget`); `Stop` from the probe; never evaluates a value twice; proptests P4, P5, P7, P11 using random deterministic predicates over random valid starts.
  - Requirements: R7, R11, R15.4–R15.6, R15.10.
  - Depends on: T11.
  - Verify: `cargo test -p counterspell-engine shrink`.
  - Preserve: the loop takes a callback only: no `Oracle`, `Clock` or `classify` import.
  - Rust lesson: `&mut dyn FnMut`, closures that capture state, and `VecDeque`.

## Wave 2: seam, autopsy and orchestration

- [x] **T13 [P] (25 min) Oracle, clock and budget**
  - Files: `crates/engine/src/oracle.rs`, `crates/engine/src/budget.rs`.
  - Implement `Oracle`, `OracleError` (message plus `Display`), `Clock`, `SystemClock`, `ManualClock` (interior `Cell` counter with `advance`), and `Budget` with the defaults in R8.1 and `validate()` (R8.2, total at most 8 s, deadlines leave the reserve). Document the Oracle contract from the design in the trait's doc comment.
  - Tests: defaults; budgets over 8 s or without reserve are rejected; `ManualClock` advances only when told; phase deadline helpers (`search_deadline`, `shrink_deadline`) match the time plan table.
  - Requirements: R1.3–R1.7, R8.1–R8.5.
  - Depends on: T01.
  - Verify: `cargo test -p counterspell-engine budget oracle`.
  - Preserve: `SystemClock` only reads `Instant`; no sleeping, no threads.
  - Rust lesson: traits as seams, `dyn Trait`, `Cell` for changing a value behind `&`.

- [x] **T14 [S] (30 min) Autopsy data**
  - Files: `crates/engine/src/autopsy.rs`.
  - Implement `Difference`, `diff_values` (cap 20 plus flag), `KnownGhost` with `matches(&ghost_parts)` per R12.6, `AuthoredMistake`, `Autopsy`, and the explanation templates (wrong, crash, timeout), with input rendering cut at 120 characters.
  - Tests: difference paths for scalars, array index changes, extra and missing items, type changes; the 20-item cap; each match-rule branch; template output for the three kinds; proptest P13 for `diff_values`.
  - Requirements: R12, R17.2.
  - Depends on: T05, T07.
  - Verify: `cargo test -p counterspell-engine autopsy`.
  - Preserve: no hidden test data; no template uses a forbidden phrase (checked in T24).
  - Rust lesson: recursion with a path accumulator, `Option` fields, `format!`.

- [x] **T15 [S] (30 min) Hunt, part 1: request, report types and the search phase**
  - Files: `crates/engine/src/hunt.rs`.
  - Implement `HuntIdentity`, `HuntRequest::new`, `Provenance`, `HuntStats`, `Ghost`, `Origin` re-use, `HuntOutcome`, `NotFoundReason`, `AbortReason`, `HuntReport` (all serde), request validation (schema definition, budget, non-empty digest, identity length caps), and the search loop: candidate stream, per-call limits from the clock, classification, target filtering, `ReferenceFailed` counting (abort after 3), `OracleError` abort, timeout skipping, and the `NotFound` reasons. Stop after the first find, returning an internal `FirstFailure` for the next task.
  - Tests with small synthetic oracles and a `ManualClock`: never-failing oracle gives `CandidateBudget` and `CandidatesExhausted` where expected; wall-time stop; reference failure abort; oracle error abort; target filter (a crash is ignored when the target is wrong answer).
  - Requirements: R1.5–R1.6, R3.5, R4, R8.2–R8.6, R9.1–R9.4, R10.1, R10.3, R13.1.
  - Depends on: T09, T13, T14.
  - Verify: `cargo test -p counterspell-engine hunt`.
  - Preserve: no panics on any request; infrastructure failures never become wrong answers.
  - Rust lesson: owning versus borrowing in a struct, `Box` in an enum, and `Result` conversion.

- [x] **T16 [S] (30 min) Hunt, part 2: shrink wiring, verification, fallback and timeouts**
  - Files: `crates/engine/src/hunt.rs`.
  - Wire `shrink::shrink` to a probe closure (clock check, Oracle call, classification equals the fixed predicate, Oracle errors remembered and turned into `Stop`). Add `verify_ghost` (R5.2, R5.5), the walk-back fallback over retained trace entries, `NotFound(Unreproducible)`, the timeout path (re-run once, no shrink, `NotAttempted`, `is_minimized() == false`), `budget_exhausted`, and ghost construction with sanitised results, autopsy, known-ghost match and provenance.
  - Tests: end-to-end with synthetic oracles: wrong answer found and shrunk; crash found and shrunk; flaky oracle triggers fallback; always-flaky gives `Unreproducible`; timeout ghost is not shrunk; shrink budget stop sets `budget_exhausted`; oracle error mid-shrink aborts.
  - Requirements: R5, R7.4, R7.7, R9.5, R9.8, R10, R11, R13.
  - Depends on: T12, T15.
  - Verify: `cargo test -p counterspell-engine hunt`.
  - Preserve: the shrink module is called only through its public callback API.
  - Rust lesson: closures borrowing `&mut` state, and returning early with `match`.

## Wave 3: test packs (no Rust, no execution of Python)

- [x] **T17 [P] (30 min) Pack `pair-count`**
  - Files: `packs/_test/pair-count/{problem.md,ghost.json,golden.json,reference/solution.py,incorrect/same-index.py,incorrect/empty-min.py}`.
  - Write the original statement; the reference and the two incorrect programs exactly as in the design; `ghost.json` with `problem_version: 1`, schema, comparator `exact`, concept tags, three samples, known ghosts for `[[0],0]` (got 1) and `[[],0]` (class `ValueError`) with authored explanations, and the `incorrect` index (file, kind, class, expected smallest input, hand-traced `vectors`, `dogfood: true` on `same-index`); `golden.json` with at least eight hand-verified vectors including `[[1,2,3,4],5] → 2`, `[[0,0,0],0] → 3`, `[[],0] → 0`. Hand-traced bug vectors: `same-index`: `[[0],0] → 1`, `[[1,2,3,4],5] → 2`, `[[2,2],4] → 3`; `empty-min`: `[[],0] → ValueError`, `[[1,2,3,4],5] → 2`.
  - Requirements: R14.
  - Depends on: none.
  - Verify: the JSON files parse (the T20 loader checks this later); Brighton reviews the statement, vectors and explanations against the Python source by hand.
  - Preserve: original wording; no copied problem text; Python is never run.
  - Rust lesson: not applicable (no Rust change). Write one line saying so, as the repo-skeleton tasks do.

- [x] **T18 [P] (30 min) Pack `longest-run`**
  - Files: `packs/_test/longest-run/…` (same layout; incorrect: `forgot-last-run.py`, `empty-first-char.py`).
  - Alphabet `abc`, length 0–40. Known ghosts: `["a"]` (got 0) and `[""]` (class `IndexError`). Golden vectors include `"aabbb" → 3`, `"" → 0`, `"abc" → 1`, `"cccc" → 4`, `"abba" → 2`. Hand-traced bug vectors: `forgot-last-run`: `"a" → 0`, `"aabbb" → 2`, `"abc" → 1`; `empty-first-char`: `"" → IndexError`, `"aabbb" → 3`.
  - Requirements: R14.
  - Depends on: none.
  - Verify and Preserve: as T17.
  - Rust lesson: not applicable.

- [x] **T19 [P] (30 min) Pack `diagonal-sum`**
  - Files: `packs/_test/diagonal-sum/…` (incorrect: `abs-diagonal.py`, `assumes-square.py`).
  - Matrix rows 0–6, columns 0–6, cells −9 to 9. Known ghosts: `[[[-1]]]` (got 1) and `[[[]]]` (class `IndexError`). Golden vectors include `[[1,2],[3,4]] → 5`, `[[]] → 0`, `[] → 0`, `[[1,2,3]] → 1`, `[[1],[2]] → 1`, `[[-3]] → -3`, `[[1,2],[3,4],[5,6]] → 5`. Hand-traced bug vectors: `abs-diagonal`: `[[-3]] → 3`, `[[1,2],[3,4]] → 5`; `assumes-square`: `[[1],[2]] → IndexError`, `[[1,2],[3,4]] → 5`, `[[1,2,3],[4,5,6]] → 6`.
  - Requirements: R14.
  - Depends on: none.
  - Verify and Preserve: as T17.
  - Rust lesson: not applicable.

## Wave 4: integration tests and documentation

- [x] **T20 [S] (30 min) Test support: pack loader and model oracles**
  - Files: `crates/engine/tests/support/mod.rs`.
  - Load `ghost.json`/`golden.json` via `env!("CARGO_MANIFEST_DIR")` and `serde_json`. Write the reference model and six buggy models as Rust functions mirroring the Python. Write `ModelOracle` (implements `Oracle`, charges a fixed cost on a `ManualClock`, returns `TimedOut` if the cost exceeds the limit), plus synthetic-oracle builders and proptest strategies shared by `properties.rs` and `packs.rs`.
  - Tests: every model reproduces its golden or hand-traced vectors exactly (R14, design "drift" control 1).
  - Requirements: R14.8, R16, R15.
  - Depends on: T16, T17, T18, T19.
  - Verify: `cargo test -p counterspell-engine --test packs vectors`.
  - Preserve: support code executes only Rust; Python files are read as text if at all.
  - Rust lesson: integration-test layout, `mod support;`, and function pointers (`fn(&Value) -> …`).

- [x] **T21 [P] (30 min) Hunt-level properties**
  - Files: `crates/engine/tests/properties.rs`.
  - Implement P1 (stream-level), P6, P8, P9, P10 with synthetic oracles (always fail, never fail, fail only on large inputs, charge clock time, reference fails). Use `with_cases(64)`. Commit generated regression files.
  - Requirements: R15.1, R15.5, R15.7–R15.9, R8.7.
  - Depends on: T20.
  - Verify: `cargo test -p counterspell-engine --test properties`, under 30 s.
  - Preserve: no real clock in properties; the manual clock keeps them deterministic.
  - Rust lesson: how `proptest` explores inputs and shrinks failing cases (and the pun with our own shrinker).

- [x] **T22 [S] (30 min) Definition-of-done tests for the packs**
  - Files: `crates/engine/tests/packs.rs`.
  - Static checks (files exist, incorrect files listed, same function name as reference by text scan). For all six incorrect programs: hunt, assert `Found`, kind and class, completeness of provenance, `contains`, fresh-oracle `verify_ghost`, independent re-check (model functions and `==`, not `classify`), the exact expected smallest input from the design's table, repeat-run equality, trace invariants. For each reference hunted against itself: assert `NotFound` with the expected reason text. If a result differs from the hand-derived table, stop and report; do not loosen the assertion.
  - Requirements: R14.9, R16.1–R16.6.
  - Depends on: T20.
  - Verify: `cargo test -p counterspell-engine --test packs`.
  - Preserve: no host execution of Python; assertions say "regression expectation", not "global minimum".
  - Rust lesson: iterating over test cases, `assert_eq!` messages, and why independent checks catch shared bugs.

- [x] **T23 [P] (20 min) Purity guard test**
  - Files: `crates/engine/tests/purity.rs`.
  - Fail if any file in `src/` contains `std::process`, `Command::new`, `std::fs`, `std::net`, `tokio`, `bollard`, `std::thread`, `unsafe`, `.unwrap()` or `.expect(` outside `#[cfg(test)]` blocks (accept a simple, documented approximation), and if the manifest's `[dependencies]` names anything but `serde` and `serde_json`, or `[dev-dependencies]` anything but `proptest`.
  - Requirements: R1.1, R1.2, R1.9, R17.3.
  - Depends on: T01.
  - Verify: `cargo test -p counterspell-engine --test purity`; temporarily adding a forbidden token makes it fail (check, then revert).
  - Preserve: the guard is a belt-and-braces check, not a replacement for review.
  - Rust lesson: reading files in tests and why "executes nothing" is checked mechanically.

- [x] **T24 [P] (25 min) Wording and serialisation tests (P14, P15)**
  - Files: `crates/engine/tests/properties.rs` (or a small `tests/wording.rs`).
  - Round-trip `HuntReport`, `Ghost`, `Schema`, `Budget` through JSON; check the JSON field names match the `ghosts` table (`kind`-derived value `wrong|crash|timeout`, `shrink_trace` content, `seed`, `generator_version`, `problem_version`, `image_digest`, `engine_version`); scan every template and `NotFoundReason` text for the forbidden phrases; check `magnitude` serialises as a string.
  - Requirements: R9.3, R9.6–R9.8, R15.12.
  - Depends on: T16.
  - Verify: `cargo test -p counterspell-engine --test properties wording`.
  - Preserve: wording constants live in one place.
  - Rust lesson: `serde` derive, `rename`, and testing a JSON contract.

- [x] **T25 [P] (25 min) Documentation**
  - Files: `crates/engine/src/lib.rs` (crate docs with a runnable-in-tests usage example that uses a fake Oracle), `packs/_test/README.md`.
  - Crate docs: the three parts, the Oracle contract, what "minimized counterexample" means, and the public surface table. README: purpose, `_` prefix rule, layout, the model-oracle limits and the conformance hand-off from R16.8 and the design, and the dogfood candidate.
  - Requirements: R16.8, R18.
  - Depends on: T22.
  - Verify: `cargo doc -p counterspell-engine --no-deps` has no warnings; `cargo test -p counterspell-engine --doc`.
  - Preserve: docs never say "smallest possible" or imply correctness when nothing is found.
  - Rust lesson: doc comments and doctests.

- [ ] **T26 [S] (30 min) Full verification gate**
  - Files: none planned; fixes only if a check fails.
  - Run: `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; `cargo test --workspace`; `cargo deny check licenses`. Confirm the workspace still has exactly six packages and that only `crates/engine/`, `packs/_test/`, `THIRD-PARTY-NOTICES.md`, `Cargo.lock` and `docs/kiro-log/rust-lessons.md` changed. Confirm proptest regression files are committed. Time the engine tests (target under 60 s).
  - Requirements: R16.7, R15, R18.
  - Depends on: T21, T22, T23, T24, T25.
  - Verify: every command passes.
  - Preserve: no later-spec work started (no judge, no MCP, no UI).
  - Rust lesson: why the CI gates (fmt, Clippy, tests, licences) are cheap insurance.

## Dependency summary and parallel waves

- **Wave 0:** T01 first. Then T02, T03, T05, T06 in parallel. T04 after T03; T07 after T06.
- **Wave 1:** T08 after T02 and T04; T09 after T08. T10 after T04 and T05; T11 after T10; T12 after T11. The generator chain (T08–T09) and the shrinker chain (T10–T12) run in parallel.
- **Wave 2:** T13 after T01 (parallel with Wave 1). T14 after T05 and T07. T15 after T09, T13, T14. T16 after T12 and T15.
- **Wave 3:** T17, T18, T19 have no Rust dependencies and can run at any time, even before Wave 0.
- **Wave 4:** T20 after T16–T19. T21 and T22 after T20 (T22 also checks pack content). T23 after T01. T24 after T16. T25 after T22. T26 last.

## Completion checklist

- [ ] The engine crate is still `counterspell-engine` in `crates/engine/`, with only `serde`, `serde_json` and (dev) `proptest`.
- [ ] Generation, classification and shrinking live in separate modules; `shrink.rs` imports only `schema` and `size`.
- [ ] Nothing in the crate executes code (purity test passes).
- [ ] All 15 correctness properties pass, with regression files committed.
- [ ] Three packs under `packs/_test/`, each with a Python reference, two incorrect programs, vectors and known ghosts; one program flagged for the dogfooding demo.
- [ ] All six incorrect programs produce valid, reproducible minimized counterexamples within budget, and each passes `verify_ghost` and the independent re-check.
- [ ] References hunted against themselves give a clear "not found" result.
- [ ] Every result carries seed, generator version, engine version, problem version and image digest.
- [ ] No engine string claims global minimality or correctness.
- [ ] `packs/_test/README.md` states the model-oracle limit and the sandbox conformance hand-off.
- [ ] fmt, Clippy (`-D warnings`), workspace tests and `cargo deny check licenses` are green.
