# efficiency-familiar: fit calibration report (T10 to T12)

Recorded 14 Nov 2026 by the implementation run. Fit version: `fit-1`.

## Summary

- P1 (soundness) first failed on `fit-1` as first written. A linear series with one spiked
  point was reported as Medium quadratic. That is two classes off, which R3.8 forbids.
- One constant was changed: `se_k` from 2.5 to 1.0. That makes the agreement check stricter.
  It moves confidence toward less certainty, as T12 allows. The algorithm is unchanged.
- After the change, 1,000,000 seeded cases per true class gave **zero Medium results two or
  more classes off** and **zero High results with the wrong class**. P1 and P2 pass.
- Zero in a sample is not a proof. P1 is a sampled property, and the report says so.

## Frozen constants (`fit-1`)

These are the values in `FitConfig::default()` (`crates/engine/src/complexity/mod.rs`):

| Constant | Value | Note |
|---|---|---|
| noise_floor / noise_cap | 0.03 / 1.0 | |
| spread_factor | 0.25 | |
| default_noise | 0.10 | |
| delta_floor / delta_k | 0.025 / 0.75 | unchanged |
| fit_ok_floor / fit_ok_k | 0.05 / 1.8 | unchanged |
| slope_tol / exp_tol | 0.30 / 0.35 | unchanged |
| se_k | **1.0** (was 2.5 in the first draft) | the only change |
| min_range_medium / min_range_high | 8 / 32 | |
| min_points_high | 5 | |
| high_noise_max | 0.12 | |
| high_margin_k | 2.0 | |

## What went wrong, and the record of each attempt

1. **Calibration counter bug (fixed).** The first calibration table did not count Medium
   results that were two or more classes off. Those rows fell into an empty arm. It now has a
   column for them.
2. **P1 counterexample (proptest, random seed).** Truth linear, four sizes 64 to 512, no
   overhead, no noise, one spike of about 2.2× with spread unknown. The fit gave Medium
   quadratic. The residuals were: linear 0.36, quadratic 0.11, with σ̄ = 0.10 and a range of
   exactly 8×. The agreement check passed because its tolerance was `se_k × SE` with SE ≈ 0.33,
   so the tolerance was about 0.82 and the slope gap was 0.43.
3. **Mutation check.** Loosening the constants on purpose made P1 fail. That showed the
   property can catch an over-confident fit. The source was restored afterwards.

Attempts, all measured with the calibration test and the property tests. Each row changes
one thing from `fit-1` as first written, unless noted.

| # | Change | Calibration (wrong Medium, linear) | P1 / P2 | Kept? |
|---|---|---|---|---|
| a | `delta_k` 0.75 → 1.5 and `fit_ok_k` 1.8 → 1.0 (together), 200k | 0 | P2 fails (clean linear with overhead, linearithmic 0.044 inside the wider slack) | no |
| b | `delta_k` 0.75 → 1.5 alone, 200k | 0 | P2 fails (same cause) | no |
| c | `fit_ok_k` 1.8 → 1.0 alone, 200k | 3 | pass | no (not zero) |
| d | `fit_ok_k` 1.8 → 0.5, 1M | 4 | pass | no (not zero) |
| e | `fit_ok_k` 1.8 → 0.3, 1M | 3 | pass | no (not zero) |
| f | **`se_k` 2.5 → 1.0**, 1M | **0** | pass | **yes, frozen** |

## Final calibration table (`fit-1`)

`cargo test -p counterspell-engine --test properties_complexity -- --ignored --nocapture calibration`,
with `CALIBRATION_CASES=1000000`, release build, 1,000,000 seeded series per class.

| true class | cases | Low | Medium correct | Medium adjacent | Medium wrong (2+ classes) | High correct | wrong at High |
|---|---|---|---|---|---|---|---|
| Linear | 1000000 | 77.6% | 16.9% | 0.1% | 0 | 5.4% | 0 |
| Linearithmic | 1000000 | 68.2% | 21.1% | 0.1% | 0 | 10.6% | 0 |
| Quadratic | 1000000 | 17.0% | 45.0% | 0.0% | 0 | 38.1% | 0 |
| Cubic | 1000000 | 14.5% | 47.5% | 0.0% | 0 | 38.0% | 0 |

Percentages are rounded to 0.1%. The zero counts are zero in this sample only.

Reading the table:

- Noisy linear and `n log n` data are mostly `Low`. Those two classes are hard to separate
  over a short range under noise. The explainer must say "could be linear or n log n".
- Quadratic and cubic data are called `Medium` or `High` for most cases.
- Medium results two or more classes off: none in the 1M sample per class.

## Property runs after the change

- P1 and P2 passed on three separate runs with fresh random seeds (`cargo test -p counterspell-engine --test properties_complexity`).
- The regression test `the_spiked_linear_counterexample_is_never_a_wrong_medium_class` pins the
  P1 counterexample, and passes.
- `properties_complexity.proptest-regressions` holds the failing seed from the first draft.

## Open risk for Brighton

The guarantee in R3.8 ("for medium, not off by more than one class") rests on sampling. The
sample found no violation after the change, but a rare one could exist. A stronger guarantee
would need an algorithm change, such as requiring five or more points for `Medium`. T12
forbids algorithm changes without a decision from Brighton, so this is reported, not made.

## Decision

`FIT_VERSION` stays `fit-1`. The fit has not been stored or shown to any learner yet, so no
result needs migrating. Any later constant or algorithm change must bump it and rerun this
table.
