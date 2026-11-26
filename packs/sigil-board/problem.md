# The Sigil Board

## Task
A square board of `n` rows and columns must hold `n` sigils so that no two share a row, a column, or a diagonal. Call `sigil_count(n)` and return the number of distinct placements.

## Input
`n` is the board's side length.

## Output
The number of valid `n`-sigil placements.

## Examples
- `sigil_count(4)` returns `2`.
- `sigil_count(1)` returns `1`.
- `sigil_count(3)` returns `0` — no placement fits.

## Constraints
- `0 <= n <= 8`
