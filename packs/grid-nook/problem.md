# The Grid Nook

## Task
A nook is a patch of open cells joined edge to edge. A grille of `1` open and `0` bricked cells reaches the keeper; call `nook_size(grid)` and return the size of the nook holding the top-left cell.

## Input
`grid` holds the cells in rows. Only up, down, left and right join cells. A bricked top-left cell gives nook size `0`.

## Output
The number of open cells reachable from the top-left through open neighbours.

## Examples
- `nook_size([[1, 1], [0, 1]])` returns `3`.
- `nook_size([[0, 1], [1, 1]])` returns `0` — the start is bricked.
- `nook_size([[1]])` returns `1`.

## Constraints
- `1 <= rows, cols <= 5`; every cell is `0` or `1`
