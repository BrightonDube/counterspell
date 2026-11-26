# The Mark Cover

## Task
Wax seals cover stretches of a lintel — each `[a, b]` covers every mark from `a` through `b`, both counted. Call `cover_length(spans)` and return how many marks carry wax.

## Input
`spans` lists the `[a, b]` seals, `0 <= a <= b <= 100`.

## Output
The count of covered marks; an empty lintel covers `0`.

## Examples
- `cover_length([[0, 2]])` returns `3` — marks `0`, `1`, `2`.
- `cover_length([[0, 2], [1, 3]])` returns `4` — the seals overlap.
- `cover_length([[0, 0], [2, 2], [4, 4]])` returns `3`.

## Constraints
- `0 <= len(spans) <= 20` rows of `[a, b]` with `0 <= a <= b <= 100`
