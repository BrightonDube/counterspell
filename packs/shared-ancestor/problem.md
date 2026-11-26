# The Shared Ancestor

## Task
Call `first_ancestor(vals, left, right, a, b)` and return the lowest node index that stands above both `a` and `b` in the stored tree — the deepest cell both walks back to the root share.

## Input
`vals` holds each node's mark, `left` and `right` hold each node's child indexes; `-1` means no child there. The root is index `0`. Cells the walk never reaches do not belong to the tree, and a child index that does not point at a real cell counts as no child. The three lists have the same length.

## Output
The shared ancestor's index, or `-1` when `a` or `b` is not a reachable node.

## Examples
- `first_ancestor([5, 3, 7, 2, 4, 6, 8], [1, 3, 5, -1, -1, -1, -1], [2, 4, 6, -1, -1, -1, -1], 3, 4)` returns `1` — both sit under node `1`.
- `first_ancestor([5, 3, 7], [1, -1, -1], [2, -1, -1], 1, 2)` returns `0`.
- `first_ancestor([5, 3, 7], [1, -1, -1], [2, -1, -1], 1, 9)` returns `-1` — `9` is not a node.

## Constraints
- `0 <= len(vals) = len(left) = len(right) <= 40`
- every `left[i]` and `right[i]` is `-1` or an index into `vals`
- `0 <= a, b <= 39`; indexes outside the tree answer `-1`
- the walk from `0` never loops back
