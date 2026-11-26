# The Lone Branches

## Task
Call `single_children(vals, left, right)` and return how many reachable nodes carry exactly one child — a lone branch, left or right.

## Input
`vals` holds each node's mark, `left` and `right` hold each node's child indexes; `-1` means no child there. The root is index `0`. Cells the walk never reaches do not belong to the tree, and a child index that does not point at a real cell counts as no child. The three lists have the same length.

## Output
The count of nodes with precisely one child; `0` when none or the tree is empty.

## Examples
- `single_children([5, 3, 7], [1, -1, -1], [2, -1, -1])` returns `0` — the root has two children.
- `single_children([4, 2], [1, -1], [-1, -1])` returns `1` — the root has only a left branch.
- `single_children([], [], [])` returns `0`.

## Constraints
- `0 <= len(vals) = len(left) = len(right) <= 40`
- every `left[i]` and `right[i]` is `-1` or an index into `vals`
- the walk from `0` never loops back
