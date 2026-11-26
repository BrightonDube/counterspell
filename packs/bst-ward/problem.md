# The Bough Order Ward

## Task
Call `bst_check(vals, left, right)` and return `1` when the stored tree keeps the search order — every mark in a left bough sits below its parent's mark and every mark in a right bough sits above it — and `0` when it breaks the rule.

## Input
`vals` holds each node's mark, `left` and `right` hold each node's child indexes; `-1` means no child there. The root is index `0`. Cells the walk never reaches do not belong to the tree, and a child index that does not point at a real cell counts as no child. The three lists have the same length.

## Output
`1` for a valid search tree, `0` otherwise; an empty tree is valid. Equal marks break the rule.

## Examples
- `bst_check([5, 3, 7], [1, -1, -1], [2, -1, -1])` returns `1`.
- `bst_check([5, 3, 2], [1, -1, -1], [2, -1, -1])` returns `0` — `2` may not sit on `5`'s right.
- `bst_check([], [], [])` returns `1`.

## Constraints
- `0 <= len(vals) = len(left) = len(right) <= 40`
- every `left[i]` and `right[i]` is `-1` or an index into `vals`
- the walk from `0` never loops back
