# The Page Turn

## Task
The folio holds `f` pages in oldest-first order — a miss swaps the oldest out. Call `page_turn(pages, f)` and return how many arrivals find their page missing.

## Input
`pages` lists arrivals (`0` to `30`); `f` is the folio's size (`1` to `6`).

## Output
The first-in-first-out fault count.

## Examples
- `page_turn([0, 1, 2, 0, 3, 1], 2)` returns `6` — every arrival faults.
- `page_turn([0, 1, 0, 1], 2)` returns `2`.
- `page_turn([], 3)` returns `0`.

## Constraints
- `0 <= len(pages) <= 24` with `0 <= page <= 30`, `1 <= f <= 6`
