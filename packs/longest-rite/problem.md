# The Longest Sound Passage

## Task
An inscription of `(` and `)` runes may hold broken stretches where closers and openers tangle. Inside it lies at least one sound passage — a run in which every `(` closes in order. Call `longest_nested(inscription)` and return the length of the longest sound passage.

## Input
A string of `(` and `)` runes. It may be empty.

## Output
Return the length of the longest sound run inside the inscription.

## Examples
- `longest_nested("(())()")` returns `6`. The whole inscription is sound.
- `longest_nested(")(()")` returns `2`. The stray `)` breaks the start; the `()` at the end is the longest sound passage.
- `longest_nested("")` returns `0`. An empty inscription holds no passage.

## Constraints
The inscription has at most 40 runes.
