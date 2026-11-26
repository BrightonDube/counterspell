# The Quantum Tick

## Task
The engine grants each rite a slice of at most `q` ticks, then sends it to the back of the queue if it still has ticks to spend. Rites that arrive during a slice join the queue before the spender returns. Call `quantum_tick(jobs, q)` and return the sum of finishing ticks across all rites.

## Input
`jobs` lists `[arrival, burst]` rows (`0` to `60`); `q` is the slice (`1` to `8`). Ties in arrival keep their listed order.

## Output
The sum of each rite's finishing tick.

## Examples
- `quantum_tick([[0, 5], [1, 4], [2, 3]], 2)` returns `33` — finishes `10, 11, 12`.
- `quantum_tick([], 2)` returns `0`.
- `quantum_tick([[0, 1]], 2)` returns `1`.

## Constraints
- `0 <= len(jobs) <= 12` rows of `[arrival, burst]` with `0 <= arrival, burst <= 60`, `1 <= q <= 8`
