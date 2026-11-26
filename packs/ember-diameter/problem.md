# The Ember Diameter

## Task
Call `tree_diameter(vals, left, right)` and return the tree's diameter — the edge count of the longest walk between any two nodes, whether or not it passes through the root.

## Input
`vals` holds each node's mark, `left` and `right` hold each node's child indexes; `-1` means no child there. The root is index `0`. Cells the walk never reaches do not belong to the tree, and a child index that does not point at a real cell counts as no child. The three lists have the same length.

## Output
The longest node-to-node walk measured in edges; `0` for an empty tree or a lone root.

## Examples
- `tree_diameter([1, 2, 3, 4, 5], [1, 3, -1, -1, -1], [2, 4, -1, -1, -1])` returns `3` — leaf `4` up to the root and down to `5`.
- `tree_diameter([1, 2, 3, 4], [1, 2, -1, -1], [-1, -1, 3, -1])` returns `3` — `4` up through `3`, `2`, `1`... wait, `3` sits under `2`.
- `tree_diameter([], [], [])` returns `0`.

## Constraints
- `0 <= len(vals) = len(left) = len(right) <= 40`
- every `left[i]` and `right[i]` is `-1` or an index into `vals`
- the walk from `0` never loops back
