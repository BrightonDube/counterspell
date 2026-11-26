# The Grille Toll

## Task
A grille of open `0`s and blocked `1`s is crossed top-left to bottom-right, stepping only down or right. Call `toll_grid(grid)` and return the fewest cells a crossing stands on, both ends counted.

## Input
`grid` lists the rows; `0` is open, `1` is blocked.

## Output
The fewest standing cells, or `-1` when no crossing survives.

## Examples
- `toll_grid([[0]])` returns `1`.
- `toll_grid([[0, 0], [0, 0]])` returns `3`.
- `toll_grid([[0, 1], [1, 0]])` returns `-1` — the lanes are walled.

## Constraints
- `1 <= rows, cols <= 6`; cells are `0` or `1`
