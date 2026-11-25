# The Valley Low

## Task
A cliff edge was carved with distinct marks sorted low to high, then rotated — a tail block slid to the front. Call `lowest_mark(rotated)` and return the smallest mark on the edge.

## Input
A list of distinct marks that is some rotation of a sorted ascending list. The edge always holds at least one mark.

## Output
The smallest mark in the list.

## Examples
- `lowest_mark([5, 7, 9, 1, 3])` returns `1`.
- `lowest_mark([3, 4, 5, 6, 7])` returns `3` — unrotated edges are legal.
- `lowest_mark([2, 1])` returns `1`.

## Constraints
- `1 <= len(rotated) <= 40`
- marks are integers, `0 <= mark <= 99`, all distinct
- `rotated` is a rotation of a sorted ascending list (rotation by 0 allowed)
