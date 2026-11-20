# Diagonal Sum

## Task

Given a grid of integers `grid`, sum the cells on the main diagonal: the cells where the row index equals the column index. The grid may be rectangular, so the diagonal stops at the shorter side.

## Input

- `grid`: a list of lists of integers with 0 to 6 rows and 0 to 6 columns; each cell is between -9 and 9.

## Output

An integer: the sum of the diagonal cells. An empty grid sums to 0.

## Examples

| grid | result |
|---|---|
| `[[1, 2], [3, 4]]` | `5` |
| `[[-1]]` | `-1` |
| `[]` | `0` |

## Constraints

- `grid` has between 0 and 6 rows and 0 and 6 columns.
- Each cell is an integer between -9 and 9.
- Rows may be empty.

This is a test pack used only by the ghost engine's conformance tests. It is not in the catalogue.
