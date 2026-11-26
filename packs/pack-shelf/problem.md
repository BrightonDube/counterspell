# The Pack Shelf

## Task
Keepsakes weigh `items[i][0]` and carry `items[i][1]` worth; the shelf holds `cap` weight. Call `pack_worth(items, cap)` and return the greatest worth that fits — each keepsake once.

## Input
`items` lists `[weight, worth]` pairs; `cap` is the shelf's weight limit.

## Output
The greatest fitting worth; an empty shelf or zero cap gathers `0`.

## Examples
- `pack_worth([[1, 1]], 1)` returns `1`.
- `pack_worth([[2, 3], [3, 4], [4, 5]], 5)` returns `7` — the two lighter keepsakes.
- `pack_worth([[5, 10]], 3)` returns `0` — nothing fits.

## Constraints
- `0 <= len(items) <= 12` rows of `[weight, worth]`, each `1..60`; `0 <= cap <= 60`
