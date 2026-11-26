# The Rank Gain

## Task
A ranked answer list pays out by position — the `i`-th entry earns `rels[i] * (k - i)` favour, and the rite counts only the first `k` entries. Call `rank_gain(rels, k)` and return the total favour.

## Input
`rels` is the relevance strip, `k` the rank depth.

## Output
The sum of `rels[i] * (k - i)` for `0 <= i < min(k, len(rels))`.

## Examples
- `rank_gain([5, 3, 1], 3)` returns `22` — `5*3 + 3*2 + 1*1`.
- `rank_gain([4, 4], 4)` returns `28` — `4*4 + 4*3`.
- `rank_gain([], 2)` returns `0`.

## Constraints
- `0 <= len(rels) <= 24`, `0 <= rels[i] <= 9`, `0 <= k <= 30`
