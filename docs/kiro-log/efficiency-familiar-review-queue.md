# efficiency-familiar: review queue (supervised tasks)

## T12: calibrate and freeze `fit-1`

- Commits: `eca89a4` (first calibration, P1 and P2 passed on their random seeds), then `24902eb` (revised after a P1 counterexample; `se_k` changed from 2.5 to 1.0).
- What to check:
  1. The one constant change, `se_k` 2.5 → 1.0 (`crates/engine/src/complexity/mod.rs`). It makes agreement stricter, so it only moves confidence toward Low.
  2. The calibration record (`docs/kiro-log/efficiency-familiar-calibration.md`), including the attempts that were rejected.
  3. Whether the sampled guarantee (zero Medium two classes off in 1M cases per class) is enough, or whether the algorithm should require five points for `Medium` (see blockers, decision 1).
  4. The regression test `the_spiked_linear_counterexample_is_never_a_wrong_medium_class` and the committed `properties_complexity.proptest-regressions`.

## T15: migration `20261115130000_efficiency_familiar.sql` (commit 649d3b4)

- Verified on a scratch database in the local Postgres 17 container: the full migration chain applied in order, then this file a second time with no error. The scratch database was dropped.
- Privilege check: the server role cannot write `familiar_mood`; the judge role can; the judge role cannot `DELETE` from `submissions`; the judge role can `INSERT` into `run_timings`.
- What to check:
  1. The server role's table-level `INSERT` on `submissions` was replaced by a column list that leaves out the five familiar columns (`REVOKE INSERT` then `GRANT INSERT (...)`). Confirm the server's insert statement uses only the listed columns.
  2. The `submissions_familiar_ck` constraint and the `cs_familiar_frozen` trigger.
  3. Found by T16 after the commit: `submissions_familiar_ck` let a class with NULL confidence through, because SQL treats a NULL CHECK result as a pass. The `measured_complexity` clause and the mood-verdict clause are now NULL-safe. The migration is not yet deployed, so it was edited in place.

## T19: measured columns in the terminal write (commit pending, see git log)

- `finish` now sets the five familiar columns in the same `UPDATE` that sets `finished_at`. A completed run with no measurement stores none. A stale owner writes none. A mood on a wrong answer fails the database rule, and the whole transaction rolls back.
- Tests: `crates/judge/tests/finish.rs`, four new `pg_familiar_*` cases. All 11 finish tests pass.
- What to check:
  1. `FinalResult.measured` is `Option<Box<MeasuredColumns>>`. The box was added because clippy's `large_enum_variant` fired on `RunOutcome::Completed`.
  2. `finish_failed` does not touch the five columns, so failed, cancelled and expired runs store none.

## T22 (kiro/drive-queue run): `counterspell-judge baseline`
- What changed: `parse_args`, `fixture_inputs`, `FixtureCollector` (sandbox feature), `execute` and `fixture_source` in `crates/judge/src/familiar/baseline_cmd.rs`; `published_version` in `familiar/store.rs` (SQL stays in `store.rs`); the `baseline` dispatch in `main.rs`.
- Check: execution goes only through `oracle::side` + `bench_side` on the verified rootless daemon (`daemon::verify` runs first, as in `preflight`); nothing runs on the host. The fixture runs with `Role::Reference`, so a fixture crash is treated as infrastructure-level for the bench, which then refuses.
- Check: baselines use the same inputs as learner benches (`bench_seed(slug, version, n)`), and the digest key is `lang.image().as_str()`, the same string the executor's `find_baseline` uses.
- Decision for review: without `--fixture` the stored Python reference is the Python fixture; JavaScript (and any other language) needs `--fixture <path>`, because only Python references are stored. The fixture path is an operator-supplied file (trusted operator input), but its contents still run only in the sandbox.
- `sbx_baseline` stores a quadratic fixture declared quadratic and refuses it declared linear. Linear vs n log n was not used for the stored path: on this host it came back `Low` (plausible [Linear, Linearithmic]) at every span tried, which matches the calibration report. Sizes 256..8192 were needed; 64..2048 and 128..4096 gave `Low` in 1/8 and 3/15 runs.

## T43 (kiro/drive-queue run): privacy across surfaces
- Judge: `crates/judge/tests/familiar_privacy.rs` runs an accepted benched Submit through the scripted backend with canaries in a hidden test's input and expected output and in stderr. The run is Accepted only because the expected canary was matched, so the canary really crossed the comparator. None appears in `bench_*`, `complexity` or `familiar` events, in the measured columns (including `fit`) or in the logs.
- Server: `crates/server/tests/familiar_privacy.rs` (`pg_`) checks the speech prompt, the bubble and the stored hint against five canaries (hidden input, hidden expected, reference, known ghost, statement). Control: the prompt does contain the learner's code. `ScriptedTransport` now records prompts (`prompts()`, testkit only). The judge catalogue fixture gained `CANARY_STATEMENT` in place of the literal "Statement"; no test depended on that text and 385 `pg_` tests pass.
- Web: `web/src/lib/familiar/privacy.test.ts` scans every Familiar source for HTML sinks. Mutation-checked: `{@html}` and an HTML-typed `DOMParser` both fail it. `Mascot.svelte` parses its own validated SVG art with `DOMParser` as `image/svg+xml`; the test allows that and nothing else.
- Server-role write grants (P16) are already covered by `pg_familiar_roles`; not duplicated.
- Not covered by an automated canary: the read-model JSON of `GET /api/runs/{id}` is built from stored columns only (T23), and its columns are covered by the judge test above.
