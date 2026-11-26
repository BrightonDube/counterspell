# The Ember Road

## Task
The keeper needs the cheapest road from node `0` to node `n-1` through a priced web. Call `road_toll(edges, weights, n)` and return the lowest total toll.

## Input
`edges` lists the arcs as `[a, b]` pairs between nodes `0..n-1`; `weights[k]` prices `edges[k]`; `n` is the node count. Arcs run both ways. Doubled arcs each carry their own price.

## Output
The cheapest total toll from `0` to `n-1`, or `-1` when no road reaches.

## Examples
- `road_toll([[0, 1], [1, 2]], [5, 7], 3)` returns `12`.
- `road_toll([[0, 1], [0, 2], [1, 2]], [9, 2, 1], 3)` returns `2` — the direct arc wins.
- `road_toll([[1, 2]], [3], 3)` returns `-1` — node `0` is stranded.

## Constraints
- `0 <= len(edges) <= 24` rows of two endpoints each; `0 <= len(weights) <= 24`; `0 <= n <= 8`
