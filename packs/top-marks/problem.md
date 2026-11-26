# The Top Marks

## Task
A scribe needs only the strongest `k` marks from a tally strip. Call `top_marks(scores, k)` and return the sum of the `k` highest marks — all of them when fewer than `k` exist.

## Input
`scores` is a list of marks and `k` the quota.

## Output
The sum of the `k` largest marks, or of every mark when `len(scores) < k`.

## Examples
- `top_marks([3, 9, 1, 7], 2)` returns `16` — `9 + 7`.
- `top_marks([5, 5, 5], 4)` returns `15`.
- `top_marks([], 3)` returns `0`.

## Constraints
- `0 <= len(scores) <= 24`, `-100 <= scores[i] <= 100`, `0 <= k <= 30`
