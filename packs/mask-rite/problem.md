# The Mask Rite

## Task
Twenty candles at most — each burns or stays dark, and a lighting is a bitmask. Call `mask_rite(weights, cap)` and return how many lightings keep their total flame at or under `cap`.

## Input
`weights` lists candle flames `0..30`; `cap` runs `0` to `60`.

## Output
The count of subsets of `weights` summing to at most `cap`. The all-dark lighting always counts.

## Examples
- `mask_rite([1, 2, 3], 3)` returns `5` — `{}, {1}, {2}, {3}, {1,2}`.
- `mask_rite([], 0)` returns `1`.
- `mask_rite([5], 3)` returns `1` — only the dark lighting fits.

## Constraints
- `0 <= len(weights) <= 20`, each `0..30`; `0 <= cap <= 60`
