# The Key Count

## Task
A ledger's keys mark its fingers — repeats point at the same finger. Call `key_count(rows)` and return how many distinct keys the ledger holds.

## Input
`rows` lists `[key, value]` pairs, both fields `0` to `30`.

## Output
The count of distinct keys.

## Examples
- `key_count([[1, 5], [2, 7], [1, 9], [3, 0]])` returns `3` — key `1` repeats.
- `key_count([])` returns `0`.
- `key_count([[2, 2], [2, 2]])` returns `1`.

## Constraints
- `0 <= len(rows) <= 20` rows of `[key, value]` with `0 <= key, value <= 30`
