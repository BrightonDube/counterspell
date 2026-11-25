# The Harvest Quota

## Task
A picker may keep any unbroken run of marked vines, but the run's marks must add to no more than the `quota`. Call `longest_harvest(marks, quota)` and return the length of the longest run the picker may keep.

## Input
A list of vine marks and a quota. Each mark is at least `1` and at most `9`.

## Output
Return the length of the longest contiguous run whose marks sum to at most `quota`, or `0` when no run fits.

## Examples
- `longest_harvest([2, 1, 3, 1], 5)` returns `3`. The last three vines sum to `5`.
- `longest_harvest([9], 8)` returns `0`. The only vine is too heavy alone.
- `longest_harvest([1, 1, 1], 3)` returns `3`. The whole row fits the quota.

## Constraints
- The row holds at most 40 marks.
- Each mark is between `1` and `9`.
- The quota is between `0` and `100`.
