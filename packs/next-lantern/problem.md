# The Next Taller Lantern

## Task
Lanterns hang in a row at marked heights. For each lantern the warden wants the distance to the nearest lantern further along that stands strictly taller — `0` when none stands taller ahead. Call `next_taller(heights)` and return those distances in order.

## Input
A list of lantern heights, each at most `99`.

## Output
Return a list where entry `i` is the distance from lantern `i` to the nearest strictly taller lantern after it, or `0`.

## Examples
- `next_taller([2, 4, 1, 3])` returns `[1, 0, 1, 0]`. The `2` sees the `4` one step on; the `4` sees nothing; the `1` sees the `3` one step on; the `3` sees nothing.
- `next_taller([3, 1, 2])` returns `[0, 1, 0]`. The `3` towers over both.
- `next_taller([])` returns `[]`. An empty row has nothing to report.

## Constraints
The row holds at most 40 lanterns.
