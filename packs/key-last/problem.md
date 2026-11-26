# The Key Last

## Task
A ledger keeps every write — the last row for a key is the value that stands. Call `key_last(rows, k)` and return the value of the last row carrying key `k`; an absent key answers `-1`.

## Input
`rows` lists `[key, value]` pairs with fields `0..30`; `k` runs `0..30`.

## Output
The latest value for `k`, or `-1` when `k` never appears.

## Examples
- `key_last([[1, 5], [2, 7], [1, 9]], 1)` returns `9` — the second write stands.
- `key_last([[1, 5], [2, 7], [1, 9]], 3)` returns `-1`.
- `key_last([], 1)` returns `-1`.

## Constraints
- `0 <= len(rows) <= 20` rows of `[key, value]` with `0 <= key, value <= 30`; `0 <= k <= 30`
