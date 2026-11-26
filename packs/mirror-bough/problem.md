# The Mirror Bough

## Task
Call `mirror_check(vals, left, right)` and return `1` when the tree reads the same folded along the root — every left bough answers a right bough with the same marks and mirrored shape — and `0` when it does not.

## Input
`vals` holds each node's mark, `left` and `right` hold each node's child indexes; `-1` means no child there. The root is index `0`. Cells the walk never reaches do not belong to the tree, and a child index that does not point at a real cell counts as no child. The three lists have the same length.

## Output
`1` for a mirrored tree (including an empty one), `0` otherwise.

## Examples
- `mirror_check([4, 2, 2, 1, 3, 3, 1], [1, 3, 5, -1, -1, -1, -1], [2, 4, 6, -1, -1, -1, -1])` returns `1`.
- `mirror_check([4, 2, 3], [1, -1, -1], [2, -1, -1])` returns `0` — the bough marks differ.
- `mirror_check([], [], [])` returns `1`.

## Constraints
- `0 <= len(vals) = len(left) = len(right) <= 40`
- every `left[i]` and `right[i]` is `-1` or an index into `vals`
- the walk from `0` never loops back
