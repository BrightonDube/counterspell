# The Toll Count

## Task
The keeper sets a cap `t`. Call `toll_count(edges, weights, t)` and return how many priced arcs cost `t` or less.

## Input
`edges` lists the arcs as `[a, b]` pairs; `weights[k]` prices `edges[k]`; `t` is the cap. Extra entries in either list are ignored.

## Output
The count of arcs whose toll does not exceed `t`.

## Examples
- `toll_count([[0, 1], [1, 2], [2, 3]], [5, 15, 25], 10)` returns `1`.
- `toll_count([[0, 1]], [10], 10)` returns `1` — the cap is inclusive.
- `toll_count([], [], 10)` returns `0`.

## Constraints
- `0 <= len(edges) <= 24` rows of two endpoints each; `0 <= len(weights) <= 24`; `0 <= t <= 100000`
