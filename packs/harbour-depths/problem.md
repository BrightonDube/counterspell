# The Harbour Depths

## Task
A harbour wall is a row of posts with heights. Water held between two posts fills to the lower post's height across the gap between them. Call `deepest_water(heights)` and return the most water any pair of posts can hold, as an integer: the lower post's height times the number of gaps between them. A row with fewer than two posts holds none.

## Input
A list of post heights in row order.

## Output
Return the greatest held water as an integer.

## Examples
`deepest_water([1, 3, 2, 4])` returns `6`. The posts of height `3` and `4` stand two gaps apart; the lower height `3` times `2` gives `6`.

## Constraints
The row has at most 40 posts. Each height is from `0` to `9`.
