# The Flood Line

## Task
After the storm, a wall of posts holds rainwater between them. Between the tallest posts to each side of a gap, water fills to the lower of the two tallest bounds minus that spot's own height. Call `trapped_water(heights)` and return the total water the wall traps, as an integer. A wall with fewer than three posts traps none.

## Input
A list of post heights in row order.

## Output
Return the total trapped water as an integer.

## Examples
`trapped_water([3, 0, 1, 0, 3])` returns `8`. The two end posts bound the dip: the gaps hold `3`, `2` and `3` water.

## Constraints
The wall has at most 40 posts. Each height is from `0` to `9`.
