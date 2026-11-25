# The Ember Index

## Task
A rack holds ember marks sorted from lowest to highest, each mark distinct. Call `find_ember(embers, target)` and return the shelf position of `target`, or `-1` when the mark is not on the rack.

## Input
A list of marks sorted strictly ascending, and the mark to find. The rack may be empty.

## Output
The position of `target` as a zero-based index, or `-1` when absent.

## Examples
- `find_ember([1, 3, 5, 7, 9], 5)` returns `2`.
- `find_ember([1, 3, 5, 7], 8)` returns `-1`.
- `find_ember([], 3)` returns `-1`.

## Constraints
- `0 <= len(embers) <= 40`
- marks are integers, `-99 <= mark <= 99`
- `embers` is sorted strictly ascending (no repeats)
- `target` is an integer, `-99 <= target <= 99`
