# The Join Span

## Task
Two ledgers join on shared keys, but only rising pairs count — a row of `a` pairs with a row of `b` only when `a`'s value is strictly below `b`'s. Call `join_span(a, b)` and return the count of rising pairs.

## Input
`a` and `b` list `[key, value]` rows; fields run `0` to `30`.

## Output
The count of `(arow, brow)` pairs with `arow.key == brow.key` and `arow.value < brow.value`.

## Examples
- `join_span([[1, 10], [1, 20]], [[1, 15], [1, 25]])` returns `3` — `20 < 15` fails, the rest rise.
- `join_span([[2, 5]], [[2, 4]])` returns `0`.
- `join_span([[1, 1]], [[1, 2]])` returns `1`.

## Constraints
- `0 <= len(a), len(b) <= 20` rows of `[key, value]` with `0 <= key, value <= 30`
