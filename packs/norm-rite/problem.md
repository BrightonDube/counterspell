# The Norm Rite

## Task
A share rite splits a tally into hundredths — each mark's share is `floor(mark * 100 / total)`, and what the floors drop is the remainder the rite must report. Call `norm_rite(v)` and return `100` minus the sum of those floored shares.

## Input
`v` is the tally strip.

## Output
`100 - sum(floor(v[i] * 100 / total))`, or `0` when the total is not positive.

## Examples
- `norm_rite([1, 3, 6])` returns `0` — `10 + 30 + 60` lands whole.
- `norm_rite([1, 1, 1])` returns `1` — `33 + 33 + 33` drops one.
- `norm_rite([])` returns `0`.

## Constraints
- `0 <= len(v) <= 24`, `0 <= v[i] <= 40`
