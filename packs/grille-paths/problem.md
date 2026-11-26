# The Grille Paths

## Task
A grille of cells stands between the keeper and the far wall: `1` is open, `0` is bricked. A route moves only right or down. Call `path_count(grid)` and return the number of routes from the top-left cell to the bottom-right.

## Input
`grid` is the grille, rows of cells, `1` open and `0` bricked. A bricked start or end admits no route.

## Output
The number of right-or-down routes through open cells.

## Examples
- `path_count([[1, 1], [1, 1]])` returns `2`.
- `path_count([[1, 0], [1, 1]])` returns `1` — the right cell is bricked.
- `path_count([[0, 1], [1, 1]])` returns `0` — the start is bricked.

## Constraints
- `1 <= rows, cols <= 5`; every cell is `0` or `1`
