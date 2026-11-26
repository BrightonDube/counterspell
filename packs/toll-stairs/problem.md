# The Toll Stairs

## Task
Each stair `i` costs `costs[i]` to stand on. From any stair you may step up one or two. Call `toll_stairs(costs)` and return the smallest toll to stand above the last stair.

## Input
`costs` lists the standing tolls per stair. You may start on stair `0` or `1` for free.

## Output
The smallest total toll; an empty stairwell costs `0`.

## Examples
- `toll_stairs([])` returns `0`.
- `toll_stairs([10, 15, 20])` returns `15`.
- `toll_stairs([1, 100, 1, 1, 1, 100, 1, 1, 100, 1])` returns `6`.

## Constraints
- `0 <= len(costs) <= 30`; each toll is `0..100`
