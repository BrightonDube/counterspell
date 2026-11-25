# The Rotated Pantry

## Task
A pantry shelf holds distinct supply marks sorted low to high, but the shelf was rotated — a tail block slid to the front. Call `find_in_rotated(stores, target)` and return the position of `target`, or `-1` when absent.

## Input
A list of distinct marks that is some rotation of a sorted ascending list, and the mark to find. The shelf may be empty.

## Output
The position of `target` as a zero-based index, or `-1` when absent.

## Examples
- `find_in_rotated([5, 7, 9, 1, 3], 9)` returns `2`.
- `find_in_rotated([5, 7, 9, 1, 3], 4)` returns `-1`.
- `find_in_rotated([4, 1, 2, 3], 4)` returns `0`.

## Constraints
- `0 <= len(stores) <= 40`
- marks are integers, `0 <= mark <= 99`, all distinct
- `stores` is a rotation of a sorted ascending list (rotation by 0 allowed)
- `target` is an integer, `0 <= target <= 99`
