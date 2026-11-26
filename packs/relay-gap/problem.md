# The Relay Gap

## Task
Relay seals may be picked, but after one ends at `e` the next must start at `e + gap` or later — `gap` marks after the end. Call `relay_count(spans, gap)` and return the most seals pickable.

## Input
`spans` lists the `[a, b]` seals; `gap` is the required spacing between picks, `0..20`.

## Output
The largest count of gap-respecting picks.

## Examples
- `relay_count([[0, 1], [2, 3], [4, 5]], 1)` returns `3` — adjacent picks are fine.
- `relay_count([[0, 1], [2, 3], [4, 5]], 2)` returns `2` — one bare mark is required.
- `relay_count([[0, 2], [2, 4], [4, 6]], 0)` returns `3` — zero gap means touching counts.

## Constraints
- `0 <= len(spans) <= 20` rows of `[a, b]` with `0 <= a <= b <= 100`; `0 <= gap <= 20`
