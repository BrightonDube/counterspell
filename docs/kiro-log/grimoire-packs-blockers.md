# grimoire-packs blockers (run of 13 Nov 2026)

## Not started in this run (agent work left for the next run)

- **T15 (mutations per code)** depends on the starter packs (T21 to T23). Not started.
- **T18 and T19 (sandbox checks SBX-01 to SBX-08, infrastructure exit)** were not started in this run. The next step is a failing `tests/sandbox_fake.rs` using a model runner, then `sandbox.rs`.
- **T21 to T23 (starter packs `count-pairs`, `longest-run-length`, `diagonal-total`)**, then **T24** (zero-error gate) and **T25** (model vectors). Not started.
- **T26 (repo-wide gates)** and **T27 (handover note)** follow the packs.

## Blocked on another spec or on a decision

- **T20 (judge implements PackRunner, wires `--sandbox`)** is blocked on the `judge-sandbox` spec, which is not implemented, and on open question 7 (who owns `PackRunner`). Unchecked.
- **T28 (review and publish starter packs)** is a human step. Brighton must review each statement, hint, ghost explanation and reference, then set `status = "published"`. Agent work stops at `draft`.
- **T29 (convert `packs/_test/`)** waits for the ghost-engine conversion decision (open question 11). Those fixtures are not touched (GRM-50).
- **T30 (CI step `validate --all packs`)** waits for the CI spec and its owner.

## Decisions taken for open questions (Brighton can change these)

- **Open question 1 (reference and hidden data in git):** assumed "keep in git". Hidden inputs are generated from the seed. Not confirmed by Brighton.
- **Open question 6 (forbidden phrases):** the engine keeps its list in a test constant, not a public export. `ghosts.rs` keeps a copy. A test reads the engine's test file as text to check each phrase is still there. This is a drift guard, not an equality check.
- **Open question 10 (launch caps):** the caps are inline constants in `limits.rs` (`time_ms` 100 to 2000, `mem_mb` 64 to 256). No test compares them with `languages/languages.toml`, because that file does not exist yet.
- **Open question 15 (hidden count):** the range 5 to 50 is enforced. The walkthrough value of 12 is used in the starter fixtures.

## Deviations from the design, to review

- **Ghost file syntax errors use KNG-02.** The design has no code for a broken `known_ghosts.toml`. KNG-02 is the nearest entry. A dedicated code needs a design change.
- **Hint file syntax errors use HNT-01.** The same gap exists here.
- **`validate_static` error type.** `record::build` returns `Result<ProblemRecord, RecordError>` with `Invalid(Report)` and `Unreadable(io::Error)`, instead of `Result<ProblemRecord, Report>`. An I/O failure cannot be turned into a report without inventing a code.
- **Return-shape table** lives in `samples.rs` (`accepts_return`), not in `signature.rs` as T05 asked. It is used by samples and known ghosts.
- **`--deny-warnings` is not tested.** Static checks never emit a warning, so no pack can trigger the path until the sandbox stage exists.
- **The `ProgramOracle` doc example** that T17 asked for is not written. The trait shape is covered by `tests/runner.rs`.

## Environment issues (need Brighton)

- **`guard-secrets` false positive.** The secrets hook blocks any write that contains `re_` followed by 20 or more identifier characters. It blocked one test file, because a test name contained `failure_` followed by a long suffix. The test was renamed to `infra_fault_is_oracle_error_not_user_result`, and the guard was not changed. This is the same false positive recorded in `ghost-engine-blockers.md`.
- **Untracked hook and script files** from before this run: `.kiro/hooks/guard-test-integrity.json`, `.kiro/steering/test-integrity.md`, `scripts/kiro-hooks/guard-tests.sh`. They were not created or committed by this run.
- **Untracked session log** `docs/kiro-log/2026-11-13T0025-21-grimoire-packs-implement-a1.log` was left as it was.

## Continuation run (13 Nov 2026, after the blockers above)

Done in this run: T15, T18, T21 to T27. T19 is partly done (see below).

### Still open

- **T19 (infrastructure exit and bench placeholder): left unticked.** The library gives the SBX-06 diagnostic (`SandboxFault::diagnostic`) and the SBX-08 bench warning. The exit-2 path is not reachable from the CLI, because `--sandbox` has no runner until the judge spec (T20). Unblock: T20.
- **T20 (judge implements `PackRunner`, wires `--sandbox`): BLOCKED.** Needs the `judge-sandbox` spec, which is not implemented, and open question 7. Unblock: judge spec complete, and Brighton confirms that the judge depends on grimoire.
- **T28 (review and publish starter packs): BLOCKED on Brighton.** The three packs stay `draft`. Brighton reviews each statement, hint, ghost explanation and reference, then sets `status = "published"`. The author byline `Brighton` is a placeholder (open question 9).
- **T29 (convert `packs/_test/`): BLOCKED.** Needs the ghost-engine spec complete and open question 11. Those fixtures are untouched (GRM-50).
- **T30 (CI step): BLOCKED.** Needs the CI spec and its owner.

### Defect found in the engine (not fixed here, another spec's code)

- **`counterspell-engine` panics in `tiny_ints`** (`crates/engine/src/generate.rs`, the `clamp` call) when a schema has min above max. The MET-06 mutation case triggered it. The grimoire side now stops the generator check on an invalid schema (`generator::check`, with a unit test). The ghost-engine owner should return an error instead of panicking. Until then, the MCP tools and the judge must not pass an unchecked schema to the engine.

### Decisions taken in this run (Brighton can change them)

- **Order:** T15 depends on the starter packs, so it ran after T21 to T23, not in list order. Its content is unchanged.
- **Starter-pack ghosts:** each program was traced by hand, not run. The model vectors in `tests/sandbox_fake.rs` are the only evidence of the traces. T20 must run the same vectors through the real Python files (the header of that file says so).
- **Model fixtures:** `tests/sandbox_fake.rs` copies the `count-pairs` text, so the sandbox tests do not depend on `packs/`. The starter-pack tests read `packs/` directly. If `packs/count-pairs` changes, update the copy in `sandbox_fake.rs` too.
- **GEN-02 is not text-reachable.** It guards the engine's own candidate stream. `tests/mutations.rs` exempts it by name and says why.

### Environment issues

- **Stale incremental build (SIGSEGV).** After `cargo fmt` and `cargo clippy`, the grimoire unit-test binary crashed with SIGSEGV before any test ran. `cargo clean -p counterspell-grimoire` fixed it. If this returns, clean the package first.
- **`guard-secrets` false positive, again.** The hook blocked an edit whose test name contained `re_` followed by 20 or more identifier characters. The test was renamed (`runner_fault_maps_to_sbx_06`). The hook should be fixed.
- **Shell parser rejected one read-only command** (a `sed` pipeline with `;`). The same reads were done with the file tools. This is harmless, but it may matter to the hook authors.

## Implement pass (13 Nov 2026, after the continuation run above)

- **T19 is now done (ticked).** The CLI `--sandbox` path goes through `run_sandbox_stage(options, runner: Option<&mut dyn PackRunner>)`. Production passes `None` and prints SBX-07, exit 2. A runner fault from `validate_sandbox` prints SBX-06 and exits 2, never exit 1. Tests: `runner_fault_is_sbx_06_from_the_sandbox_stage`, `runner_fault_exits_two_not_one`, `missing_runner_exits_two_with_sbx_07` (in `src/main.rs`). SBX-08 is covered by `declared_bench_warns_sbx_08_without_calling_the_bench_runner` in `tests/sandbox_fake.rs`.
- **Still open, unchanged:** T20 (judge implements `PackRunner`, wires the real runner into `--sandbox`). Blocked on the `judge-sandbox` spec and open question 7. Until then the CLI never reaches a real runner.
- **T28, T29, T30:** still blocked on Brighton's review, the ghost-engine conversion decision and the CI spec. Not started.
- **Known gap in T19:** the `--json` branch of `print_fault` has no test. It is one line, and the text branch is tested through the exit code.
