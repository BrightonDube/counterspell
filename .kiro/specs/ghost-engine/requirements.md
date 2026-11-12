# Requirements: ghost-engine

## Introduction

The ghost engine is the pure Rust library in `crates/engine` (package `counterspell-engine`). When a learner's solution is wrong, the engine finds an input that breaks it, shrinks that input, and returns a **minimized counterexample** (the "ghost input") with everything the UI needs for the shrink animation and the Bug Autopsy card.

The engine has three clearly separated parts: **input generation**, **failure classification**, and **shrinking**. A thin orchestration layer (`hunt`) joins them. The engine never runs code. It talks to the user's program and the trusted Python reference only through an `Oracle` trait, which the trusted judge implements on top of the sandbox.

Property-based testing (PBT) with `proptest` is **enabled** for this spec. Correctness properties are listed in `design.md` and map back to the requirements below.

Sources: `docs/system-design.md` §8 (Ghost engine) and §5 (Ghost hunt protocol), `docs/requirements.md` GHO-1, GHO-5, GHO-6 and the Sandbox criteria, `docs/product-brief.md`, and the steering files `ghost-engine.md`, `sandbox.md`, `security.md`, `grimoire-authoring.md`, `rust-teaching.md`.

## Glossary

- **Oracle**: the trait through which the engine asks "what do the user's program and the reference do on this input?". Implemented by the judge with the sandbox. In tests it is implemented by pure-Rust model functions.
- **Observation**: the Oracle's answer for one input: the user's raw result and the reference's raw result.
- **Candidate**: an input value produced by the generator (or supplied as a sample or known-ghost seed) that has passed the schema validator.
- **Failure predicate**: the pair (failure kind, exception class). Kinds are `wrong`, `crash`, `timeout`. The exception class applies to `crash` only.
- **Ghost / minimized counterexample**: the smallest failing input the engine *found* within its budget that reproduces the failure predicate. It is never claimed to be the smallest possible input.
- **Size metric**: the explicit, deterministic, total order on input values defined in R6.
- **Shrink trace**: the ordered, capped list of accepted shrink steps, used by the UI to animate the shrink.
- **Budget**: candidate, evaluation and wall-time limits that bound a hunt (R8).
- **Provenance**: seed, generator version, engine version, problem version and runtime image digest recorded with every result.
- **Known ghost**: an authored failing input in a pack with an authored explanation of the conceptual mistake.
- **Test pack**: a pack under `packs/_test/` used only to test the engine and for the Kiro dogfooding demo (R14).
- **Model oracle**: a pure-Rust test double that mirrors a pack's Python reference or incorrect program, so the engine can be tested without executing Python (R14, R16).

## Scope constraints

- The crate SHALL stay `counterspell-engine` at `crates/engine/`. No new Cargo package, crate or parallel layout is created.
- Postgres, Valkey, SSE, the job queue, the sandbox, language drivers, MCP tools and the UI are other specs. This spec defines only the library, its tests, and three test packs.
- Supported input kinds in this spec: bounded integers, strings, integer arrays, 2-D integer arrays (matrices), and tuples of these.

## Requirements

### R1. Purity and the Oracle seam

**User story:** As the operator, I want the engine to be unable to run code, so that all untrusted execution stays inside the judge's sandbox.

1. THE engine crate SHALL NOT spawn processes, open sockets, read or write files, call a Docker API, or depend on Tokio, bollard, sqlx, or any HTTP client.
2. THE engine's non-test dependencies SHALL be limited to `serde` and `serde_json`. Test-only dependencies SHALL be limited to `proptest`.
3. THE engine SHALL reach the user's program and the reference program only through the `Oracle` trait.
4. WHEN the engine asks the Oracle to evaluate one input, THE Oracle SHALL return an Observation holding the user program's result and the trusted reference's result for that same input. The engine SHALL NOT obtain expected values from any other source.
5. THE engine SHALL pass the Oracle a time limit for each call, and THE Oracle SHALL enforce it by killing the program from outside, as the judge does for per-test deadlines.
6. IF the Oracle reports an infrastructure error (for example, the sandbox daemon or image failed), THEN THE engine SHALL stop and return `Aborted`. It SHALL NOT report a wrong-answer ghost or a "not found" result, because infrastructure failure is never the user's wrong answer.
7. THE engine SHALL be synchronous and single-threaded. It SHALL NOT require an async runtime.
8. THE engine SHALL receive only public inputs: generated candidates, public samples, and authored known-ghost seeds. It SHALL NOT receive hidden test inputs or hidden expected outputs.
9. A test in the crate SHALL fail if any file under `crates/engine/src/` uses process, filesystem or network APIs, or if the manifest gains a non-allowed dependency.

### R2. Input schema and validators

**User story:** As a pack author, I want to declare input constraints once, so that every generated and shrunk input is legal for the problem.

1. THE engine SHALL support these schema kinds: bounded integer, string (ASCII alphabet, length range), integer array (length range, element range, optional `sorted` and `unique` flags), 2-D integer array (row-count range, column-count range, element range, rectangular), and tuple of the non-tuple kinds.
2. THE engine SHALL NOT support graphs, trees, floats, ragged matrices, nested tuples, or executable custom checkers in this spec. A schema document that asks for them SHALL be rejected with a clear error.
3. WHEN a schema definition is loaded, THE engine SHALL validate it: bounds ordered (`min <= max`), integer bounds within ±(2^53 − 1) so values are safe in JavaScript, alphabets non-empty, distinct and printable ASCII, and `unique` arrays able to reach `min_len` from their element range.
4. THE engine SHALL provide a validator `contains(schema, value)` that reports `Ok` or a `Violation` naming the JSON path and the broken constraint.
5. THE validator SHALL be implemented separately from the generator and the shrinker, so a generator bug cannot hide itself.
6. THE validator SHALL reject non-integer numbers, integers outside bounds, characters outside the alphabet, wrong lengths, ragged matrices, wrong tuple arity, unsorted values when `sorted` is set, and duplicate values when `unique` is set.
7. THE validator SHALL never panic on any JSON value, including deeply nested or very large values.
8. THE schema SHALL have a JSON form using `serde`, with unknown fields rejected.

### R3. Seeded, deterministic, edge-biased generation

**User story:** As a learner, I want the same code to produce the same ghost every time, so that I can trust and reproduce it.

1. THE engine SHALL derive the default seed as a stable 64-bit hash of (problem slug, code SHA-256, language, engine version). THE hash SHALL NOT use Rust's `DefaultHasher`, whose output is not stable across releases.
2. WHERE a seed override is supplied (for replay or tooling), THE engine SHALL use it and record it.
3. THE engine SHALL use its own small seeded PRNG. The same seed SHALL produce the same candidate sequence on every platform.
4. EVERY candidate SHALL satisfy `contains(schema, candidate)` before it is evaluated. Candidates that fail validation SHALL be dropped and counted, never evaluated.
5. THE candidate stream SHALL follow this order: known-ghost seeds, samples, a tiny exhaustive enumeration, then edge-biased random candidates of growing size.
6. THE random stage SHALL favour edge values: empty and single-element collections, all-equal elements, bounds, zero, ±1, negatives, duplicates, and sorted or reverse-sorted runs, where the schema allows them.
7. THE stream SHALL not yield the same value twice.
8. UNLESS the request targets a timeout, THE generator SHALL cap each candidate at 256 scalar units, because ghost inputs are meant to be small.

### R4. Failure classification and comparators

**User story:** As the operator, I want failures classified by one trusted rule, so that "same failure" has a precise meaning.

1. THE engine SHALL classify an Observation as `Pass`, `Fail(predicate)`, or `ReferenceFailed`.
2. WHEN the reference result is anything other than a returned value, THE engine SHALL classify the Observation as `ReferenceFailed`. Such candidates SHALL never be reported as the user's failure.
3. WHEN both programs return values and the comparator says they differ, THE engine SHALL classify `Fail(wrong)`.
4. WHEN the user's program crashes, THE engine SHALL classify `Fail(crash, exception_class)`.
5. WHEN the user's program times out, THE engine SHALL classify `Fail(timeout)`.
6. WHEN the user's program prints output that is not a valid answer, THE engine SHALL treat it as a wrong answer. Text that looks like a verdict is only an answer to compare.
7. THE engine SHALL provide two comparators: `exact` (strict JSON equality, so `1` and `1.0` differ) and `unordered` (top-level arrays compared as multisets; anything else compared exactly).
8. THE engine SHALL treat exception classes as untrusted text. It SHALL accept only `[A-Za-z_][A-Za-z0-9_.]{0,63}` and SHALL replace anything else with `UnknownError`. It SHALL truncate crash messages to 200 bytes and strip control characters.
9. THE engine SHALL NOT use the user's program output to decide any verdict beyond the comparison above.

### R5. Reference-verified failures and reproduction

**User story:** As a learner, I want every ghost to be a real failure against the trusted reference, so that I never chase a phantom.

1. THE engine SHALL report a ghost only when the Oracle's reference result, compared by the engine's comparator, disagrees with the user's result (or the user crashed or timed out where the reference returned a value).
2. BEFORE returning a ghost, THE engine SHALL re-evaluate the final input through the Oracle `verify_runs` times (default 2) and require the same failure predicate, the same expected value and the same actual result each time.
3. IF the final input fails verification, THEN THE engine SHALL walk back through the retained trace entries, newest first, and return the first one that verifies.
4. IF no retained entry verifies, THEN THE engine SHALL return `NotFound(Unreproducible)` and SHALL say the failure was seen but did not reproduce.
5. THE engine SHALL expose `verify_ghost` so the judge and tests can re-check a stored ghost with a fresh Oracle at any time.
6. WHERE a hunt has a target predicate from the original failure, THE engine SHALL only accept ghosts that reproduce exactly that predicate.

### R6. Deterministic size metric

**User story:** As a reviewer, I want "smaller" to have one precise meaning, so that shrinking can be checked mechanically.

1. THE engine SHALL define the size of a value as the triple (`nodes`, `magnitude`, canonical text), compared lexicographically.
2. `nodes` SHALL count JSON nodes: a number counts 1, a string counts 1 plus its character count, and an array or tuple counts 1 plus the sum of its children.
3. `magnitude` SHALL be the sum of the absolute values of all integers plus the Unicode code point of every string character, held in a `u128`.
4. THE canonical text SHALL be the compact `serde_json` serialisation, compared byte by byte.
5. THE order SHALL be a strict total order over distinct values and SHALL be well-founded, so any strictly decreasing sequence of valid values is finite.
6. THE engine SHALL record `nodes` and `magnitude` with every ghost and every trace step. `magnitude` SHALL be serialised as a decimal string.

### R7. Shrinking

**User story:** As a learner, I want the ghost to be small enough to read and reason about.

1. THE shrinking algorithm SHALL be pure: it SHALL take a schema, a starting value and a probe callback, and SHALL NOT know about the Oracle, the classifier, the clock or any program.
2. THE shrinker SHALL try, in this order: whole-value integer moves (all integers to their target, then all halved toward it), structural removal (chunk removal on arrays, strings, matrix rows and matrix columns, from large chunks down to single items), and scalar simplification (each integer toward its target by jump, halving and single step; each character toward the first alphabet character).
3. THE shrinker SHALL evaluate a proposal only if it satisfies the schema validator AND is strictly smaller than the current value under the size metric.
4. THE hunt SHALL keep a proposal only if the probe reports that it still reproduces the same failure predicate (kind plus exception class) as the first failing input.
5. THE shrinker SHALL never evaluate the same value twice in one run.
6. WHEN no valid, strictly smaller proposal reproduces the failure, THE shrinker SHALL stop with `LocalMinimum`. A local minimum means no single move in the shrinker's repertoire helps. It does not mean no smaller failing input exists.
7. THE shrinker SHALL stop with `EvalBudget` when it has used `max_shrink_evals`, and with `WallTime` when the probe asks it to stop.
8. THE shrinker SHALL always terminate. Every accepted step strictly decreases a well-founded measure, and evaluations are bounded.

### R8. Budgets and termination

**User story:** As the operator, I want a hard cap on how long a ghost hunt can take.

1. THE default budget SHALL be: 500 candidates and 4 s for the search, 300 evaluations and 3 s for shrinking, 8 s total, a 1 s per-evaluation limit, 2 verification runs, and a 1 s reserve for verification.
2. THE engine SHALL reject a budget whose total wall time exceeds 8 s, or whose search and shrink deadlines leave no verification reserve, with `Aborted(InvalidRequest)`.
3. THE search deadline SHALL be `min(search_wall, total_wall − verify_reserve)`. THE shrink deadline SHALL be `min(shrink_start + shrink_wall, total_wall − verify_reserve)`.
4. THE engine SHALL read time only through a `Clock` trait, so tests can drive a manual clock and stay deterministic.
5. BEFORE each Oracle call, THE engine SHALL pass a limit of `min(eval_limit, time remaining until the current phase deadline)` and SHALL not call the Oracle at all when no time remains.
6. WHEN any budget is reached, THE engine SHALL stop that phase and report which budget stopped it.
7. THE number of Oracle calls in one hunt SHALL not exceed `max_candidates + max_shrink_evals + verify_runs × 21` (the 20 retained trace entries plus the final input), and the hunt SHALL finish within `total_wall`, provided the Oracle honours the limit it is given.

### R9. Results, status and honest wording

**User story:** As a learner, I want honest feedback, including when no ghost was found.

1. THE engine SHALL return a `HuntReport` for every request. It SHALL never panic and SHALL never return a bare error.
2. THE report SHALL hold exactly one outcome: `Found(ghost)`, `NotFound(reason)`, or `Aborted(reason)`.
3. `NotFound` reasons SHALL be `CandidateBudget`, `WallTime`, `CandidatesExhausted` and `Unreproducible`. Each SHALL have display text that says no failing input was found within the budget. None SHALL say or imply that the solution is correct.
4. `Aborted` reasons SHALL be `OracleFailure`, `ReferenceUnreliable` and `InvalidRequest`. They SHALL map to an internal error in the judge, never to a wrong answer.
5. A found ghost SHALL carry `budget_exhausted = true` when shrinking stopped on a budget instead of reaching a local minimum.
6. ALL user-facing text produced by the engine SHALL call a shrunk result a "minimized counterexample". It SHALL NOT contain "smallest possible", "global", "proven", "optimal" or "the minimum".
7. THE engine SHALL describe a ghost as the smallest failing input *found* within its budget.
8. ALL result types SHALL serialise to and from JSON without loss, so the judge can store them as JSON columns.

### R10. Provenance and reproducibility

1. EVERY `HuntReport`, including `NotFound` and `Aborted`, SHALL carry provenance: seed, generator version, engine version, problem version and runtime image digest.
2. THE generator version SHALL be a constant that is changed whenever candidate generation order, edge-bias rules, shrink move order or the size metric changes.
3. THE problem version and image digest SHALL be supplied by the caller and echoed unchanged. THE engine SHALL reject an empty image digest.
4. GIVEN equal requests, equal seeds and equal deterministic Oracle behaviour, THE engine SHALL return equal outcomes and equal provenance.
5. THE report SHALL echo the budget used and the counts of candidates, shrink evaluations and verification runs.

### R11. Shrink trace

**User story:** As a learner, I want to watch the failing input shrink, step by recorded step.

1. THE engine SHALL record the first failing input as step 0 and then every accepted shrink step, in order.
2. EACH step SHALL hold its index in the full sequence, the input, its size metric and the evaluation count at acceptance.
3. THE trace SHALL keep at most 20 entries: the first 10 and the last 10. THE report SHALL state the total step count and how many were omitted between them.
4. THE last trace entry SHALL always equal the ghost's input, and the first SHALL always equal the first failing input found.
5. THE engine SHALL record only steps it actually accepted and SHALL NOT invent intermediate inputs.
6. EVERY entry SHALL be strictly smaller than the one before it under the size metric.

### R12. Bug Autopsy data

**User story:** As a learner, I want the card to show what failed and why, using only data the engine really observed.

1. THE ghost SHALL carry the input, the expected value, the actual result (a value, a crash with exception class and message, a timeout, or garbled output), and the failure kind.
2. THE engine SHALL compute a structural difference between expected and actual values: a list of JSON-pointer paths with the expected and actual values at each, capped at 20 entries with a truncation flag.
3. THE engine SHALL produce a one-line plain-text explanation, for example "Your solution returned 3; expected 2 for input [[0, 0], 0].".
4. THE engine SHALL carry the problem's concept tags into the autopsy.
5. WHEN the final ghost matches a known ghost, THE engine SHALL attach that known ghost's id and authored explanation, marked as authored.
6. A ghost SHALL match a known ghost only if the inputs are equal, the kinds are equal, the exception classes are equal when the known ghost names one, and the actual values are equal when the known ghost gives one.
7. WHEN no known ghost matches, THE autopsy SHALL carry no mistake text. The server fills in a labelled AI explanation later.
8. THE autopsy SHALL contain only the ghost's own data. It SHALL NOT contain hidden test inputs or hidden expected outputs.

### R13. Timeouts handled separately

1. WHEN the hunt target is a wrong answer or a crash, THE engine SHALL count timeouts it meets, skip them, and never accept them as the ghost or as a shrink step.
2. WHEN the hunt target is a timeout, THE engine SHALL look for a timing-out input, re-run it to confirm it times out again, and return it with `kind = timeout`.
3. THE engine SHALL NOT shrink timeout ghosts in this spec. Their shrink report SHALL say `NotAttempted`, and the engine SHALL NOT call them minimized counterexamples.
4. FOR timeout hunts, THE per-evaluation limit SHALL be the caller's real problem time limit, so a reported timeout would also be a timeout in judging.
5. FOR timeout hunts, THE generator MAY grow toward the schema's own maximum sizes, up to a hard cap of 100,000 scalar units.

### R14. Test problem packs

**User story:** As a developer, I want small original problems with known bugs, so that I can prove the engine finds and shrinks real mistakes.

1. THE repository SHALL contain exactly three test packs under `packs/_test/`: `pair-count`, `longest-run` and `diagonal-sum`.
2. EACH pack SHALL contain an original statement, a correct Python reference, at least two deliberately incorrect Python implementations, a machine-readable engine data file, hand-verified golden vectors, and known-ghost seeds with authored explanations.
3. TOGETHER the packs SHALL cover every supported schema kind: tuple, integer, integer array (`pair-count`), string (`longest-run`) and matrix (`diagonal-sum`).
4. TOGETHER the incorrect implementations SHALL include at least two wrong-answer bugs and at least two runtime-error bugs with different exception classes.
5. EXACTLY one incorrect implementation SHALL be marked as the Kiro dogfooding demo candidate.
6. THE statements and code SHALL be original. They SHALL NOT copy HackerRank, LeetCode or other protected text.
7. THE catalogue and any future pack importer SHALL skip directories whose name starts with `_`.
8. THE Python files SHALL be read only as text in this spec. No test or task SHALL execute them on the host.
9. A static test SHALL check each pack's files exist, each incorrect file is listed in the engine data file, and each file defines the same function name as the reference.

### R15. Property-based tests

PBT is enabled. Each property SHALL be a `proptest` test with a fixed regression file committed under `crates/engine/proptest-regressions/`.

1. Generated candidates satisfy their validators, for arbitrary valid schemas and seeds.
2. The validator rejects values that were valid and then broken in one controlled way (so it is not vacuous).
3. The size order is a strict total order (irreflexive, antisymmetric, transitive, total) on valid values.
4. Every shrink proposal that is evaluated satisfies the validator and is strictly smaller.
5. Every accepted shrink step strictly reduces the size metric, and every kept candidate reproduces the same predicate.
6. When shrinking stops with `LocalMinimum`, no valid strictly smaller single-move proposal reproduces the failure.
7. Every returned ghost reproduces its claimed failure on a fresh Oracle.
8. Equal seeds and equal deterministic Oracle behaviour give equal reports.
9. The search always terminates within its budgets, and within 8 s of manual-clock time, for Oracles that always fail, never fail, fail rarely, or consume clock time.
10. The trace cap invariants hold (R11).
11. The comparators obey their laws; `classify` and `contains` never panic on arbitrary values.
12. Every result type survives a JSON round trip.

### R16. Definition of done

1. WHEN the three packs' incorrect programs are run through their model oracles, THE engine SHALL return a `Found` ghost for every one of them within the default budget.
2. EACH ghost SHALL satisfy its schema, reproduce its failure on a fresh Oracle, have the expected failure kind and exception class, and carry complete provenance.
3. EACH ghost SHALL pass an independent re-check written in the test code, which calls the model functions directly and compares with plain `==`, bypassing the engine's classifier and comparator.
4. FOR each incorrect program where the design proves a unique smallest failing input, THE test SHALL assert the engine's result equals it. These are regression expectations for the shrinker's current moves, not claims of minimality.
5. RUNNING the same hunt twice SHALL give identical reports.
6. EVERY correct reference SHALL produce `NotFound` when hunted against itself, with a clear "no failing input found within the budget" reason.
7. THE crate SHALL pass `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace` and `cargo deny check licenses`.
8. THE limits of this definition SHALL be written in `packs/_test/README.md`: model oracles stand in for the sandbox here, and the real Python files are run through the sandbox in the `judge-sandbox` and `run-lifecycle` specs.

### R17. Robustness and untrusted data

1. THE engine SHALL treat Oracle output (values, exception classes, messages, snippets) as untrusted data.
2. THE engine SHALL bound every string it stores or returns: exception class 64 bytes, crash message 200 bytes, garbled snippet 200 bytes, explanation 240 bytes, identity strings 256 bytes each.
3. THE engine SHALL have no `unwrap()` or `expect()` outside tests, and no `unsafe` code.
4. THE engine SHALL cap memory by bounding candidate size (R3.8), trace size (R11.3), difference count (R12.2) and its de-duplication sets to the number of evaluations.
5. FOR unusually deep input values, THE validator SHALL fail fast on depth because schema depth is at most 3.

### R18. Small API and learning support

**User story:** As a developer new to Rust, I want a small, readable API.

1. THE crate root SHALL re-export a documented, small public surface (listed in `design.md`). Everything else SHALL be private to its module.
2. THE crate SHALL have `#![warn(missing_docs)]`, and every public item SHALL have a doc comment in plain English.
3. EVERY Rust task SHALL end with a short Rust lesson (3–5 lines) following the convention in `docs/kiro-log/rust-lessons.md`.
4. `design.md` SHALL explain the main design trade-offs in plain English.

## Traceability

| Source | Covered by |
|---|---|
| GHO-1.1 (find and shrink within 8 s) | R7, R8 |
| GHO-1.2 (same failure predicate, verified against the Python reference) | R4, R5, R7.4 |
| GHO-1.3 (strictly reduce size metric) | R6, R7.3 |
| GHO-1.4 (deterministic seed, record provenance) | R3, R10 |
| GHO-1.5 (inputs satisfy constraints) | R2, R3.4 |
| GHO-1.6 (never claim the smallest possible) | R9.6, R9.7 |
| GHO-1.7 (no ghost found, fall back to first failing sample) | R9.3; samples are early candidates in R3.5 |
| GHO-5 (shrink trace for animation) | R11 |
| GHO-6 (Bug Autopsy data) | R12 |
| System design §8 (Oracle, budgets, output) | R1, R8, R9, R10 |
| System design §5 ghost hunt protocol | R3.5, R4, R5 |
| Prompt items 1–10 | 1: R3.4. 2: R5. 3: R7.4. 4: R6. 5: R3, R10. 6: R4, R13. 7: R8. 8: R9.6–R9.7. 9: R9.3. 10: R11, R12 |

## Out of scope

- AI hints, the AI explanation for unmatched ghosts, and the Familiar.
- UI animation, the ghost card and the Bug Autopsy card (this spec returns their data only).
- Complexity profiling and bench mode.
- Language drivers, sandbox execution, the job queue, SSE and persistence (`judge-sandbox`, `language-drivers`, `run-lifecycle`).
- The final pack format and the `validate` command (`grimoire-packs`).
- MCP tools (`counterspell-mcp`), although the API is shaped so `find_ghost_input`, `shrink_input` and `explain_ghost` can wrap it.
- Graphs, trees, floats, ragged matrices, nested tuples, Unicode alphabets and custom checkers.
- Shrinking timeout ghosts.
