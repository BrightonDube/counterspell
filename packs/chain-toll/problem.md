# The Chain Tolls

## Task
Cheap roads can tie. Call `toll_ways(edges, weights, n)` and return how many distinct cheapest-toll routes run from `0` to `n-1`.

## Input
`edges` lists the arcs as `[a, b]` pairs between nodes `0..n-1`; `weights[k]` prices `edges[k]`; `n` is the node count. Arcs run both ways; doubled arcs each offer their price.

## Output
The count of routes achieving the minimum toll, `0` when no route reaches, `1` when `0` is already the target.

## Examples
- `toll_ways([[0, 1], [1, 2], [0, 2]], [5, 7, 12], 3)` returns `2` — the direct and the hop both cost 12.
- `toll_ways([[0, 1], [0, 2], [1, 3], [2, 3]], [5, 5, 5, 5], 4)` returns `2`.
- `toll_ways([[0, 1], [0, 2]], [5, 5], 4)` returns `0` — node `3` is stranded.

## Constraints
- `0 <= len(edges) <= 24` rows of two endpoints each; `0 <= len(weights) <= 24`; `0 <= n <= 8`
