# The Ember Floor

## Task
The keeper keeps the smallest embers in a floor pile. Call `floor_sum(marks, k)` and return the total of the `k` smallest marks.

## Input
`marks` holds the marks; `k` says how many of the smallest count. When `k` is larger than the pile, every mark counts. When the pile is empty or `k` is zero, the total is `0`.

## Output
The sum of the `k` smallest marks.

## Examples
- `floor_sum([5, 1, 4, 2, 3], 2)` returns `3` — `1` and `2`.
- `floor_sum([7, 7, 7], 5)` returns `21`.
- `floor_sum([], 3)` returns `0`.

## Constraints
- `0 <= len(marks) <= 40`, `-999 <= marks[i] <= 999`, `0 <= k <= 40`
