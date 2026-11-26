# The Ember Partition

## Task
The pile must split into `ways` non-empty heaps whose marks each total the same. Call `partition_count(marks, ways)` and return the number of splits — heaps are unlabelled, so swapping two whole heaps is not a new split.

## Input
`marks` holds the marks and `ways` the number of heaps.

## Output
The number of equal-total `ways`-heap splits.

## Examples
- `partition_count([1, 1, 2], 2)` returns `1` — `{1,1}` against `{2}`.
- `partition_count([1, 1, 2, 2], 2)` returns `2` — either `1` pairs with either `2`.
- `partition_count([1, 2, 3], 3)` returns `0` — no three equal heaps exist.

## Constraints
- `0 <= len(marks) <= 12`, `0 <= marks[i] <= 10`, `1 <= ways <= 4`
