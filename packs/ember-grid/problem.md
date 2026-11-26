# The Ember Grid

## Task
A grid of tolls is crossed top-left to bottom-right, stepping only down or right. Call `grid_min(grid)` and return the cheapest total paid on the cells you stand on, both ends included.

## Input
`grid` lists the rows of tolls; every row is the same width.

## Output
The cheapest standing toll.

## Examples
- `grid_min([[5]])` returns `5`.
- `grid_min([[1, 9], [1, 1]])` returns `3`.
- `grid_min([[1, 3, 1], [1, 5, 1], [4, 2, 1]])` returns `7`.

## Constraints
- `1 <= rows, cols <= 6`; each toll is `0..9`
