# The First Clear

## Task
A warden lists lantern heights from shortest to tallest. Call `first_clear(heights, mark)` and return the position of the first lantern at least `mark` tall, or the number of lanterns when none clears the mark.

## Input
A list of heights sorted ascending (ties allowed), and the mark to clear. The list may be empty.

## Output
The position of the first lantern whose height is `>= mark`, or `len(heights)` when no lantern qualifies.

## Examples
- `first_clear([2, 4, 4, 6], 4)` returns `1`.
- `first_clear([2, 4, 6], 10)` returns `3`.
- `first_clear([], 5)` returns `0`.

## Constraints
- `0 <= len(heights) <= 40`
- heights are integers, `0 <= height <= 99`
- `heights` is sorted ascending (ties allowed)
- `mark` is an integer, `0 <= mark <= 99`
