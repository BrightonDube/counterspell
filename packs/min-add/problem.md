# Fewest New Runes

## Task
A fragile inscription of `(` and `)` runes must be made sound by inserting extra runes of the same two kinds anywhere — none may be removed or moved. Call `fewest_inserts(inscription)` and return the fewest runes that must be added.

## Input
A string of `(` and `)` runes. It may be empty.

## Output
Return the smallest number of insertions that make the inscription sound.

## Examples
- `fewest_inserts("(())")` returns `0`. The inscription already holds.
- `fewest_inserts(")(")` returns `2`. One `(` before and one `)` after.
- `fewest_inserts("(((")` returns `3`. Each opener needs its closer.

## Constraints
The inscription has at most 40 runes.
