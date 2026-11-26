# The Calm Gaps

## Task
Marks in a row carry worth, but taking two neighbouring marks breaks the rite. Call `calm_haul(marks)` and return the largest total worth taking only non-adjacent marks.

## Input
`marks` lists the worths in row order.

## Output
The largest reachable total; an empty row hauls `0`.

## Examples
- `calm_haul([])` returns `0`.
- `calm_haul([2, 7, 9, 3, 1])` returns `12` — `2`, `9` and `1`.
- `calm_haul([1, 2, 3, 1])` returns `4` — the `1` and `3` carry the haul.

## Constraints
- `0 <= len(marks) <= 30`; each worth is `0..100`
