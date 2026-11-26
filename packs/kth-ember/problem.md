# The Kth Ember

## Task
The keeper asks for the ember sitting at rank `k` among the smallest. Call `kth_mark(marks, k)` and return that mark; repeats keep their own ranks.

## Input
`marks` holds the marks; `k` is one-based. When the pile holds fewer than `k` marks the answer is `-1`.

## Output
The `k`-th smallest mark, or `-1` when the pile is too small.

## Examples
- `kth_mark([5, 1, 4, 2, 3], 2)` returns `2`.
- `kth_mark([4, 4, 4], 2)` returns `4` — repeats keep their ranks.
- `kth_mark([7], 2)` returns `-1`.

## Constraints
- `0 <= len(marks) <= 40`, `-999 <= marks[i] <= 999`, `0 <= k <= 40`
