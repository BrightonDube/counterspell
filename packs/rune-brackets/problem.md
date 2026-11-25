# The Rune Inscription

## Task
A scribe scratches bracket runes — `(`, `)`, `[`, `]`, `{` and `}` — along the lintel. The inscription holds only when every closing rune lands on the opener still waiting beneath it and no opener is left waiting when the scratching ends. Call `nested_runes(inscription)` and return `1` when the inscription holds, `0` when it does not.

## Input
A string of bracket runes. It may be empty and holds only the six rune characters.

## Output
Return `1` when every closing rune meets the opener still waiting beneath it in nesting order and no opener remains, otherwise `0`. The empty inscription returns `1`.

## Examples
- `nested_runes("()[]{}")` returns `1`. Three sound pairs stand side by side.
- `nested_runes("([)]")` returns `0`. The `)` lands while `[` is still the open rune.
- `nested_runes("((()))")` returns `1`. Each layer closes in turn.

## Constraints
- The inscription has at most 40 runes.
- Only the six rune characters `(`, `)`, `[`, `]`, `{` and `}` appear.
