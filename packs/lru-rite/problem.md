# The Lru Rite

## Task
The folio holds `f` pages by freshness — a miss swaps out the page used least recently, a touch keeps it warm. Call `lru_rite(pages, f)` and return how many arrivals find their page missing.

## Input
`pages` lists arrivals (`0` to `30`); `f` is the folio's size (`1` to `6`).

## Output
The least-recently-used fault count.

## Examples
- `lru_rite([0, 1, 2, 0], 2)` returns `4` — `2` evicts `0`, so `0`'s return faults.
- `lru_rite([0, 1, 0, 1], 2)` returns `2`.
- `lru_rite([], 3)` returns `0`.

## Constraints
- `0 <= len(pages) <= 24` with `0 <= page <= 30`, `1 <= f <= 6`
