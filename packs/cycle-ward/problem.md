# The Cycle Ward

## Task
A ward stands wherever a walk can leave a node, wander, and return without reusing the arc it just crossed. Call `cycle_check(edges, n)` and return `1` when the web holds such a ring, `0` when it is a forest.

## Input
`edges` lists the arcs as `[a, b]` pairs between nodes `0..n-1`; `n` is the node count. A self-loop is always a ring, and two copies of one arc ring each other.

## Output
`1` when a ring exists, `0` otherwise.

## Examples
- `cycle_check([[0, 1], [1, 2], [0, 2]], 3)` returns `1`.
- `cycle_check([[0, 1], [1, 2]], 3)` returns `0`.
- `cycle_check([[0, 1], [0, 1]], 2)` returns `1` — the doubled arc rings.

## Constraints
- `0 <= len(edges) <= 20` rows of two endpoints each; `0 <= n <= 8`
