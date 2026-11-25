# The Widest Canvas

## Task
Shelf blocks stand shoulder to shoulder along a wall, each at its own height. A canvas can be stretched across any run of neighbouring blocks whose heights are all at least the canvas height — its area is that height times how many blocks it spans. Call `widest_fit(wall)` and return the largest canvas area the wall can hold.

## Input
A list of block heights, each at most `9`.

## Output
Return the largest area any run of neighbouring blocks can hold.

## Examples
- `widest_fit([2, 1, 5, 6, 2, 3])` returns `10`. The `5` and `6` hold a canvas of height `5` across two blocks.
- `widest_fit([2, 1, 2])` returns `3`. A canvas of height `1` spans all three blocks.
- `widest_fit([])` returns `0`. An empty wall holds nothing.

## Constraints
The wall holds at most 40 blocks.
