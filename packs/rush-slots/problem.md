# The Rush Slots

## Task
Wax seals bid for marks they cover; two picks may not share a mark. Call `pick_count(spans)` and return the most seals that can be picked without sharing.

## Input
`spans` lists the `[a, b]` bids, `0 <= a <= b <= 100`.

## Output
The largest count of mark-disjoint seals.

## Examples
- `pick_count([[0, 1], [1, 2], [2, 3]])` returns `2` — `1` is shared, so the middle loses.
- `pick_count([[0, 10], [1, 2], [3, 4], [5, 6], [7, 8], [9, 10]])` returns `5`.
- `pick_count([[0, 2], [1, 3]])` returns `1`.

## Constraints
- `0 <= len(spans) <= 20` rows of `[a, b]` with `0 <= a <= b <= 100`
