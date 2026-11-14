# efficiency-familiar: fit calibration report (T10 to T12)

Recorded 14 Nov 2026 by the implementation run. Fit version: `fit-1`.

## Constants (frozen as `fit-1`)

No constant was changed during calibration. The values are those written in
`FitConfig::default()` (`crates/engine/src/complexity/mod.rs`) when the algorithm was
implemented in T09:

| Constant | Value |
|---|---|
| noise_floor / noise_cap | 0.03 / 1.0 |
| spread_factor | 0.25 |
| default_noise | 0.10 |
| delta_floor / delta_k | 0.025 / 0.75 |
| fit_ok_floor / fit_ok_k | 0.05 / 1.8 |
| slope_tol / exp_tol / se_k | 0.30 / 0.35 / 2.5 |
| min_range_medium / min_range_high | 8 / 32 |
| min_points_high | 5 |
| high_noise_max | 0.12 |
| high_margin_k | 2.0 |

Calibration could only move constants toward less confidence, and it was needed only if P1
or P2 failed. Both passed with these values, so T12 changed nothing.

## Corpus parameters (`crates/engine/tests/support/complexity.rs`)

- True classes: `n`, `n log n`, `n^2`, `n^3`.
- Points `k`: 3 to 8. Size factor: 2 or 4. Smallest size `n_0`: 2^6 to 2^10.
- Fixed overhead: 0 to 3 times the compute at `n_0` (noise-free property P2: 0 to 1).
- Lognormal noise per repeat: sd 0 to 0.25. Spike probability 0 to 0.1, multiplier 1.5 to 3.
- Five repeats per size; the median is kept and the spread is `(max − min) / median`.
- Half the series carry their measured spread, half have none (noise then defaults).
- Seeded by splitmix64, so every case replays.

## T11: property run before calibration (`cargo test -p counterspell-engine --test properties_complexity`)

| Property | Cases | Result |
|---|---|---|
| P1 soundness (High is true, Medium is true or adjacent) | 4000 | pass |
| P2 usefulness (noise-free, 32× span, at least Medium, true class) | 4000 | pass |
| P3 scale invariance (powers of two) | 256 | pass |
| P4 totality on arbitrary points, including NaN and infinite spreads | 256 | pass |
| P5 determinism | 256 | pass |
| P6 class only with Medium or High, leaning is plausible | 256 | pass |
| P7 Quick never High | 256 | pass |

A sanity check: with the plausibility slack and the goodness ceiling loosened on purpose
(`delta_floor = 0`, `delta_k = 0`, `fit_ok_k = 100`, `min_range_medium = 1`), P1 fails. So the
property does catch an over-confident fit. The source was restored afterwards and the
mutant's proptest regression file was deleted, because it did not describe a real failure.

## T12: calibration table (`cargo test -p counterspell-engine --test properties_complexity -- --ignored --nocapture calibration`)

4000 seeded series per true class, `fit-1` constants.

| true class | cases | Low | Medium correct | Medium adjacent | High correct | wrong at High |
|---|---|---|---|---|---|---|
| Linear | 4000 | 77.8% | 17.0% | 0.1% | 5.2% | 0 |
| Linearithmic | 4000 | 67.9% | 21.3% | 0.1% | 10.7% | 0 |
| Quadratic | 4000 | 11.9% | 47.7% | 0.0% | 40.4% | 0 |
| Cubic | 4000 | 9.6% | 51.6% | 0.0% | 38.8% | 0 |

**Wrong-at-High count: 0 for every class.** This meets the requirement.

Reading the table honestly:

- Noisy linear and `n log n` series are mostly `Low`. A linear curve with noise and spikes is
  hard to tell from `n log n` over a short range. This is the blind spot the explainer must
  state ("could be linear or n log n").
- Quadratic and cubic series with the same noise are called `Medium` or `High` most of the time.
- Medium-adjacent results are rare (0.1% or less). The table does not show why.

## Decision

`FIT_VERSION` stays `fit-1`. Any future change to a constant or to the algorithm must bump it
and rerun this table.
