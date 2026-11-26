# The Null Marks

## Task
A ledger row without a value carries the mark `-1` — never a real value, never a match. Call `null_marks(rows)` and return how many rows carry the mark.

## Input
`rows` lists `[key, value]` pairs; either field may hold the `-1` mark, values run to `30`.

## Output
The count of rows whose value is `-1`.

## Examples
- `null_marks([[1, -1], [2, 5], [3, -1]])` returns `2`.
- `null_marks([])` returns `0`.
- `null_marks([[0, -1], [0, -1]])` returns `2`.

## Constraints
- `0 <= len(rows) <= 20` rows of `[key, value]` with `-1 <= key, value <= 30`; the mark is `v == -1`
