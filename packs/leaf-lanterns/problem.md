# The Leaf Lanterns

## Task
A grove of ember trees is stored as index arrays. Call `count_leaves(vals, left, right)` and return how many lit leaves the tree has — a leaf is a node with no left and no right child.

## Input
`vals` holds each node's mark, `left` and `right` hold each node's child indexes; `-1` means no child there. The root is index `0`. Cells the walk never reaches do not belong to the tree, and a child index that does not point at a real cell counts as no child. The three lists have the same length.

## Output
The number of leaf nodes the walk from the root can reach.

## Examples
- `count_leaves([5, 3, 7], [1, -1, -1], [2, -1, -1])` returns `2`.
- `count_leaves([], [], [])` returns `0`.
- `count_leaves([9, 8, 7, 1], [1, -1, -1, -1], [2, 3, -1, -1])` returns `2` — cell 3 is unreachable and is not counted.

## Constraints
- `0 <= len(vals) = len(left) = len(right) <= 40`
- every `left[i]` and `right[i]` is `-1` or an index into `vals`
- the walk from `0` never loops back
