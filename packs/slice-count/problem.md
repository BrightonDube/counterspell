# The Slice Count

## Task
The engine grants each rite slices of at most `q` ticks; a rite of `b` ticks needs `ceil(b / q)` slices. Call `slice_count(bursts, q)` and return the total number of slices the roster needs.

## Input
`bursts` lists each rite's ticks (`0` to `60`); `q` is the slice size (`1` to `12`).

## Output
The total slice count across all rites.

## Examples
- `slice_count([5, 3, 0], 2)` returns `5` — `3 + 2 + 0` slices.
- `slice_count([], 3)` returns `0`.
- `slice_count([4, 4], 4)` returns `2`.

## Constraints
- `0 <= len(bursts) <= 16` with `0 <= burst <= 60`, `1 <= q <= 12`
