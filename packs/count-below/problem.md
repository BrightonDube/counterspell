# The Count Below

## Task
A ledger lists toll marks from smallest to largest, repeats allowed. Call `count_below(marks, toll)` and return how many listed marks are strictly smaller than `toll`.

## Input
A list of marks sorted ascending (ties allowed), and the toll to compare against. The list may be empty.

## Output
The count of marks strictly smaller than `toll`.

## Examples
- `count_below([1, 2, 2, 4, 7], 4)` returns `3`.
- `count_below([1, 2, 2, 4, 7], 9)` returns `5`.
- `count_below([], 5)` returns `0`.

## Constraints
- `0 <= len(marks) <= 40`
- marks are integers, `0 <= mark <= 99`
- `marks` is sorted ascending (ties allowed)
- `toll` is an integer, `0 <= toll <= 99`
