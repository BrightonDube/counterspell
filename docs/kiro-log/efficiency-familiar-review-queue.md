# efficiency-familiar: review queue (supervised tasks)

## T12: calibrate and freeze `fit-1`

- Commits: `eca89a4` (first calibration, P1 and P2 passed on their random seeds), then `24902eb` (revised after a P1 counterexample; `se_k` changed from 2.5 to 1.0).
- What to check:
  1. The one constant change, `se_k` 2.5 → 1.0 (`crates/engine/src/complexity/mod.rs`). It makes agreement stricter, so it only moves confidence toward Low.
  2. The calibration record (`docs/kiro-log/efficiency-familiar-calibration.md`), including the attempts that were rejected.
  3. Whether the sampled guarantee (zero Medium two classes off in 1M cases per class) is enough, or whether the algorithm should require five points for `Medium` (see blockers, decision 1).
  4. The regression test `the_spiked_linear_counterexample_is_never_a_wrong_medium_class` and the committed `properties_complexity.proptest-regressions`.
