# The Ember Echo

## Task
Two scrawls echo where their letters agree in order — not necessarily adjacent. Call `echo_len(first, second)` and return the longest echo both lines share.

## Input
`first` and `second` are the two scrawls, letters `a-z`.

## Output
The shared echo length; an empty scrawl echoes nothing.

## Examples
- `echo_len("abcde", "ace")` returns `3` — `a`, `c`, `e`.
- `echo_len("rune", "ember")` returns `1` — only the `e`.
- `echo_len("", "abc")` returns `0`.

## Constraints
- `0 <= len(first), len(second) <= 20`
