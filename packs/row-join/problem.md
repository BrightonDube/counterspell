# The Row Join

## Task
Two ledgers meet on matching keys — every row of `a` pairs with every row of `b` that shares its key, and each pairing is a joined row. Call `row_join(a, b)` and return how many joined rows the pairing makes.

## Input
`a` and `b` list `[key, value]` rows; keys and values run `0` to `30`.

## Output
The count of `(arow, brow)` pairs with equal keys.

## Examples
- `row_join([[1, 10], [2, 20], [1, 30]], [[1, 5], [3, 7], [1, 9]])` returns `4` — key `1` joins twice per ledger, key `3` finds nothing on the left.
- `row_join([], [[1, 2]])` returns `0`.
- `row_join([[1, 1]], [[1, 2], [1, 3]])` returns `2`.

## Constraints
- `0 <= len(a), len(b) <= 20` rows of `[key, value]` with `0 <= key, value <= 30`
