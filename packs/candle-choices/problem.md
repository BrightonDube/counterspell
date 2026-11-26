# The Candle Choices

## Task
The altar needs exactly `k` of `n` distinct candles. Call `pick_ways(n, k)` and return how many choices there are — order does not matter.

## Input
`n` distinct candles, `k` picks. When `k` is zero there is exactly one choice — none. When `k` exceeds `n` there is none.

## Output
The number of ways to pick `k` candles from `n`, as an integer.

## Examples
- `pick_ways(4, 2)` returns `6`.
- `pick_ways(3, 0)` returns `1`.
- `pick_ways(2, 5)` returns `0`.

## Constraints
- `0 <= n <= 20`, `0 <= k <= 20`
