# Diagonal Sum

Given a grid of whole numbers, add up the cells where the row index equals the column index. Only cells that exist are counted, so a grid that is not square uses the shorter side.

- An empty grid has no cells, so the sum is 0.
- A grid with rows but no columns also sums to 0.

## Function

Write `diagonal_sum(grid)`. Return the sum as an integer. The grid is a list of rows, and each row is a list of integers of the same length.

## Constraints

- The grid has between 0 and 6 rows and between 0 and 6 columns.
- Each cell is between -9 and 9.

## Examples

| grid | result |
|---|---|
| `[[1, 2], [3, 4]]` | `5` |
| `[[1, 2, 3]]` | `1` |
| `[]` | `0` |

This is a test pack used only by the ghost engine's conformance tests. It is not in the catalogue.
