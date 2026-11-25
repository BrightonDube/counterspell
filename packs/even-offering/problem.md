# The Even Offering

## Task
A row of offering bowls must be shared among `parts` keepers as unbroken runs — each keeper takes one consecutive stretch. Call `smallest_largest(offers, parts)` and return the smallest possible value of the heaviest share.

## Input
A list of offering marks and the keeper count `parts`. The list is never empty, and `parts` never exceeds the number of bowls.

## Output
The minimum achievable value of the largest share's total.

## Examples
- `smallest_largest([2, 3, 1, 4, 1], 2)` returns `6` — split `[2, 3, 1]` and `[4, 1]` (or `[2, 3]` and `[1, 4, 1]`).
- `smallest_largest([7, 7, 7], 3)` returns `7` — every keeper takes one bowl.
- `smallest_largest([1, 2, 3], 1)` returns `6` — one keeper takes it all.

## Constraints
- `1 <= len(offers) <= 40`
- marks are integers, `1 <= mark <= 99`
- `1 <= parts <= len(offers)`
- every keeper takes one non-empty unbroken run
