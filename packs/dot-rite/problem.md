# The Dot Rite

## Task
Two sparse tally scrolls list `[position, mark]` pairs — a dot rite multiplies marks that share a position and sums the products. Call `dot_sparse(a, b)` and return that shared-position product total.

## Input
`a` and `b` are lists of `[position, mark]` rows. When a position repeats in one scroll its last mark stands.

## Output
The sum of `mark_a * mark_b` over positions present in both scrolls.

## Examples
- `dot_sparse([[0, 3], [2, 5]], [[0, 4], [1, 7], [2, 1]])` returns `17` — `3*4 + 5*1`.
- `dot_sparse([], [[0, 1]])` returns `0`.
- `dot_sparse([[3, 2]], [[0, 9]])` returns `0` — no shared position.

## Constraints
- `0 <= len(a), len(b) <= 20` rows each of `[position, mark]` with `-40 <= position, mark <= 40`
