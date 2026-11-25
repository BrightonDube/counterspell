# The Highest Floor

## Task
A stairwell's steps are marked from lowest to highest, repeats allowed. Call `highest_floor(steps, reach)` and return the mark of the tallest step no higher than `reach`, or `-1` when every step lies above it.

## Input
A list of step marks sorted ascending (ties allowed), and the reach to fit under. The list may be empty.

## Output
The greatest mark `<= reach`, or `-1` when no step qualifies.

## Examples
- `highest_floor([1, 3, 3, 7, 9], 6)` returns `3`.
- `highest_floor([5, 7, 9], 4)` returns `-1`.
- `highest_floor([5, 7, 9], 9)` returns `9`.

## Constraints
- `0 <= len(steps) <= 40`
- marks are integers, `0 <= mark <= 99`
- `steps` is sorted ascending (ties allowed)
- `reach` is an integer, `0 <= reach <= 99`
