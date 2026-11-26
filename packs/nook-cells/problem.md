# The Nook Cells

## Task
A nook is a grid of open `1`s and blocked `0`s. Cross it top-left to bottom-right stepping only down or right through open cells. Call `nook_ways(grid)` and return the crossing count.

## Input
`grid` lists the rows; `1` is open, `0` is blocked. The start and end cells may be blocked — then nothing crosses.

## Output
The crossing count; `0` when no path survives.

## Examples
- `nook_ways([[1]])` returns `1`.
- `nook_ways([[0]])` returns `0` — the only cell is blocked.
- `nook_ways([[1, 1], [1, 1]])` returns `2`.

## Constraints
- `1 <= rows, cols <= 6`; cells are `0` or `1`
