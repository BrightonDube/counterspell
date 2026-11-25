# The Twin Tides

## Task
Two tide tables each list water heights, smallest first. Call `merge_tides(first, second)` and return one table holding every height from both, still ordered smallest first. Two empty tables return an empty list.

## Input
Two lists of heights. Each list is ordered smallest first.

## Output
Return one ordered list holding every height from both tables.

## Examples
`merge_tides([1, 3], [2, 4])` returns `[1, 2, 3, 4]`. The four heights interleave in order.

## Constraints
Each table has at most 20 heights, ordered smallest first. Each height is from `0` to `99`.
