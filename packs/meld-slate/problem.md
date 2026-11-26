# The Meld Slate

## Task
Two slates arrive already ordered. Call `meld(a, b)` and return one ordered list holding every mark from both.

## Input
`a` and `b` each hold marks in ascending order; either may be empty.

## Output
The merged list in ascending order, keeping every mark — repeats included.

## Examples
- `meld([1, 3, 5], [2, 4])` returns `[1, 2, 3, 4, 5]`.
- `meld([1, 2], [2, 3])` returns `[1, 2, 2, 3]` — repeats stay.
- `meld([], [4])` returns `[4]`.

## Constraints
- `0 <= len(a), len(b) <= 40`, `-999 <= a[i], b[i] <= 999`
