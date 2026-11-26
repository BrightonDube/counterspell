# The Shelf Toll

## Task
A shelf of worths is crossed top-left to bottom-right, stepping only down or right, and the keeper keeps every worth stood on. Call `shelf_max(grid)` and return the largest total a crossing can gather.

## Input
`grid` lists the rows of worths; every row is the same width.

## Output
The largest gathered worth.

## Examples
- `shelf_max([[5]])` returns `5`.
- `shelf_max([[1, 9], [1, 1]])` returns `11` — the top-right route gathers `1 + 9 + 1`.
- `shelf_max([[1, 3, 1], [1, 5, 1], [4, 2, 1]])` returns `12`.

## Constraints
- `1 <= rows, cols <= 6`; each worth is `0..9`
