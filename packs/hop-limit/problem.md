# The Hop Limit

## Task
A sigil packet dies after `ttl` link-crossings — each crossing spends one tick of life. Call `hop_limit(n, links, ttl)` and return how many other towers the packet can reach before its life runs out.

## Input
`n` towers (`0` to `15`), undirected `links` rows `[a, b]`, and `ttl` hops of life (`0` to `15`).

## Output
The count of towers whose shortest path from `0` takes at most `ttl` links, not counting `0` itself.

## Examples
- `hop_limit(5, [[0, 1], [1, 2], [2, 3], [3, 4]], 2)` returns `2` — towers `1` and `2` are close enough; `3` and `4` are not.
- `hop_limit(4, [[0, 1], [0, 2], [0, 3]], 1)` returns `3`.
- `hop_limit(5, [[0, 1], [1, 2], [2, 3], [3, 4]], 0)` returns `0`.

## Constraints
- `0 <= n <= 15`, `0 <= len(links) <= 24`, `0 <= ttl <= 15`
- `links` rows whose endpoints lie outside `0..n-1` name no tower and are skipped.
