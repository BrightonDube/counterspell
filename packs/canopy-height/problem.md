# The Canopy Height

## Task
Call `tree_height(vals, left, right)` and return the height of the stored tree — the number of nodes on the longest root-to-leaf walk.

## Input
`vals` holds each node's mark, `left` and `right` hold each node's child indexes; `-1` means no child there. The root is index `0`. Cells the walk never reaches do not belong to the tree, and a child index that does not point at a real cell counts as no child. The three lists have the same length.

## Output
The height in nodes: an empty tree gives `0`, a lone root gives `1`.

## Examples
- `tree_height([5, 3, 7], [1, -1, -1], [2, -1, -1])` returns `2`.
- `tree_height([], [], [])` returns `0`.
- `tree_height([1, 2, 3], [1, 2, -1], [-1, -1, -1])` returns `3` — a skewed chain still counts every level.

## Constraints
- `0 <= len(vals) = len(left) = len(right) <= 40`
- every `left[i]` and `right[i]` is `-1` or an index into `vals`
- the walk from `0` never loops back
