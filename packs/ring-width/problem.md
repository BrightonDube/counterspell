# The Ring Width

## Task
Call `widest_ring(vals, left, right)` and return the breadth of the widest ring of the stored tree — the count of nodes on its fullest level.

## Input
`vals` holds each node's mark, `left` and `right` hold each node's child indexes; `-1` means no child there. The root is index `0`. Cells the walk never reaches do not belong to the tree, and a child index that does not point at a real cell counts as no child. The three lists have the same length.

## Output
The node count on the fullest level; `0` for an empty tree, `1` for a lone root.

## Examples
- `widest_ring([5, 3, 7, 2, 4, 6, 8], [1, 3, 5, -1, -1, -1, -1], [2, 4, 6, -1, -1, -1, -1])` returns `4`.
- `widest_ring([1, 2, 3, 4], [1, 2, 3, -1], [-1, -1, -1, -1])` returns `1` — a skewed chain stays one node wide.
- `widest_ring([], [], [])` returns `0`.

## Constraints
- `0 <= len(vals) = len(left) = len(right) <= 40`
- every `left[i]` and `right[i]` is `-1` or an index into `vals`
- the walk from `0` never loops back
