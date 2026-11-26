# The Depend Rite

## Task
A clean ledger lets each key settle on one value — key `x` determines `y`. Call `depend_rite(rows)` and return `1` when every repeated key agrees on its value, `0` when any key splits.

## Input
`rows` lists `[x, y]` pairs, fields `0` to `30`.

## Output
`1` when `x -> y` holds across the ledger, `0` on a split.

## Examples
- `depend_rite([[1, 5], [2, 7], [1, 9]])` returns `0` — key `1` settles on both `5` and `9`.
- `depend_rite([[1, 5], [1, 5], [2, 7]])` returns `1`.
- `depend_rite([])` returns `1`.

## Constraints
- `0 <= len(rows) <= 20` rows of `[x, y]` with `0 <= x, y <= 30`
