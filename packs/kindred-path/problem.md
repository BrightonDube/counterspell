# The Kindred Path

## Task
Call `ember_path(vals, left, right, target)` and return `1` when some root-to-leaf walk of the stored tree sums its marks to `target`, and `0` when none does.

## Input
`vals` holds each node's mark, `left` and `right` hold each node's child indexes; `-1` means no child there. The root is index `0`. Cells the walk never reaches do not belong to the tree, and a child index that does not point at a real cell counts as no child. The three lists have the same length.

## Output
`1` when a full root-to-leaf path hits `target`, `0` otherwise; an empty tree answers `0`.

## Examples
- `ember_path([5, 4, 8, 11, 13, 4, 7, 2, 1], [1, 3, 5, -1, -1, -1, -1, -1, -1], [2, 4, 6, -1, -1, 7, 8, -1, -1])` returns `1` for `target = 22` (`5 + 4 + 11 + 2`).
- `ember_path([5, 4, 8], [1, -1, -1], [2, -1, -1], 10)` returns `0` — no leaf path sums to ten.
- `ember_path([], [], [], 0)` returns `0`.

## Constraints
- `0 <= len(vals) = len(left) = len(right) <= 40`
- every `left[i]` and `right[i]` is `-1` or an index into `vals`
- `-999 <= target <= 999`
- the walk from `0` never loops back
