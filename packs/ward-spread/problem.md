# The Ward Spread

## Task
A row of `cells` cells must hold `guards` identical guards, and no two guards may sit in neighbouring cells. Call `spread_ways(cells, guards)` and return the number of placements.

## Input
`cells` is the row length; `guards` is how many to place. Placing none is always one way.

## Output
The number of placements with no neighbouring guards.

## Examples
- `spread_ways(4, 2)` returns `3` — cells 0+2, 0+3 and 1+3.
- `spread_ways(5, 3)` returns `1` — cells 0, 2 and 4.
- `spread_ways(3, 0)` returns `1`.

## Constraints
- `0 <= cells <= 20`, `0 <= guards <= 20`
