# The Grille Ways

## Task
A grille of `rows` × `cols` cells is crossed from the top-left to the bottom-right, stepping only down or right. Call `grille_ways(rows, cols)` and return the step-order count.

## Input
`rows` and `cols` are the grille's dimensions, `0..10` each.

## Output
The step-order count; a zero-sized grille offers no crossing.

## Examples
- `grille_ways(1, 1)` returns `1`.
- `grille_ways(2, 2)` returns `2`.
- `grille_ways(3, 3)` returns `6`.

## Constraints
- `0 <= rows, cols <= 10`
