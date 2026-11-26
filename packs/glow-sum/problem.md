# The Glow Sum

## Task
Call `tree_sum(vals, left, right)` and return the total of every mark the walk from the root can reach.

## Input
`vals` holds each node's mark, `left` and `right` hold each node's child indexes; `-1` means no child there. The root is index `0`. Cells the walk never reaches do not belong to the tree, and a child index that does not point at a real cell counts as no child. The three lists have the same length.

## Output
The sum of reachable `vals`; an empty tree sums to `0`.

## Examples
- `tree_sum([5, 3, 7], [1, -1, -1], [2, -1, -1])` returns `15`.
- `tree_sum([], [], [])` returns `0`.
- `tree_sum([4, 2, 6, 100], [1, -1, -1, -1], [2, -1, -1, -1])` returns `12` — the stray `100` never joins the tree.

## Constraints
- `0 <= len(vals) = len(left) = len(right) <= 40`
- every `left[i]` and `right[i]` is `-1` or an index into `vals`
- the walk from `0` never loops back
