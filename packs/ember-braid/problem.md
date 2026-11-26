# The Ember Braid

## Task
Two ember strands braid into one line when their letters interleave in order — each strand's letters must appear in the scrawl in their own sequence. Call `braid_ways(text, first, second)` and return `1` when the scrawl is exactly a braid of the two strands.

## Input
`text` is the braided scrawl; `first` and `second` are the strands. Every letter is `a-z`.

## Output
`1` when the braid holds exactly, `0` otherwise — including any length mismatch.

## Examples
- `braid_ways("aabbcc", "abc", "abc")` returns `1`.
- `braid_ways("aab", "a", "ab")` returns `1` — `a|ab` or `aa|b`.
- `braid_ways("ab", "b", "a")` returns `1` — `b` then `a`.

## Constraints
- `0 <= len(text) <= 12`; `0 <= len(first), len(second) <= 6`
