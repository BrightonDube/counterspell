# The Ember Split

## Task
The pile must split into two non-empty halves whose marks total the same. Call `split_count(marks)` and return the number of such splits — a split is chosen embers, not cut positions.

## Input
`marks` holds the marks. Two splits that pick the same embers are one split.

## Output
The number of equal-total two-way splits, each side non-empty.

## Examples
- `split_count([1, 1, 2])` returns `1` — `{1,1}` against `{2}`.
- `split_count([1, 2, 3])` returns `1` — `{3}` against `{1,2}`.
- `split_count([1, 2])` returns `0` — the total is odd.

## Constraints
- `0 <= len(marks) <= 16`, `0 <= marks[i] <= 20`
