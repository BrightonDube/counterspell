# The Ember Edges

## Task
A rack holds ember marks sorted low to high, repeats allowed. Call `ember_edges(embers, target)` and return the first and last shelf positions holding `target` as a two-mark list, or `[-1, -1]` when the mark is absent.

## Input
A list of marks sorted ascending (ties allowed), and the mark to bound. The rack may be empty.

## Output
`[first, last]` zero-based positions, or `[-1, -1]` when absent.

## Examples
- `ember_edges([1, 3, 3, 3, 5], 3)` returns `[1, 3]`.
- `ember_edges([1, 3, 5], 4)` returns `[-1, -1]`.
- `ember_edges([2, 2, 2], 2)` returns `[0, 2]`.

## Constraints
- `0 <= len(embers) <= 40`
- marks are integers, `0 <= mark <= 99`
- `embers` is sorted ascending (ties allowed)
- `target` is an integer, `0 <= target <= 99`
