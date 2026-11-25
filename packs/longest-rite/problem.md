# The Longest Sound Passage

## Task
An inscription of `(` and `)` runes may hold broken stretches where closers and openers tangle. A sound passage is a contiguous run in which every `)` closes the `(` still open just before it and no `(` is left open when the run ends. Call `longest_nested(inscription)` and return the length of the longest sound passage, or `0` when none exists.

## Input
A string of `(` and `)` runes. It may be empty.

## Output
Return the length of the longest sound passage inside the inscription, or `0` when none exists.

## Examples
- `longest_nested("(())()")` returns `6`. The whole inscription is sound.
- `longest_nested(")(()")` returns `2`. The stray `)` breaks the start; the `()` at the end is the longest sound passage.
- `longest_nested("")` returns `0`. An empty inscription holds no passage.

## Constraints
- The inscription has at most 40 runes.
- Only `(` and `)` runes appear.
