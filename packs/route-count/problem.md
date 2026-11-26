# The Route Count

## Task
A ward network links towers in pairs. Call `route_count(n, links)` and return how many towers a signal from tower `0` can reach — counting every other tower it can reach through the links, but not tower `0` itself.

## Input
`n` is the tower count (`0` to `15`), `links` lists undirected `[a, b]` pairs with `0 <= a, b < n`.

## Output
The number of towers reachable from `0`, not counting `0` itself.

## Examples
- `route_count(4, [[0, 1], [1, 2]])` returns `2` — towers `1` and `2`.
- `route_count(3, [])` returns `0` — no links, nothing reached.
- `route_count(1, [])` returns `0`.

## Constraints
- `0 <= n <= 15`, `0 <= len(links) <= 24` rows of `[a, b]` with `0 <= a, b < n`
