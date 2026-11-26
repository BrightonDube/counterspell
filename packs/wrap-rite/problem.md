# The Wrap Rite

## Task
The ring holds `b` marks — a new mark evicts the oldest when the ring is full, while a mark already inside is a no-op. Call `wrap_rite(marks, b)` and return how many arrivals force an eviction.

## Input
`marks` lists writes in order (`0` to `30`); `b` is the ring's size (`1` to `8`).

## Output
The overwrite count — misses on a full ring.

## Examples
- `wrap_rite([0, 1, 2, 1, 3], 2)` returns `2` — `2` evicts `0`, `3` evicts `1`.
- `wrap_rite([0, 1, 0, 1], 2)` returns `0`.
- `wrap_rite([], 4)` returns `0`.

## Constraints
- `0 <= len(marks) <= 24` with `0 <= mark <= 30`, `1 <= b <= 8`
