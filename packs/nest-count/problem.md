# The Nest Count

## Task
A seal `[a, b]` nests another `[c, d]` when `a < c` and `d < b` — strictly inside. Call `nest_pairs(spans)` and return how many pairs have one seal nesting the other.

## Input
`spans` lists the `[a, b]` seals, `0 <= a <= b <= 100`.

## Output
The count of nesting pairs; `0` when none nest.

## Examples
- `nest_pairs([[0, 4], [1, 2]])` returns `1`.
- `nest_pairs([[0, 4], [1, 4]])` returns `0` — a shared end is not nesting.
- `nest_pairs([[0, 9], [1, 8], [2, 7], [3, 6]])` returns `6` — every pair nests.

## Constraints
- `0 <= len(spans) <= 20` rows of `[a, b]` with `0 <= a <= b <= 100`
