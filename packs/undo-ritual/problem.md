# The Undo Ritual

## Task
A scribe chants small runes and an undo mark. Every `a`, `b` or `c` is written down; every `#` erases the rune still standing just before it (a `#` with nothing written erases nothing). Call `surviving_text(chant)` and return what remains written.

## Input
A string of runes drawn from `a`, `b`, `c` and `#`. It may be empty.

## Output
Return the text still standing after the whole chant.

## Examples
- `surviving_text("ab#c")` returns `"ac"`. The `#` erases the `b`, then the `c` lands.
- `surviving_text("a##")` returns `""`. The first `#` erases the `a`; the second finds nothing.
- `surviving_text("#ba")` returns `"ba"`. The leading `#` erases nothing.

## Constraints
The chant has at most 40 runes.
