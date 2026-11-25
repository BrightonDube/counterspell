# The Deepest Ember

## Task
A warden reads an inscription of `(` and `)` runes. Each `(` opens a layer; each `)` closes the layer still open, and a `)` with no layer open is ignored as a stray. Call `deepest_ember(inscription)` and return the deepest nesting the inscription ever reached.

## Input
A string of `(` and `)` runes. It may be empty.

## Output
Return the largest number of layers that were ever open at once.

## Examples
- `deepest_ember("(())")` returns `2`. Two layers were open at the middle.
- `deepest_ember(")(")` returns `1`. The stray `)` closes nothing; the `(` opens one layer.
- `deepest_ember(")")` returns `0`. No layer was ever opened.

## Constraints
The inscription has at most 40 runes.
