# The Ring Haul

## Task
Marks lie in a ring — the first and last mark are neighbours too. Call `ring_haul(marks)` and return the largest worth taking only non-adjacent marks.

## Input
`marks` lists the worths around the ring.

## Output
The largest reachable total; an empty ring hauls `0`.

## Examples
- `ring_haul([])` returns `0`.
- `ring_haul([9, 1, 1, 9])` returns `10` — one end plus one middle, never both ends.
- `ring_haul([2, 1, 1, 2])` returns `3` — the ring makes the ends neighbours.

## Constraints
- `0 <= len(marks) <= 30`; each worth is `0..100`
