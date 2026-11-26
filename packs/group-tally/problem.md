# The Group Tally

## Task
The ledger sums its values by key — each key's tally is the total of its rows. Call `group_tally(rows)` and return the largest tally any key reaches; an empty ledger tallies `0`.

## Input
`rows` lists `[key, value]` pairs, fields `0` to `30`.

## Output
The maximum per-key value sum, or `0` for an empty ledger.

## Examples
- `group_tally([[1, 5], [2, 7], [1, 9], [3, 2]])` returns `14` — key `1` tallies `5+9`.
- `group_tally([])` returns `0`.
- `group_tally([[2, 3], [2, 4], [1, 1]])` returns `7`.

## Constraints
- `0 <= len(rows) <= 20` rows of `[key, value]` with `0 <= key, value <= 30`
