# The Null Join

## Task
Two ledgers join on matching keys — but the mark `-1` is no key at all, and marked rows never join, not even with each other. Call `null_join(a, b)` and return the count of joined pairs.

## Input
`a` and `b` list `[key, value]` rows; fields run `-1` to `30`.

## Output
The count of pairs with equal keys where the key is not `-1`.

## Examples
- `null_join([[1, 5], [-1, 9]], [[-1, 3], [1, 7]])` returns `1` — the marked rows never meet.
- `null_join([[-1, 1]], [[-1, 2]])` returns `0` — marks do not match marks.
- `null_join([[1, 1], [1, 2]], [[1, 3], [1, 4]])` returns `4`.

## Constraints
- `0 <= len(a), len(b) <= 20` rows of `[key, value]` with `-1 <= key, value <= 30`
