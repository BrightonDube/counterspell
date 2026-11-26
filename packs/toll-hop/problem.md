# The Toll Hop

## Task
The keeper allows at most `k` arcs on the road from `0` to `n-1`. Call `hop_toll(edges, weights, n, k)` and return the cheapest toll within the hop limit.

## Input
`edges` lists the arcs as `[a, b]` pairs between nodes `0..n-1`; `weights[k]` prices `edges[k]`; `n` is the node count; `k` is the hop cap. Arcs run both ways.

## Output
The cheapest toll using at most `k` arcs, or `-1` when no road fits the cap.

## Examples
- `hop_toll([[0, 1], [1, 2]], [5, 7], 3, 2)` returns `12`.
- `hop_toll([[0, 1], [1, 2]], [5, 7], 3, 1)` returns `-1` — one hop cannot reach.
- `hop_toll([[0, 1], [1, 2], [0, 2]], [1, 1, 50], 3, 1)` returns `50` — the cap forces the dear arc.

## Constraints
- `0 <= len(edges) <= 24` rows of two endpoints each; `0 <= len(weights) <= 24`; `0 <= n <= 8`; `0 <= k <= 8`
