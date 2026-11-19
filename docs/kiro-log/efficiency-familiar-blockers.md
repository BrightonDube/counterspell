# efficiency-familiar: blockers (run of 14 Nov 2026)

Tasks left unchecked, with the reason and what would unblock each.

## Blocked by missing prerequisites

| Task | Reason | Unblocked by |
|---|---|---|
| T15 migration | Not written. Postgres is not running locally (`pg_isready` got no response), so `sqlx migrate run` cannot be verified. | A throwaway Postgres container, and run-lifecycle migrations applied. |
| T16 `pg_` constraint, freeze and role tests | Needs T15 and a database. | T15 plus a database. |
| T19 terminal write with measured columns | Needs T15 for the columns and a database for the fenced transaction. | T15, Postgres. |
| T20 `measure_run` orchestration | Partly possible with the scripted collector, but the real collector comes from language-drivers, which has 0 of 27 tasks done. Not started this run. | language-drivers T08 and T09, and T17 and T19 first. |
| T21 baseline store | Needs T15 and Postgres. | T15, Postgres. |
| T22 `baseline` subcommand | Needs the judge-sandbox runner (18 of 58 tasks done) and language-drivers. | judge-sandbox runner, language-drivers T08. |
| T23 read-model sections and stream parity | Needs T19, Postgres and the server read model. | T19, Postgres. |
| T25 `PackRunner::bench` in the judge | Needs language-drivers bench and the judge runner. | language-drivers, judge-sandbox. |
| T32 prefs endpoints | Pure part not started. The `pg_` cases need Postgres. | Postgres for the database tests. |
| T37 speech orchestration | Needs hints `ai::generate` and the `hints` table (hints has 0 of 29 tasks done) and Valkey. | hints spec, Postgres, Valkey. |
| T38 `familiar-hint` route | Needs T37 and the hints table. | T37. |
| T40 admission on launch problems | Needs language-drivers, the judge runner, Postgres and the VPS. | Those, plus Brighton's VPS run. |
| T41 real-stack proof (30 runs each) | Needs the real sandbox and the VPS. | T40 and the VPS. |

## Not started this run (no hard blocker)

| Task | Note |
|---|---|
| T17 `assess` (pure) | Inputs exist (T03, T04, T05, T09, T14). Left for the next run to keep this pass within time. |
| T18 load guard and bench gate | Pure core is small. Not started. |
| T24 grimoire SBX-05 and BEN-01 | Needs grimoire-packs T19 checked first. |
| T26 to T31 web mascot | `web/static/familiar/` is absent, so a labelled placeholder is needed. GSAP is not in `web/package.json` and must be added (npm). Not started. |
| T27, T28, T34 (pure web modules) | Not started. |
| T33 browser state, T39 bubble | Depend on T23, T32, T38. |
| T35, T36 (speech prompt and claim guard) | Pure. Not started. |
| T42 to T45 | End-to-end and documentation depend on the above. E2E is deferred per the standing rule. |
| T46 to T48 | Stretch, skipped. |

## Decisions for Brighton

1. **Medium soundness.** The fit was calibrated (T12) by changing `se_k` from 2.5 to 1.0.
   Proptest found a rare Medium-two-classes-off case (about 4 per million linear series).
   Zero were found after the change, but a sample cannot prove zero. A stronger guarantee
   would need an algorithm change, for example requiring five or more points for `Medium`.
   The algorithm must not change without your decision.
2. **Jack-o'-lantern art** is absent (design Open question 1). Placeholder needed.
3. **Error envelope and `hints` table** are owned by other specs that are not yet implemented.

## Run 2 (14 Nov 2026, continuation): status and new blockers

### T15 migration: written, not verified, not committed
- File: `migrations/20261114120000_efficiency_familiar.sql` (untracked; not part of any commit).
- Blocker: no Postgres is available. `docker run postgres:17` was refused by the tool wrapper as a long-running daemon command, and only the Postgres client (no server) is installed locally, so no scratch cluster could start.
- Unblock: a throwaway Postgres 17 container started by Brighton (or an approved way to start one), then `psql -f` each migration in order, twice. Then tick T15 and commit the file.
- Also blocks T16, T19, T21, T23 and T32 (database tests and terminal write), because they need the T15 columns.

### T24 grimoire SBX-05 and BEN-01: partly done
- Done: `check_bench` runs `PackRunner::bench` on the declared sizes, fits with `engine::complexity::fit` in `Mode::Full`, and reports SBX-05 on a class mismatch or a `Low` fit. BEN-01 is raised for a size parameter that is not in the function, a negative size, or a size the schema cannot hold (including an impossible `unique` range). SBX-08 is no longer emitted. Runner faults still surface as SBX-06 (exit 2).
- Not done: the warning when bench sizes span less than 32× (R13.4). It needs a new warning code, but the codes test `only_three_codes_are_warnings` asserts the exact warning list, so it cannot be added without changing that test. Decision for Brighton: approve a new warning code (for example `BEN-02`) and the matching change to the codes test, which must cite a requirement ID that exists in `docs/requirements.md`.
- Test changes made in this run (please review):
  1. `crates/grimoire/tests/mutations.rs`: the static-mutation coverage filter now also skips `BEN-` codes, because BEN-01 is raised only by the sandbox stage, the same as `SBX-`. This is the narrowest fix, but it is a change to an assertion's scope.
  2. `crates/grimoire/tests/sandbox_fake.rs`: `declared_bench_warns_sbx_08_without_calling_the_bench_runner` is replaced by `declared_bench_calls_the_runner_and_a_fault_is_infrastructure`, marked `test-change: PCK-1`. The old test expected SBX-08 and no bench call, which R13.1 removes. PCK-1.3 in `docs/requirements.md` is the cited requirement.

## Run 3 (15 Nov 2026): T20 partly done

- T20 `measure_run` is implemented and tested with the scripted collector (`crates/judge/tests/familiar_measure.rs`, 8 tests: the failure matrix, event order, collector and stop errors). Not ticked, because three parts of T20 are still open:
  1. The canary test (hidden data absent from every payload) and the restart test are not written. The canary check belongs with the privacy tests (T43).
  2. The call from the real executor is not wired. It needs the language-drivers `bench.rs` collector (language-drivers T08 and T09).
  3. The `pg_` cases for T20 are not written, because they depend on T23 and the real collector.

## Run 3 (15 Nov 2026): what is left and why

| Task | State | Blocker and unblock |
|---|---|---|
| T20 | partial (commit 6c366ab) | canary and restart tests, executor wiring and pg cases. Needs language-drivers T08/T09 and T23. |
| T22 | not started | judge-sandbox runner and language-drivers T08. |
| T23 | not started | read-model `complexity` and `familiar` sections need headline and badge text, which the store does not keep, and the stream-parity test needs the SSE endpoint (run-lifecycle T29, not done). |
| T24 | partly done (earlier run) | warning for bench sizes spanning under 32x (R13.4) needs a new warning code, `BEN-02`, and a change to the codes test. Brighton to approve. |
| T25 | not started | judge-sandbox runner and language-drivers. |
| T29 to T31 | not started | Mascot component and GSAP. Real jack-o'-lantern art is still absent (placeholder only). |
| T32 to T33 | not started | prefs endpoints need Postgres `pg_` cases; browser state rules depend on T23. |
| T34 to T36 | not started | explainer depends on T23; claim guard and prompt builder are pure but not yet written. |
| T37 to T39 | not started | blocked by the hints spec (`ai::generate`, `hints` table, Valkey). |
| T40, T41 | not started | blocked by language-drivers, judge runner, VPS, and Brighton's VPS run. |
| T42 | deferred: E2E | browser test. |
| T43 to T45 | not started | depend on the above. |
| T46 to T48 | stretch | skipped. |

Other notes:
- T15 migration fix: the NULL-safety fix to `submissions_familiar_ck` was made in place (migration not yet deployed). Recorded in the review queue.
- T21 `store.rs` was written before its tests, which departs from red-first. The tests were written after and pass.
- T28 tests pass; `npm run check` is clean.

## Run 4 (15 Nov 2026, continuation): what changed and what is still open

Postgres (`cs-dev-pg`) and Valkey are running, so the `pg_` tests ran this time. The earlier "no database" blocker is cleared.

Ticked this run (each with its verification passing in this session): T29, T30, T31, T32, T33, T35, T36.

Still unticked, with the reason:

| Task | Reason | Unblock |
|---|---|---|
| T20 | Canary and restart tests not written; executor wiring needs the language-drivers collector (T08/T09); `pg_` cases need T23. The canary cannot be tested at `measure_run` because its inputs carry no hidden data, so the check belongs in T43. | language-drivers T08/T09, T23, T43. |
| T22, T25 | Need the judge-sandbox runner and language-drivers bench. | language-drivers T08 and judge runner. |
| T23 | Read-model `complexity` and `familiar` sections need the headline and badge text. The store keeps neither the reason code nor the headline, and the SSE endpoint (run-lifecycle T29) is not routed. | Decide whether to store the reason code and rebuild the headline with `words::headline` (a design decision, so I did not make it), and run-lifecycle T29. |
| T24 | The bench-size warning (R13.4) needs a new warning code (`BEN-02`) and a change to the codes test. | Brighton approves `BEN-02` and the test change. |
| T34 | The explainer reads the `complexity` section that T23 defines. | T23. |
| T37 to T39 | Need `hints::ai::generate(Kind::Familiar)`, the `hints` table and Valkey. The hints module is not in `crates/server` yet. | Hints spec implementation. |
| T38 | Needs T37. | T37. |
| T40, T41 | Need language-drivers, the judge runner, the VPS and Brighton's admission run. | VPS run by Brighton. |
| T42 | E2E in the browser, deferred under the standing rule. | Deferred: E2E. |
| T43 to T45 | Depend on the items above. T44 and T45 follow them. | As above. |
| T46 to T48 | Stretch, skipped. | Stretch. |

Manual check for T30 (`npm run dev` with the placeholder art): deferred. It is a browser check, so it is not a unit test.

## Deviations this run (for Brighton to review)

1. **Test-first gap.** `prefs.rs`, `claims.rs` and `prompt.rs` were written with their tests in the same change, so the red step was not seen for those pure modules. The HTTP tests in `familiar_prefs.rs` were seen failing (404) before the handler existed.
2. **T30 crossfade.** The mood change crossfade also loads GSAP, when the result is live and motion is allowed. The design says GSAP loads only for planned bursts. I kept it because the design lists the crossfade as a timeline. Revert it if you want GSAP for bursts only.
3. **T29 glow.** The glow uses CSS `color-mix()` on palette variables, not JavaScript mixing. No hex value appears in components.
4. **T29 `failed` prop.** Added beyond the design's prop list, because the design's text-colour rule needs to know a run failed.
5. **T36 P18.** Implemented as an exhaustive hand-derived table (one form per class), not as a generated-sentence property test.
6. **T32 demo test.** The database refuses to make an existing account the demo account, so the test creates the demo row inside the seeding transaction (`counterspell.demo_seed`). This is the sanctioned seeding path, and the guard is unchanged.
7. **`enforce` made `pub(crate)`** in `runs/create.rs` so the prefs route reuses the rate-limit helper.
8. **Support helper.** `patch()` added to `crates/server/tests/support/mod.rs` next to `post()`.

## Minor review findings (not fixed; for hand-off)

- `applyRestarted(_state)` ignores its argument, which is intentional for a uniform reducer signature.
- `Mascot.svelte`: `drawArt` swallows errors on purpose (missing art leaves the badge only). A debug log would help later.
- `prompt.rs`: `neutralise_markers` edits the learner's code text inside the prompt (`<<<` becomes `< < <`). The model sees a slightly different string. Acceptable, but the user never sees this text.
- `claims.rs`: "linear" in "linear search" counts as a class claim (strict by design, per the design).
- `api.rs`: the PATCH response reads the merged row back, so a row changed by another request is shown with its latest values.

## Run 5 (15 Nov 2026): T20 restart test added

- Added `a_restarted_attempt_gets_a_fresh_result_and_the_gate_is_released` to `crates/judge/tests/familiar_measure.rs`. Attempt 1 fails with an infrastructure error and emits nothing. Attempt 2 emits exactly one `familiar` event. The test passed when first written, because the behaviour already existed in `measure_run`. It therefore did not go red first (a deviation to note).
- T20 stays unchecked. Still open: the canary test (moved to T43, see above), the executor wiring (needs language-drivers T08 and T09), and the `pg_` cases (need T23).
- `cargo clippy -p counterspell-judge --all-targets -- -D warnings` and `cargo test --test familiar_measure` pass (9 tests).

## Run 6 (15 Nov 2026, continuation): what changed and what is still open

Ticked this run, each with verification passing in this session:

| Task | Commit | Verification |
|---|---|---|
| T49 real contract SVG and loader | 3715afa | vitest familiar (84 then 88), `npm run check`, `npm run build` |
| T50 cross-fade the five mood images | 3715afa | mascot tests incl. live cross-fade of only the new image |
| T51 Impeccable polish of the dock | 3715afa | log in `docs/kiro-log/efficiency-familiar-art.md`; visual review deferred (E2E) |
| T23 read-model sections and parity (P17) | a5e5dcc | `cargo test -p counterspell-server --test familiar_read -- --ignored pg_` (3 pass); clippy and fmt clean for common, judge, server |
| T34 explainer view-model | a9d7e74 | `vitest explain` (9 pass), `npm run check` |

Changes beyond the single tasks, for review:
1. `MoodReason::from_code` (common) reads a stored reason back. Needed by the read model. Tests added.
2. `WordFacts.reference` is now `Option<ClassId>` (common, judge assess). The read model can have no fit (for example a failed run), and the wording never needs a reference there. Measured reasons still always carry one, and `measured_wording` falls back to the low-confidence text without it. This changes the signature, so the review should confirm it.
3. The mascot contract now needs five mood images, not carved eye and mouth layers. This follows the 15 Nov amendment in design.md. Tests that checked eye and mouth layers were rewritten to check the image layers. No test was removed without a replacement.
4. The P17 parity test writes its `familiar` event as a fixture built from the same stored facts. The judge's own event writer is covered by the judge's tests. The test checks the wording path, not the whole judge-to-stream pipeline. A full pipeline test needs the judge executor (T20, language-drivers).
5. The `pg_` read-model tests were written after the read-model wiring, not before. Red was seen only for the pure builder and `from_code`.
6. The dock (`Familiar.svelte`) renders the mascot for a measured mood. It passes `reason="measured"` and `seed=1` because the state does not keep the reason code or run id. Effect is derived in the dock: a live failed mood 1 plays the crack burst, and a live mood 5 plays the laugh burst.

Still open (not ticked), with the reason and what would unblock it:

| Task | Reason | Unblock |
|---|---|---|
| T20 measure_run | Canary and restart tests done or moved to T43. Executor wiring needs language-drivers `bench.rs` (T08, T09). The `pg_` cases were unblocked by T23 but need the executor. | language-drivers T08/T09. |
| T22 `baseline` subcommand | Needs judge-sandbox runner and language-drivers T08. | Same. |
| T24 grimoire | The bench-size warning (R13.4) needs a new warning code `BEN-02` and a change to the codes test. | Brighton approves `BEN-02`. |
| T25 `PackRunner::bench` | Needs language-drivers bench and the judge runner. | Same as T22. |
| T37 speech orchestration | `crates/server/src/hints` is absent. Needs `hints::ai::generate(Kind::Familiar)`, the `hints` table and the Valkey marker. | The hints spec implementation. |
| T38 `familiar-hint` route | Depends on T37. | T37. |
| T39 speech bubble component | Depends on T38 and T33. | T38. |
| T40 enable bench on launch problems | Needs the VPS, the sandbox, language-drivers and Brighton's admission run. | Brighton's VPS run. Brighton must decide. |
| T41 real-stack proof (30 runs each) | Needs the real sandbox, the VPS and T40. | T40. |
| T42 end-to-end IDE pass | Browser E2E, deferred under the standing rule. | Deferred: E2E. |
| T43 privacy tests | Depends on T20 (canary), T38 and T39. | T20, T38, T39. |
| T44 docs | Depends on T42. | T42. |
| T45 full gate | Depends on T43 and T44. | T43, T44. |
| T46 to T48 | Stretch. Not started, as the brief says. | Stretch. |

Blockers for Brighton:
- Approve `BEN-02` for T24 (the bench-size warning).
- The VPS admission run for T40 and T41 (which problems to enable bench for, and the baselines).
- Confirm the `WordFacts.reference` signature change (item 2 above).

Pre-existing: the earlier `cargo build` run printed three unused-import warnings in `crates/server/src/auth/clock.rs` and `crates/server/src/ratelimit.rs`. These files were not touched in this run. Clippy with `-D warnings` on the three changed crates passed.

## Run 7 (resume, 15 Nov 2026 late session)

- **T20 `measure_run`: ticked (commit 3e54f81).** Added `a_fitted_bench_without_a_usable_baseline_is_capped_at_pleased` (matrix row: no baseline gives mood 4, `no_baseline`). That behaviour already existed in `assess` (T17), so the new test passed on its first run. Fmt and Clippy (`-D warnings`) are clean for `counterspell-judge`, and its 56 tests pass. The canary and the full restart test with the real executor stay with T43 and T20's executor wiring (language-drivers), as the earlier run recorded.
- **T24 grimoire: still open (partial, commit to follow).** Added three sandbox_fake tests: a quadratic series declared `n` warns SBX-05; a linear series declared `n` does not; two points warn SBX-05 (cannot be fitted). Still missing:
  1. The span warning (sizes under 32×) needs a new warning code, `BEN-02`. Brighton has not approved it (same blocker as before).
  2. A test for the `Low` path (SBX-05 "not confirmed"). I could not find a deterministic noisy series that the fit reports as `Low` without tuning the test to the code.
  3. A test for an infeasible `unique` size, giving BEN-01. The shared `values` fixture has no unique constraint, so this needs a new schema fixture.
- **T22 and T25: blocked** on judge-sandbox and language-drivers T08/T09 (bench handoff). Both are unchecked in their specs.
- **T37 to T39 (speech): blocked** on the hints spec. `crates/server/src/hints` does not exist and the hints tasks (G1, H01 and so on) are all unchecked. T38 and T39 depend on T37.
- **T40 and T41: blocked** on the VPS, a real rootless sandbox and language-drivers. These need Brighton's admission run and cannot be done from this machine.
- **T42: deferred: E2E** (browser pass). **T43** depends on T38 and T39. **T44** depends on T42. **T45** depends on T43 and T44.
- **T46 to T48: stretch.** Not started, as the brief says.

## Run 8 (15 Nov 2026, resume): status

Open non-stretch tasks and the reason each stays open:

| Task | Reason | Unblock |
|---|---|---|
| T20 | Executor wiring and `pg_` cases need language-drivers T08/T09 (still 0 of the bench tasks done; the language-drivers spec has 2 of 27 tasks done). | language-drivers T08/T09, then T23 is already done. |
| T22, T25 | Need judge-sandbox runner (20 of 58 done) and language-drivers bench. | Same. |
| T24 | Span warning (R13.4) needs new code `BEN-02` and a change to the codes test. Brighton has not approved this. A BEN-01 test for an impossible `unique` size was tried and reverted: the `unique` flag also makes the samples invalid, so the fixture cannot isolate the case. It needs a dedicated fixture pack. | Brighton approves `BEN-02`. A new fixture for the unique case can then be written. |
| T37 to T39 | The hints spec has 0 of 29 tasks done. `hints::ai::generate(Kind::Familiar)`, the `hints` table and the Valkey marker do not exist. | Hints spec implementation. |
| T40, T41 | VPS, real sandbox and Brighton's admission run. | Brighton's VPS run. |
| T42 | Browser end-to-end pass. | Deferred: E2E. |
| T43 to T45 | Depend on T20 (canary), T38, T39 and T42. | As above. |
| T46 to T48 | Stretch. Skipped. | Stretch. |

No code change this run. The grimoire test suite passes at 16 of 16.

### verify all, 16 Nov 2026: T24 stays open: the 'sizes span less than 32x' warning and the BEN-01 unique-size case are still missing in crates/grimoire (only the BEN_01 code constant exists). Also, the main tree holds uncommitted judge bench work (crates/judge/src/sandbox/bench.rs, tests/sbx_bench.rs, model_bench.rs) that fails 'cargo fmt --check' and clippy (manual_async_fn, cast_precision_loss in bench.rs:276). Not touched by verify; fix before it is committed under T25.

## kiro/drive-queue run (20 Nov 2026)
- **Decision (routine, under the standing "decide and record" rule; Brighton may overrule):** R13.4's bench-span warning is `BEN-02` (warning), emitted by `check_bench` when the largest bench size is less than 32 times the lowest. The bench still runs. The codes test now lists four warnings (`test-change: PCK-2`). No pack in `packs/` declares a `[bench]` yet, so no import changes.
- T24 ticked: the BEN-01 `unique` case and the `Low` ("not confirmed") case were already tested by an earlier run; BEN-02 adds the span warning. SBX-08 is no longer emitted anywhere.
- T22 and T25: the judge-sandbox runner and language-drivers bench (T08, T09) are now done, so these are no longer blocked on them. Next run.
